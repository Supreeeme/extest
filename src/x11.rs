use crate::Display;
use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use std::sync::OnceLock;

type Window = c_ulong;
type XDefaultScreenFn = unsafe extern "C" fn(*mut Display) -> c_int;
type XRootWindowFn = unsafe extern "C" fn(*mut Display, c_int) -> Window;
type XGetGeometryFn = unsafe extern "C" fn(
    *mut Display,
    Window,
    *mut Window,
    *mut c_int,
    *mut c_int,
    *mut c_uint,
    *mut c_uint,
    *mut c_uint,
    *mut c_uint,
) -> c_int;

extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

struct Xlib {
    default_screen: XDefaultScreenFn,
    root_window: XRootWindowFn,
    get_geometry: XGetGeometryFn,
}

// Looked up at runtime rather than linked, so preloading extest into a process
// without libX11 cannot fail. Any caller that passes a Display has it loaded.
fn xlib() -> Option<&'static Xlib> {
    static XLIB: OnceLock<Option<Xlib>> = OnceLock::new();
    XLIB.get_or_init(|| {
        // A null handle is RTLD_DEFAULT on glibc: search the global scope.
        let find = |name: &std::ffi::CStr| unsafe { dlsym(std::ptr::null_mut(), name.as_ptr()) };
        let default_screen = find(c"XDefaultScreen");
        let root_window = find(c"XRootWindow");
        let get_geometry = find(c"XGetGeometry");
        if default_screen.is_null() || root_window.is_null() || get_geometry.is_null() {
            return None;
        }
        // SAFETY: these are the Xlib functions with the signatures declared above.
        unsafe {
            Some(Xlib {
                default_screen: std::mem::transmute::<*mut c_void, XDefaultScreenFn>(
                    default_screen,
                ),
                root_window: std::mem::transmute::<*mut c_void, XRootWindowFn>(root_window),
                get_geometry: std::mem::transmute::<*mut c_void, XGetGeometryFn>(get_geometry),
            })
        }
    })
    .as_ref()
}

/// Returns the current size of the X screen that XTest coordinates are relative to.
/// It is read on every call so monitor hotplug, resume, and scale changes are followed.
pub fn screen_size(display: *mut Display, screen: c_int) -> Option<(u32, u32)> {
    if display.is_null() {
        return None;
    }
    let xlib = xlib()?;
    let (mut root_out, mut x, mut y) = (0, 0, 0);
    let (mut width, mut height, mut border, mut depth) = (0, 0, 0, 0);
    // SAFETY: display is the live Display the caller handed to the XTest function.
    let status = unsafe {
        // XTest uses -1 for "the screen the pointer is on"; the default screen is the only one under XWayland.
        let screen = if screen < 0 {
            (xlib.default_screen)(display)
        } else {
            screen
        };
        let root = (xlib.root_window)(display, screen);
        (xlib.get_geometry)(
            display,
            root,
            &mut root_out,
            &mut x,
            &mut y,
            &mut width,
            &mut height,
            &mut border,
            &mut depth,
        )
    };
    (status != 0 && width > 0 && height > 0).then_some((width, height))
}

#[cfg(test)]
mod tests {
    use super::screen_size;

    #[test]
    fn null_display_has_no_size() {
        assert_eq!(screen_size(std::ptr::null_mut(), 0), None);
    }
}
