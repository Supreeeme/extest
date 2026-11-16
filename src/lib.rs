mod steam_keys;
use steam_keys::KEYS;

mod x11;

use evdev::{
    uinput::VirtualDevice, AbsInfo, AbsoluteAxisCode, AttributeSet, EventType, InputEvent, KeyCode,
    RelativeAxisCode, UinputAbsSetup,
};
use once_cell::sync::Lazy;
use std::ffi::{c_int, c_uint, c_ulong};
use std::sync::Mutex;

// Opaque type
#[repr(C)]
pub struct Display {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

// Absolute positions are sent as a fraction of the X screen in this range, and the
// compositor maps the range onto its whole layout. A fixed range lets the screen
// size change without recreating the device.
const ABS_RANGE: i32 = 1 << 16;

/// Maps an X screen coordinate onto 0..ABS_RANGE, clamping positions off the screen.
fn scale_to_axis(coord: c_int, size: u32) -> i32 {
    let scaled = i64::from(coord) * i64::from(ABS_RANGE) / i64::from(size.max(1));
    scaled.clamp(0, i64::from(ABS_RANGE - 1)) as i32
}

static DEVICE: Lazy<Mutex<VirtualDevice>> = Lazy::new(|| {
    let abs = AbsInfo::new(0, 0, ABS_RANGE - 1, 0, 0, 1);
    Mutex::new(
        VirtualDevice::builder()
            .unwrap()
            .name("extest fake device")
            .with_keys(&AttributeSet::from_iter(
                [
                    KeyCode::BTN_LEFT,
                    KeyCode::BTN_RIGHT,
                    KeyCode::BTN_MIDDLE,
                    KeyCode::BTN_EXTRA,
                    KeyCode::BTN_SIDE,
                ]
                .into_iter()
                .chain(KEYS.iter().copied()),
            ))
            .unwrap()
            .with_relative_axes(&AttributeSet::from_iter([
                RelativeAxisCode::REL_X,
                RelativeAxisCode::REL_Y,
                RelativeAxisCode::REL_WHEEL,
            ]))
            .unwrap()
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, abs))
            .unwrap()
            .with_absolute_axis(&UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, abs))
            .unwrap()
            .build()
            .unwrap(),
    )
});

#[no_mangle]
pub extern "C" fn XTestFakeKeyEvent(
    _: *mut Display,
    keycode: c_uint,
    is_press: bool,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();

    // Seems that X11 keycodes are just 8 + linux keycode - https://wiki.archlinux.org/title/Keyboard_input#Identifying_keycodes
    let key = match keycode {
        156 => KeyCode::KEY_TAB, // I have no idea where this comes from
        keycode => KeyCode::new((keycode - 8) as u16),
    };

    #[cfg(debug_assertions)]
    println!("emitting keycode {key:?}");

    dev.emit(&[InputEvent::new_now(
        EventType::KEY.0,
        key.0,
        is_press as i32,
    )])
    .unwrap();
    1
}

#[repr(u8)]
enum MouseButtons {
    LeftClick = 1,
    MiddleClick = 2,
    RightClick = 3,
    ScrollUp = 4,
    ScrollDown = 5,
    Side = 8,
    Extra = 9,
}

impl TryFrom<u32> for MouseButtons {
    type Error = u32;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use MouseButtons::*;
        match value {
            1 => Ok(LeftClick),
            2 => Ok(MiddleClick),
            3 => Ok(RightClick),
            4 => Ok(ScrollUp),
            5 => Ok(ScrollDown),
            8 => Ok(Side),
            9 => Ok(Extra),
            other => Err(other),
        }
    }
}

#[no_mangle]
pub extern "C" fn XTestFakeButtonEvent(
    _: *mut Display,
    button: c_uint,
    is_press: bool,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();
    // values determined via xev
    let key = match button.try_into() {
        Ok(MouseButtons::LeftClick) => KeyCode::BTN_LEFT,
        Ok(MouseButtons::MiddleClick) => KeyCode::BTN_MIDDLE,
        Ok(MouseButtons::RightClick) => KeyCode::BTN_RIGHT,
        Ok(MouseButtons::Side) => KeyCode::BTN_SIDE,
        Ok(MouseButtons::Extra) => KeyCode::BTN_EXTRA,
        Ok(MouseButtons::ScrollUp | MouseButtons::ScrollDown) => {
            // These are sent with is_press true and is_press false like the other buttons,
            // but we only care about is_press because an "unpressed" scroll event doesn't make
            // sense. Why are these considered "buttons" anyway?
            if is_press {
                let value = match button.try_into() {
                    Ok(MouseButtons::ScrollUp) => 1,
                    Ok(MouseButtons::ScrollDown) => -1,
                    _ => unreachable!(),
                };
                dev.emit(&[InputEvent::new_now(
                    EventType::RELATIVE.0,
                    RelativeAxisCode::REL_WHEEL.0,
                    value,
                )])
                .unwrap();
            }
            return 1;
        }
        Err(other) => {
            println!("WARNING: received unknown keycode {other}");
            return 1;
        }
    };

    #[cfg(debug_assertions)]
    println!("emitting mouse button {key:?}");
    dev.emit(&[InputEvent::new_now(
        EventType::KEY.0,
        key.0,
        is_press as i32,
    )])
    .unwrap();
    1
}

#[no_mangle]
pub extern "C" fn XTestFakeRelativeMotionEvent(
    _: *mut Display,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();
    let events = [
        InputEvent::new_now(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, x),
        InputEvent::new_now(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, y),
    ];
    dev.emit(&events).unwrap();
    1
}

#[no_mangle]
pub extern "C" fn XTestFakeMotionEvent(
    display: *mut Display,
    screen: c_int,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    let Some((width, height)) = x11::screen_size(display, screen) else {
        eprintln!("extest: ignoring absolute motion, the X screen size is unknown");
        return 0;
    };
    let mut dev = DEVICE.lock().unwrap();
    let events = [
        InputEvent::new_now(
            EventType::ABSOLUTE.0,
            AbsoluteAxisCode::ABS_X.0,
            scale_to_axis(x, width),
        ),
        InputEvent::new_now(
            EventType::ABSOLUTE.0,
            AbsoluteAxisCode::ABS_Y.0,
            scale_to_axis(y, height),
        ),
    ];
    dev.emit(&events).unwrap();
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_motion_is_relative_to_the_x_screen() {
        // A 2560x1440 X screen on a 1.25 scaled output: the center must map to the
        // center of the axis, not to 1280/2048 of the logical Wayland size.
        assert_eq!(scale_to_axis(1280, 2560), ABS_RANGE / 2);
        assert_eq!(scale_to_axis(720, 1440), ABS_RANGE / 2);
        assert_eq!(scale_to_axis(0, 2560), 0);
        assert_eq!(scale_to_axis(2559, 2560), ABS_RANGE - 26);
        assert_eq!(scale_to_axis(-5, 2560), 0);
        assert_eq!(scale_to_axis(9000, 2560), ABS_RANGE - 1);
    }
}
