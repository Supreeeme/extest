use evdev::Key;

// Every key that Big Picture allows binding.

pub const KEYS: &[Key] = &[
    Key::KEY_ESC,
    Key::KEY_F1,
    Key::KEY_F2,
    Key::KEY_F3,
    Key::KEY_F4,
    Key::KEY_F5,
    Key::KEY_F6,
    Key::KEY_F7,
    Key::KEY_F8,
    Key::KEY_F9,
    Key::KEY_F10,
    Key::KEY_F11,
    Key::KEY_F12,

    Key::KEY_GRAVE, // `
    Key::KEY_1,
    Key::KEY_2,
    Key::KEY_3,
    Key::KEY_4,
    Key::KEY_5,
    Key::KEY_6,
    Key::KEY_7,
    Key::KEY_8,
    Key::KEY_9,
    Key::KEY_0,
    Key::KEY_MINUS,
    Key::KEY_EQUAL,
    Key::KEY_BACKSPACE,

    Key::KEY_TAB,
    Key::KEY_Q,
    Key::KEY_W,
    Key::KEY_E,
    Key::KEY_R,
    Key::KEY_T,
    Key::KEY_Y,
    Key::KEY_U,
    Key::KEY_I,
    Key::KEY_O,
    Key::KEY_P,
    Key::KEY_LEFTBRACE,
    Key::KEY_RIGHTBRACE,
    Key::KEY_BACKSLASH,

    Key::KEY_CAPSLOCK,
    Key::KEY_A,
    Key::KEY_S,
    Key::KEY_D,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_H,
    Key::KEY_J,
    Key::KEY_K,
    Key::KEY_L,
    Key::KEY_SEMICOLON,
    Key::KEY_APOSTROPHE,
    Key::KEY_ENTER,

    Key::KEY_LEFTSHIFT,
    Key::KEY_Z,
    Key::KEY_X,
    Key::KEY_C,
    Key::KEY_V,
    Key::KEY_B,
    Key::KEY_N,
    Key::KEY_M,
    Key::KEY_COMMA,
    Key::KEY_DOT,
    Key::KEY_SLASH,
    Key::KEY_RIGHTSHIFT,

    Key::KEY_LEFTCTRL,
    Key::KEY_RIGHTCTRL,
    Key::KEY_LEFTMETA,
    Key::KEY_LEFTALT,
    Key::KEY_SPACE,
    Key::KEY_RIGHTALT,

    Key::KEY_VOLUMEUP,
    Key::KEY_VOLUMEDOWN,
    Key::KEY_MUTE,
    Key::KEY_PLAY,
    Key::KEY_STOP,
    Key::KEY_NEXT,
    Key::KEY_PREVIOUS,
    Key::KEY_PREVIOUSSONG,
    Key::KEY_NEXTSONG,
    Key::KEY_PLAYPAUSE,
    Key::KEY_INSERT,
    Key::KEY_HOME,
    Key::KEY_PAGEUP,
    Key::KEY_DELETE,
    Key::KEY_END,
    Key::KEY_PAGEDOWN,

    Key::KEY_UP,
    Key::KEY_LEFT,
    Key::KEY_RIGHT,
    Key::KEY_DOWN,

    Key::KEY_NUMLOCK,
    Key::KEY_KPSLASH,
    Key::KEY_KPASTERISK,
    Key::KEY_KPMINUS,

    Key::KEY_KP7,
    Key::KEY_KP8,
    Key::KEY_KP9,
    Key::KEY_KPPLUS,

    Key::KEY_KP4,
    Key::KEY_KP5,
    Key::KEY_KP6,

    Key::KEY_KP1,
    Key::KEY_KP2,
    Key::KEY_KP3,
    Key::KEY_KPENTER,

    Key::KEY_KP0,
    Key::KEY_KPDOT,
    Key::KEY_KPLEFTPAREN,
    Key::KEY_KPRIGHTPAREN,
    Key::KEY_102ND, // Output by < (less than key)
];
