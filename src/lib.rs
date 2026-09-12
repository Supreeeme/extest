mod steam_keys;
use steam_keys::KEYS;

mod wayland;
use wayland::get_axes_range;

use evdev::{
    uinput::VirtualDevice, AbsInfo, AbsoluteAxisCode, AttributeSet, EventType, InputEvent, KeyCode,
    RelativeAxisCode, UinputAbsSetup,
};
use std::ffi::{c_int, c_uint, c_ulong};
use std::sync::Mutex;

// Opaque type
#[repr(C)]
pub struct Display {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

static DEVICE: std::sync::LazyLock<Mutex<VirtualDevice>> = std::sync::LazyLock::new(|| {
    let size = get_axes_range();
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
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, size.width, 0, 0, 1),
            ))
            .unwrap()
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, size.height, 0, 0, 1),
            ))
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
        i32::from(is_press),
    )])
    .unwrap();
    return 1;
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
        use MouseButtons::{LeftClick, MiddleClick, RightClick, ScrollUp, ScrollDown, Side, Extra};
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
        i32::from(is_press),
    )])
    .unwrap();
    return 1;
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
    return 1;
}

#[no_mangle]
pub extern "C" fn XTestFakeMotionEvent(
    _: *mut Display,
    _: c_int,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();
    let events = [
        InputEvent::new_now(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, x),
        InputEvent::new_now(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_Y.0, y),
    ];
    dev.emit(&events).unwrap();
    return 1;
}
