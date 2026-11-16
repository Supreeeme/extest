mod steam_keys;
use steam_keys::KEYS;

mod wayland;
use wayland::get_axes_range;

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

struct FakeDevice {
    dev: VirtualDevice,
    // False when the display size could not be read, so absolute motion is unsupported.
    has_abs: bool,
}

fn build_device() -> std::io::Result<FakeDevice> {
    let size = get_axes_range()
        .map_err(|e| eprintln!("extest: absolute pointer motion disabled: {e}"))
        .ok();
    let mut builder = VirtualDevice::builder()?
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
        ))?
        .with_relative_axes(&AttributeSet::from_iter([
            RelativeAxisCode::REL_X,
            RelativeAxisCode::REL_Y,
            RelativeAxisCode::REL_WHEEL,
        ]))?;
    if let Some(size) = size {
        builder = builder
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                AbsInfo::new(0, 0, size.width, 0, 0, 1),
            ))?
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                AbsInfo::new(0, 0, size.height, 0, 0, 1),
            ))?;
    }
    Ok(FakeDevice {
        dev: builder.build()?,
        has_abs: size.is_some(),
    })
}

static DEVICE: Lazy<Option<Mutex<FakeDevice>>> = Lazy::new(|| match build_device() {
    Ok(dev) => Some(Mutex::new(dev)),
    Err(e) => {
        eprintln!("extest: could not create the virtual input device, fake input is disabled: {e}");
        None
    }
});

/// Runs `f` on the device and maps the outcome to an XTest return value.
/// These functions are called from Steam, where a panic would abort the whole client,
/// so every failure is logged and reported as 0 instead.
fn with_device<T>(
    device: Option<&Mutex<T>>,
    f: impl FnOnce(&mut T) -> std::io::Result<()>,
) -> c_int {
    let Some(device) = device else { return 0 };
    let mut dev = device.lock().unwrap_or_else(|e| e.into_inner());
    match f(&mut dev) {
        Ok(()) => 1,
        Err(e) => {
            eprintln!("extest: failed to emit input event: {e}");
            0
        }
    }
}

fn emit(events: &[InputEvent]) -> c_int {
    with_device(DEVICE.as_ref(), |d| d.dev.emit(events))
}

/// X11 keycodes are the Linux keycode plus 8 - https://wiki.archlinux.org/title/Keyboard_input#Identifying_keycodes
fn x11_keycode_to_key(keycode: c_uint) -> Option<KeyCode> {
    match keycode {
        156 => Some(KeyCode::KEY_TAB), // I have no idea where this comes from
        keycode => keycode
            .checked_sub(8)
            .and_then(|code| u16::try_from(code).ok())
            .map(KeyCode::new),
    }
}

#[no_mangle]
pub extern "C" fn XTestFakeKeyEvent(
    _: *mut Display,
    keycode: c_uint,
    is_press: bool,
    _: c_ulong,
) -> c_int {
    let Some(key) = x11_keycode_to_key(keycode) else {
        eprintln!("extest: ignoring invalid X11 keycode {keycode}");
        return 0;
    };

    #[cfg(debug_assertions)]
    println!("emitting keycode {key:?}");

    emit(&[InputEvent::new_now(
        EventType::KEY.0,
        key.0,
        is_press as i32,
    )])
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
                return emit(&[InputEvent::new_now(
                    EventType::RELATIVE.0,
                    RelativeAxisCode::REL_WHEEL.0,
                    value,
                )]);
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
    emit(&[InputEvent::new_now(
        EventType::KEY.0,
        key.0,
        is_press as i32,
    )])
}

#[no_mangle]
pub extern "C" fn XTestFakeRelativeMotionEvent(
    _: *mut Display,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    emit(&[
        InputEvent::new_now(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, x),
        InputEvent::new_now(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, y),
    ])
}

#[no_mangle]
pub extern "C" fn XTestFakeMotionEvent(
    _: *mut Display,
    _: c_int,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    let events = [
        InputEvent::new_now(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, x),
        InputEvent::new_now(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_Y.0, y),
    ];
    with_device(DEVICE.as_ref(), |d| {
        if d.has_abs {
            d.dev.emit(&events)
        } else {
            Err(std::io::Error::other("absolute motion is unsupported"))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn keycodes_below_the_x11_offset_are_rejected() {
        for keycode in 0..8 {
            assert_eq!(x11_keycode_to_key(keycode), None);
        }
        assert_eq!(x11_keycode_to_key(u32::MAX), None);
        assert_eq!(x11_keycode_to_key(9), Some(KeyCode::KEY_ESC));
        assert_eq!(x11_keycode_to_key(156), Some(KeyCode::KEY_TAB));
    }

    #[test]
    fn missing_device_returns_zero() {
        assert_eq!(with_device::<()>(None, |_| Ok(())), 0);
    }

    #[test]
    fn emit_error_returns_zero() {
        let dev = Mutex::new(());
        assert_eq!(
            with_device(Some(&dev), |_| Err(io::Error::from(io::ErrorKind::PermissionDenied))),
            0
        );
        assert_eq!(with_device(Some(&dev), |_| Ok(())), 1);
    }
}
