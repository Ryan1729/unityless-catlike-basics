use sokol_bindings_sys as sys;
use crate::{Int, UInt};

/// The expected Frames Per Second. That is, the number of times a second the
/// frame callbacks are usually called.
pub const FPS: u8 = 60;

#[derive(Default)]
pub struct Desc<UserData = ()>
where UserData: Default {
pub init_cb: ::core::option::Option<unsafe extern "C" fn()>,
    pub frame_cb: ::core::option::Option<unsafe extern "C" fn()>,
    pub cleanup_cb: ::core::option::Option<unsafe extern "C" fn()>,
    pub event_cb: ::core::option::Option<unsafe extern "C" fn(arg1: *const SysEvent)>,
    pub fail_cb: ::core::option::Option<unsafe extern "C" fn(arg1: *const ::std::os::raw::c_char)>,
    pub user_data: UserData,
    pub init_userdata_cb:
        ::core::option::Option<unsafe extern "C" fn(arg1: *mut ::core::ffi::c_void)>,
    pub frame_userdata_cb:
        ::core::option::Option<unsafe extern "C" fn(arg1: *mut ::core::ffi::c_void)>,
    pub cleanup_userdata_cb:
        ::core::option::Option<unsafe extern "C" fn(arg1: *mut ::core::ffi::c_void)>,
    pub event_userdata_cb: ::core::option::Option<
        unsafe extern "C" fn(arg1: *const SysEvent, arg2: *mut ::core::ffi::c_void),
    >,
    pub fail_userdata_cb: ::core::option::Option<
        unsafe extern "C" fn(arg1: *const ::std::os::raw::c_char, arg2: *mut ::core::ffi::c_void),
    >,
    pub width: Int,
    pub height: Int,
    pub sample_count: Int,
    pub swap_interval: Int,
    pub high_dpi: bool,
    pub fullscreen: bool,
    pub alpha: bool,
    pub window_title: &'static str,
    pub user_cursor: bool,
    pub enable_clipboard: bool,
    pub clipboard_size: Int,
    pub enable_dragndrop: bool,
    pub max_dropped_files: Int,
    pub max_dropped_file_path_length: Int,
    pub icon: IconDesc,
    pub gl_force_gles2: bool,
    pub win32_console_utf8: bool,
    pub win32_console_create: bool,
    pub win32_console_attach: bool,
    pub html5_canvas_name: &'static str,
    pub html5_canvas_resize: bool,
    pub html5_preserve_drawing_buffer: bool,
    pub html5_premultiplied_alpha: bool,
    pub html5_ask_leave_site: bool,
    pub ios_keyboard_resizes_canvas: bool,
}

pub fn run<UserData>(mut desc: Desc<UserData>)
where UserData: Default {
    // Don't allow both user-data and non-user-data callbacks to be passed for a
    // any of the callback types, since they will not both be called.

    macro_rules! a {
        ($($cb: ident $user_data_cb: ident);+ $(;)?) => {
            $(
                assert!(
                    desc.$cb.is_none() || desc.$user_data_cb.is_none(),
                    concat!(
                        "Only pass one of ",
                        stringify!($cb),
                        " and ",
                        stringify!($user_data_cb),
                        " since only one of them will be called."
                    )
                );
            )+
        }
    }
    a!(
        init_cb init_userdata_cb;
        frame_cb frame_userdata_cb;
        cleanup_cb cleanup_userdata_cb;
        event_cb event_userdata_cb;
        fail_cb fail_userdata_cb;
    );

    let mut desc_parameter = sys::sapp_desc::default();

    desc_parameter.init_cb = desc.init_cb;
    desc_parameter.frame_cb = desc.frame_cb;
    desc_parameter.cleanup_cb = desc.cleanup_cb;
    desc_parameter.event_cb = desc.event_cb;
    desc_parameter.fail_cb = desc.fail_cb;

    desc_parameter.user_data = &mut desc.user_data as *mut UserData as _;

    desc_parameter.init_userdata_cb = desc.init_userdata_cb;
    desc_parameter.frame_userdata_cb = desc.frame_userdata_cb;
    desc_parameter.cleanup_userdata_cb = desc.cleanup_userdata_cb;
    desc_parameter.event_userdata_cb = desc.event_userdata_cb;
    desc_parameter.fail_userdata_cb = desc.fail_userdata_cb;

    desc_parameter.width = desc.width;
    desc_parameter.height = desc.height;
    desc_parameter.sample_count = desc.sample_count;
    desc_parameter.swap_interval = desc.swap_interval;
    desc_parameter.high_dpi = desc.high_dpi;
    desc_parameter.fullscreen = desc.fullscreen;
    desc_parameter.alpha = desc.alpha;
    desc_parameter.window_title = desc.window_title.as_ptr() as _;
    desc_parameter.user_cursor = desc.user_cursor;
    desc_parameter.enable_clipboard = desc.enable_clipboard;
    desc_parameter.clipboard_size = desc.clipboard_size;
    desc_parameter.enable_dragndrop = desc.enable_dragndrop;
    desc_parameter.max_dropped_files = desc.max_dropped_files;
    desc_parameter.max_dropped_file_path_length = desc.max_dropped_file_path_length;
    desc_parameter.icon = desc.icon;
    desc_parameter.gl_force_gles2 = desc.gl_force_gles2;
    desc_parameter.win32_console_utf8 = desc.win32_console_utf8;
    desc_parameter.win32_console_create = desc.win32_console_create;
    desc_parameter.win32_console_attach = desc.win32_console_attach;
    desc_parameter.html5_canvas_name = desc.html5_canvas_name.as_ptr() as _;
    desc_parameter.html5_canvas_resize = desc.html5_canvas_resize;
    desc_parameter.html5_preserve_drawing_buffer = desc.html5_preserve_drawing_buffer;
    desc_parameter.html5_premultiplied_alpha = desc.html5_premultiplied_alpha;
    desc_parameter.html5_ask_leave_site = desc.html5_ask_leave_site;
    desc_parameter.ios_keyboard_resizes_canvas = desc.ios_keyboard_resizes_canvas;

    // SAFETY: The generic `Desc<_>` type prevents the userdata from being
    // used as a different type, which prevents it from being the wrong size.
    unsafe { sys::sapp_run(&desc_parameter) }
}

/// This macro calls sapp::run for you. It syntactically ensures that the same
/// userdata type is used for each callback passed in the cbs section. It
/// assumes your userdata type implements `Default`. It also restrcts you to using
/// only userdata callbacks, but you can just ignore the parameter if you like. The
/// only userdata callbacks restriciton is mainly to simplfy the macro itself.
#[macro_export]
macro_rules! _run_with_userdata {
    (
        cbs: {
            type: $type: ty,
            $(init: $init: ident,)?
            $(frame: $frame: ident,)?
            $(cleanup: $cleanup: ident,)?
            $(event: $event: ident,)?
            $(fail: $fail: ident,)?
        },
        $desc: expr $(,)?
    ) => {
        let mut desc: $crate::sapp::Desc<$type> = $desc;

        {
            // Don't allow callbacks not passed in through the cbs section to avoid
            // possible unexpected behaviour.
            macro_rules! a {
                // Can't do $($field: ident)+ in inner macros
                // See https://github.com/rust-lang/rust/issues/35853
                ($field: ident) => {
                    assert!(
                        desc.$field.is_none(),
                        "Only pass callbacks through the cbs section"
                    );
                }
            }
            a!(init_cb);
            a!(frame_cb);
            a!(cleanup_cb);
            a!(event_cb);
            a!(fail_cb);
            a!(init_userdata_cb);
            a!(frame_userdata_cb);
            a!(cleanup_userdata_cb);
            a!(event_userdata_cb);
            a!(fail_userdata_cb);
        }

        desc.user_data = <$type>::default();

        $( desc.init_userdata_cb = $crate::cb_wrap_userdata!($init : fn(&mut $type)); )?
        $( desc.frame_userdata_cb = $crate::cb_wrap_userdata!($frame : fn(&mut $type)); )?
        $( desc.cleanup_userdata_cb = $crate::cb_wrap_userdata!($cleanup : fn(&mut $type)); )?
        $(
            desc.event_userdata_cb = {
                unsafe extern "C" fn cb_extern(
                    event: *const $crate::sapp::SysEvent,
                    userdata: *mut ::std::os::raw::c_void
                ) {
                    let mut event_paramter = $crate::sapp::Event::from(&*event);

                    // SAFETY: The macro containing this code prevents the userdata
                    // from being used as a different type, which prevents it from
                    // being the wrong size.
                    let userdata_parameter: &mut $type = unsafe { &mut*(userdata as *mut $type) };

                    $event(&event_paramter, userdata_parameter)
                }

                Some(cb_extern)
            };
        )?
        $(
            desc.fail_userdata_cb = {
                unsafe extern "C" fn cb_extern(
                    msg: *const ::std::os::raw::c_char,
                    userdata: *mut ::std::os::raw::c_void
                ) {
                    // SAFETY: Sokol passes us a valid, nul-terminated pointer with
                    // an appropriate lifetime. As of this writing, only C string
                    // literals are ever passed down, so we're safe there.
                    let msg_parameter = unsafe { std::ffi::CStr::from_ptr(msg) };
                    // SAFETY: The macro containing this code prevents the userdata
                    // from being used as a different type, which prevents it from
                    // being the wrong size.
                    let userdata_parameter: &mut $type = unsafe { &mut*(userdata as *mut $type) };


                    $fail(msg_parameter, userdata_parameter)
                }

                Some(cb_extern)
            };
        )?

        $crate::sapp::run(desc);
    };
}
pub use _run_with_userdata as run_with_userdata;

pub type IconDesc = sys::sapp_icon_desc;

pub type Touchpoint = sys::sapp_touchpoint;

pub fn width() -> Int {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sapp_width() }
}

pub fn height() -> Int {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sapp_height() }
}

pub fn frame_duration() -> f64 {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sapp_frame_duration() }
}

pub const MAX_TOUCHPOINTS: u8 = 8;

pub use sys::sapp_event as SysEvent;

#[derive(Default)]
pub struct Event {
    pub frame_count: u64,
    pub kind: EventKind,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub window_width: Int,
    pub window_height: Int,
    pub framebuffer_width: Int,
    pub framebuffer_height: Int,
}

impl From<&SysEvent> for Event {
    fn from(sys_event: &SysEvent) -> Self {
        let mut output = Self::default();

        output.frame_count = sys_event.frame_count;

        use EventKind::*;

        output.kind = match sys_event.type_ {
            sys::sapp_event_type_SAPP_EVENTTYPE_KEY_DOWN => KeyDown {
                key_code: sys_event.key_code.into(),
                key_repeat: sys_event.key_repeat,
                modifiers: sys_event.modifiers,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_KEY_UP => KeyUp {
                key_code: sys_event.key_code.into(),
                key_repeat: sys_event.key_repeat,
                modifiers: sys_event.modifiers,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_CHAR => Char {
                char_code: sys_event.char_code,
                key_repeat: sys_event.key_repeat,
                modifiers: sys_event.modifiers,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_MOUSE_DOWN => MouseDown {
                mouse_button: sys_event.mouse_button.into(),
                modifiers: sys_event.modifiers,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_MOUSE_UP => MouseUp {
                mouse_button: sys_event.mouse_button.into(),
                modifiers: sys_event.modifiers,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_MOUSE_SCROLL => MouseScroll {
                scroll_x: sys_event.scroll_x,
                scroll_y: sys_event.scroll_y,
                modifiers: sys_event.modifiers,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_MOUSE_MOVE => MouseMove {
                modifiers: sys_event.modifiers
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_MOUSE_ENTER => MouseEnter {
                modifiers: sys_event.modifiers
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_MOUSE_LEAVE => MouseLeave {
                modifiers: sys_event.modifiers
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_TOUCHES_BEGAN => TouchesBegan {
                num_touches: sys_event.num_touches,
                touches: sys_event.touches,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_TOUCHES_MOVED => TouchesMoved {
                num_touches: sys_event.num_touches,
                touches: sys_event.touches,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_TOUCHES_ENDED => TouchesEnded {
                num_touches: sys_event.num_touches,
                touches: sys_event.touches,
            },
            sys::sapp_event_type_SAPP_EVENTTYPE_TOUCHES_CANCELLED => TouchesCancelled,
            sys::sapp_event_type_SAPP_EVENTTYPE_RESIZED => Resized,
            sys::sapp_event_type_SAPP_EVENTTYPE_ICONIFIED => Iconified,
            sys::sapp_event_type_SAPP_EVENTTYPE_RESTORED => Restored,
            sys::sapp_event_type_SAPP_EVENTTYPE_FOCUSED => Focused,
            sys::sapp_event_type_SAPP_EVENTTYPE_UNFOCUSED => Unfocused,
            sys::sapp_event_type_SAPP_EVENTTYPE_SUSPENDED => Suspended,
            sys::sapp_event_type_SAPP_EVENTTYPE_RESUMED => Resumed,
            sys::sapp_event_type_SAPP_EVENTTYPE_UPDATE_CURSOR => UpdateCursor,
            sys::sapp_event_type_SAPP_EVENTTYPE_QUIT_REQUESTED => QuitRequested,
            sys::sapp_event_type_SAPP_EVENTTYPE_CLIPBOARD_PASTED => ClipboardPasted,
            sys::sapp_event_type_SAPP_EVENTTYPE_FILES_DROPPED => FilesDropped,
            _ => Invalid,
        };

        output.mouse_x = sys_event.mouse_x;
        output.mouse_y = sys_event.mouse_y;
        output.mouse_dx = sys_event.mouse_dx;
        output.mouse_dy = sys_event.mouse_dy;
        output.window_width = sys_event.window_width;
        output.window_height = sys_event.window_height;
        output.framebuffer_width = sys_event.framebuffer_width;
        output.framebuffer_height = sys_event.framebuffer_height;

        output
    }
}

pub type Modifiers = u32;

pub const SHIFT: Modifiers = 0x1;
pub const CTRL: Modifiers  = 0x2;
pub const ALT: Modifiers   = 0x4;
pub const SUPER: Modifiers = 0x8;
pub const LMB: Modifiers   = 0x100;
pub const RMB: Modifiers   = 0x200;
pub const MMB: Modifiers   = 0x400;

// TODO can this just be `char`?
pub type CharCode = u32;

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum EventKind {
    Invalid,
    KeyDown { key_code: KeyCode, key_repeat: bool, modifiers: Modifiers },
    KeyUp { key_code: KeyCode, key_repeat: bool, modifiers: Modifiers },
    Char { char_code: CharCode, key_repeat: bool, modifiers: Modifiers },
    MouseDown { mouse_button: MouseButton, modifiers: Modifiers },
    MouseUp { mouse_button: MouseButton, modifiers: Modifiers },
    MouseScroll { scroll_x: f32, scroll_y: f32, modifiers: Modifiers },
    MouseMove { modifiers: Modifiers },
    MouseEnter { modifiers: Modifiers },
    MouseLeave { modifiers: Modifiers },
    TouchesBegan { num_touches: Int, touches: [Touchpoint; MAX_TOUCHPOINTS as usize] },
    TouchesMoved { num_touches: Int, touches: [Touchpoint; MAX_TOUCHPOINTS as usize] },
    TouchesEnded { num_touches: Int, touches: [Touchpoint; MAX_TOUCHPOINTS as usize] },
    TouchesCancelled,
    Resized,
    Iconified,
    Restored,
    Focused,
    Unfocused,
    Suspended,
    Resumed,
    UpdateCursor,
    QuitRequested,
    ClipboardPasted,
    FilesDropped,
}

impl Default for EventKind {
    fn default() -> Self {
        Self::Invalid
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum MouseButton {
    Left = sys::sapp_mousebutton_SAPP_MOUSEBUTTON_LEFT,
    Right = sys::sapp_mousebutton_SAPP_MOUSEBUTTON_RIGHT,
    Middle = sys::sapp_mousebutton_SAPP_MOUSEBUTTON_MIDDLE,
    Invalid = sys::sapp_mousebutton_SAPP_MOUSEBUTTON_INVALID,
}

impl From<UInt> for MouseButton {
    fn from(uint: UInt) -> Self {
        use MouseButton::*;
        match uint {
            sys::sapp_mousebutton_SAPP_MOUSEBUTTON_LEFT => Left,
            sys::sapp_mousebutton_SAPP_MOUSEBUTTON_RIGHT => Right,
            sys::sapp_mousebutton_SAPP_MOUSEBUTTON_MIDDLE => Middle,
            _ => Invalid,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Invalid = sys::sapp_keycode_SAPP_KEYCODE_INVALID,
    Space = sys::sapp_keycode_SAPP_KEYCODE_SPACE,
    Apostrophe = sys::sapp_keycode_SAPP_KEYCODE_APOSTROPHE,
    Comma = sys::sapp_keycode_SAPP_KEYCODE_COMMA,
    Minus = sys::sapp_keycode_SAPP_KEYCODE_MINUS,
    Period = sys::sapp_keycode_SAPP_KEYCODE_PERIOD,
    Slash = sys::sapp_keycode_SAPP_KEYCODE_SLASH,
    _0 = sys::sapp_keycode_SAPP_KEYCODE_0,
    _1 = sys::sapp_keycode_SAPP_KEYCODE_1,
    _2 = sys::sapp_keycode_SAPP_KEYCODE_2,
    _3 = sys::sapp_keycode_SAPP_KEYCODE_3,
    _4 = sys::sapp_keycode_SAPP_KEYCODE_4,
    _5 = sys::sapp_keycode_SAPP_KEYCODE_5,
    _6 = sys::sapp_keycode_SAPP_KEYCODE_6,
    _7 = sys::sapp_keycode_SAPP_KEYCODE_7,
    _8 = sys::sapp_keycode_SAPP_KEYCODE_8,
    _9 = sys::sapp_keycode_SAPP_KEYCODE_9,
    Semicolon = sys::sapp_keycode_SAPP_KEYCODE_SEMICOLON,
    Equal = sys::sapp_keycode_SAPP_KEYCODE_EQUAL,
    A = sys::sapp_keycode_SAPP_KEYCODE_A,
    B = sys::sapp_keycode_SAPP_KEYCODE_B,
    C = sys::sapp_keycode_SAPP_KEYCODE_C,
    D = sys::sapp_keycode_SAPP_KEYCODE_D,
    E = sys::sapp_keycode_SAPP_KEYCODE_E,
    F = sys::sapp_keycode_SAPP_KEYCODE_F,
    G = sys::sapp_keycode_SAPP_KEYCODE_G,
    H = sys::sapp_keycode_SAPP_KEYCODE_H,
    I = sys::sapp_keycode_SAPP_KEYCODE_I,
    J = sys::sapp_keycode_SAPP_KEYCODE_J,
    K = sys::sapp_keycode_SAPP_KEYCODE_K,
    L = sys::sapp_keycode_SAPP_KEYCODE_L,
    M = sys::sapp_keycode_SAPP_KEYCODE_M,
    N = sys::sapp_keycode_SAPP_KEYCODE_N,
    O = sys::sapp_keycode_SAPP_KEYCODE_O,
    P = sys::sapp_keycode_SAPP_KEYCODE_P,
    Q = sys::sapp_keycode_SAPP_KEYCODE_Q,
    R = sys::sapp_keycode_SAPP_KEYCODE_R,
    S = sys::sapp_keycode_SAPP_KEYCODE_S,
    T = sys::sapp_keycode_SAPP_KEYCODE_T,
    U = sys::sapp_keycode_SAPP_KEYCODE_U,
    V = sys::sapp_keycode_SAPP_KEYCODE_V,
    W = sys::sapp_keycode_SAPP_KEYCODE_W,
    X = sys::sapp_keycode_SAPP_KEYCODE_X,
    Y = sys::sapp_keycode_SAPP_KEYCODE_Y,
    Z = sys::sapp_keycode_SAPP_KEYCODE_Z,
    LeftBracket = sys::sapp_keycode_SAPP_KEYCODE_LEFT_BRACKET,
    Backslash = sys::sapp_keycode_SAPP_KEYCODE_BACKSLASH,
    RightBracket = sys::sapp_keycode_SAPP_KEYCODE_RIGHT_BRACKET,
    GraveAccent = sys::sapp_keycode_SAPP_KEYCODE_GRAVE_ACCENT,
    World1 = sys::sapp_keycode_SAPP_KEYCODE_WORLD_1,
    World2 = sys::sapp_keycode_SAPP_KEYCODE_WORLD_2,
    Escape = sys::sapp_keycode_SAPP_KEYCODE_ESCAPE,
    Enter = sys::sapp_keycode_SAPP_KEYCODE_ENTER,
    Tab = sys::sapp_keycode_SAPP_KEYCODE_TAB,
    Backspace = sys::sapp_keycode_SAPP_KEYCODE_BACKSPACE,
    Insert = sys::sapp_keycode_SAPP_KEYCODE_INSERT,
    Delete = sys::sapp_keycode_SAPP_KEYCODE_DELETE,
    Right = sys::sapp_keycode_SAPP_KEYCODE_RIGHT,
    Left = sys::sapp_keycode_SAPP_KEYCODE_LEFT,
    Down = sys::sapp_keycode_SAPP_KEYCODE_DOWN,
    Up = sys::sapp_keycode_SAPP_KEYCODE_UP,
    PageUp = sys::sapp_keycode_SAPP_KEYCODE_PAGE_UP,
    PageDown = sys::sapp_keycode_SAPP_KEYCODE_PAGE_DOWN,
    Home = sys::sapp_keycode_SAPP_KEYCODE_HOME,
    End = sys::sapp_keycode_SAPP_KEYCODE_END,
    CapsLock = sys::sapp_keycode_SAPP_KEYCODE_CAPS_LOCK,
    ScrollLock = sys::sapp_keycode_SAPP_KEYCODE_SCROLL_LOCK,
    NumLock = sys::sapp_keycode_SAPP_KEYCODE_NUM_LOCK,
    PrintScreen = sys::sapp_keycode_SAPP_KEYCODE_PRINT_SCREEN,
    Pause = sys::sapp_keycode_SAPP_KEYCODE_PAUSE,
    F1 = sys::sapp_keycode_SAPP_KEYCODE_F1,
    F2 = sys::sapp_keycode_SAPP_KEYCODE_F2,
    F3 = sys::sapp_keycode_SAPP_KEYCODE_F3,
    F4 = sys::sapp_keycode_SAPP_KEYCODE_F4,
    F5 = sys::sapp_keycode_SAPP_KEYCODE_F5,
    F6 = sys::sapp_keycode_SAPP_KEYCODE_F6,
    F7 = sys::sapp_keycode_SAPP_KEYCODE_F7,
    F8 = sys::sapp_keycode_SAPP_KEYCODE_F8,
    F9 = sys::sapp_keycode_SAPP_KEYCODE_F9,
    F10 = sys::sapp_keycode_SAPP_KEYCODE_F10,
    F11 = sys::sapp_keycode_SAPP_KEYCODE_F11,
    F12 = sys::sapp_keycode_SAPP_KEYCODE_F12,
    F13 = sys::sapp_keycode_SAPP_KEYCODE_F13,
    F14 = sys::sapp_keycode_SAPP_KEYCODE_F14,
    F15 = sys::sapp_keycode_SAPP_KEYCODE_F15,
    F16 = sys::sapp_keycode_SAPP_KEYCODE_F16,
    F17 = sys::sapp_keycode_SAPP_KEYCODE_F17,
    F18 = sys::sapp_keycode_SAPP_KEYCODE_F18,
    F19 = sys::sapp_keycode_SAPP_KEYCODE_F19,
    F20 = sys::sapp_keycode_SAPP_KEYCODE_F20,
    F21 = sys::sapp_keycode_SAPP_KEYCODE_F21,
    F22 = sys::sapp_keycode_SAPP_KEYCODE_F22,
    F23 = sys::sapp_keycode_SAPP_KEYCODE_F23,
    F24 = sys::sapp_keycode_SAPP_KEYCODE_F24,
    F25 = sys::sapp_keycode_SAPP_KEYCODE_F25,
    KP0 = sys::sapp_keycode_SAPP_KEYCODE_KP_0,
    KP1 = sys::sapp_keycode_SAPP_KEYCODE_KP_1,
    KP2 = sys::sapp_keycode_SAPP_KEYCODE_KP_2,
    KP3 = sys::sapp_keycode_SAPP_KEYCODE_KP_3,
    KP4 = sys::sapp_keycode_SAPP_KEYCODE_KP_4,
    KP5 = sys::sapp_keycode_SAPP_KEYCODE_KP_5,
    KP6 = sys::sapp_keycode_SAPP_KEYCODE_KP_6,
    KP7 = sys::sapp_keycode_SAPP_KEYCODE_KP_7,
    KP8 = sys::sapp_keycode_SAPP_KEYCODE_KP_8,
    KP9 = sys::sapp_keycode_SAPP_KEYCODE_KP_9,
    KPDecimal = sys::sapp_keycode_SAPP_KEYCODE_KP_DECIMAL,
    KPDivide = sys::sapp_keycode_SAPP_KEYCODE_KP_DIVIDE,
    KPMultiply = sys::sapp_keycode_SAPP_KEYCODE_KP_MULTIPLY,
    KPSubtract = sys::sapp_keycode_SAPP_KEYCODE_KP_SUBTRACT,
    KPAdd = sys::sapp_keycode_SAPP_KEYCODE_KP_ADD,
    KPEnter = sys::sapp_keycode_SAPP_KEYCODE_KP_ENTER,
    KPEqual = sys::sapp_keycode_SAPP_KEYCODE_KP_EQUAL,
    LeftShift = sys::sapp_keycode_SAPP_KEYCODE_LEFT_SHIFT,
    LeftControl = sys::sapp_keycode_SAPP_KEYCODE_LEFT_CONTROL,
    LeftAlt = sys::sapp_keycode_SAPP_KEYCODE_LEFT_ALT,
    LeftSuper = sys::sapp_keycode_SAPP_KEYCODE_LEFT_SUPER,
    RightShift = sys::sapp_keycode_SAPP_KEYCODE_RIGHT_SHIFT,
    RightControl = sys::sapp_keycode_SAPP_KEYCODE_RIGHT_CONTROL,
    RightAlt = sys::sapp_keycode_SAPP_KEYCODE_RIGHT_ALT,
    RightSuper = sys::sapp_keycode_SAPP_KEYCODE_RIGHT_SUPER,
    Menu = sys::sapp_keycode_SAPP_KEYCODE_MENU,
}

impl KeyCode {
    #[allow(non_upper_case_globals)]
    pub const Plus: Self = Self::Equal;
}

impl From<UInt> for KeyCode {
    fn from(uint: UInt) -> Self {
        use KeyCode::*;
        match uint {
            sys::sapp_keycode_SAPP_KEYCODE_SPACE => Space,
            sys::sapp_keycode_SAPP_KEYCODE_APOSTROPHE => Apostrophe,
            sys::sapp_keycode_SAPP_KEYCODE_COMMA => Comma,
            sys::sapp_keycode_SAPP_KEYCODE_MINUS => Minus,
            sys::sapp_keycode_SAPP_KEYCODE_PERIOD => Period,
            sys::sapp_keycode_SAPP_KEYCODE_SLASH => Slash,
            sys::sapp_keycode_SAPP_KEYCODE_0 => _0,
            sys::sapp_keycode_SAPP_KEYCODE_1 => _1,
            sys::sapp_keycode_SAPP_KEYCODE_2 => _2,
            sys::sapp_keycode_SAPP_KEYCODE_3 => _3,
            sys::sapp_keycode_SAPP_KEYCODE_4 => _4,
            sys::sapp_keycode_SAPP_KEYCODE_5 => _5,
            sys::sapp_keycode_SAPP_KEYCODE_6 => _6,
            sys::sapp_keycode_SAPP_KEYCODE_7 => _7,
            sys::sapp_keycode_SAPP_KEYCODE_8 => _8,
            sys::sapp_keycode_SAPP_KEYCODE_9 => _9,
            sys::sapp_keycode_SAPP_KEYCODE_SEMICOLON => Semicolon,
            sys::sapp_keycode_SAPP_KEYCODE_EQUAL => Equal,
            sys::sapp_keycode_SAPP_KEYCODE_A => A,
            sys::sapp_keycode_SAPP_KEYCODE_B => B,
            sys::sapp_keycode_SAPP_KEYCODE_C => C,
            sys::sapp_keycode_SAPP_KEYCODE_D => D,
            sys::sapp_keycode_SAPP_KEYCODE_E => E,
            sys::sapp_keycode_SAPP_KEYCODE_F => F,
            sys::sapp_keycode_SAPP_KEYCODE_G => G,
            sys::sapp_keycode_SAPP_KEYCODE_H => H,
            sys::sapp_keycode_SAPP_KEYCODE_I => I,
            sys::sapp_keycode_SAPP_KEYCODE_J => J,
            sys::sapp_keycode_SAPP_KEYCODE_K => K,
            sys::sapp_keycode_SAPP_KEYCODE_L => L,
            sys::sapp_keycode_SAPP_KEYCODE_M => M,
            sys::sapp_keycode_SAPP_KEYCODE_N => N,
            sys::sapp_keycode_SAPP_KEYCODE_O => O,
            sys::sapp_keycode_SAPP_KEYCODE_P => P,
            sys::sapp_keycode_SAPP_KEYCODE_Q => Q,
            sys::sapp_keycode_SAPP_KEYCODE_R => R,
            sys::sapp_keycode_SAPP_KEYCODE_S => S,
            sys::sapp_keycode_SAPP_KEYCODE_T => T,
            sys::sapp_keycode_SAPP_KEYCODE_U => U,
            sys::sapp_keycode_SAPP_KEYCODE_V => V,
            sys::sapp_keycode_SAPP_KEYCODE_W => W,
            sys::sapp_keycode_SAPP_KEYCODE_X => X,
            sys::sapp_keycode_SAPP_KEYCODE_Y => Y,
            sys::sapp_keycode_SAPP_KEYCODE_Z => Z,
            sys::sapp_keycode_SAPP_KEYCODE_LEFT_BRACKET => LeftBracket,
            sys::sapp_keycode_SAPP_KEYCODE_BACKSLASH => Backslash,
            sys::sapp_keycode_SAPP_KEYCODE_RIGHT_BRACKET => RightBracket,
            sys::sapp_keycode_SAPP_KEYCODE_GRAVE_ACCENT => GraveAccent,
            sys::sapp_keycode_SAPP_KEYCODE_WORLD_1 => World1,
            sys::sapp_keycode_SAPP_KEYCODE_WORLD_2 => World2,
            sys::sapp_keycode_SAPP_KEYCODE_ESCAPE => Escape,
            sys::sapp_keycode_SAPP_KEYCODE_ENTER => Enter,
            sys::sapp_keycode_SAPP_KEYCODE_TAB => Tab,
            sys::sapp_keycode_SAPP_KEYCODE_BACKSPACE => Backspace,
            sys::sapp_keycode_SAPP_KEYCODE_INSERT => Insert,
            sys::sapp_keycode_SAPP_KEYCODE_DELETE => Delete,
            sys::sapp_keycode_SAPP_KEYCODE_RIGHT => Right,
            sys::sapp_keycode_SAPP_KEYCODE_LEFT => Left,
            sys::sapp_keycode_SAPP_KEYCODE_DOWN => Down,
            sys::sapp_keycode_SAPP_KEYCODE_UP => Up,
            sys::sapp_keycode_SAPP_KEYCODE_PAGE_UP => PageUp,
            sys::sapp_keycode_SAPP_KEYCODE_PAGE_DOWN => PageDown,
            sys::sapp_keycode_SAPP_KEYCODE_HOME => Home,
            sys::sapp_keycode_SAPP_KEYCODE_END => End,
            sys::sapp_keycode_SAPP_KEYCODE_CAPS_LOCK => CapsLock,
            sys::sapp_keycode_SAPP_KEYCODE_SCROLL_LOCK => ScrollLock,
            sys::sapp_keycode_SAPP_KEYCODE_NUM_LOCK => NumLock,
            sys::sapp_keycode_SAPP_KEYCODE_PRINT_SCREEN => PrintScreen,
            sys::sapp_keycode_SAPP_KEYCODE_PAUSE => Pause,
            sys::sapp_keycode_SAPP_KEYCODE_F1 => F1,
            sys::sapp_keycode_SAPP_KEYCODE_F2 => F2,
            sys::sapp_keycode_SAPP_KEYCODE_F3 => F3,
            sys::sapp_keycode_SAPP_KEYCODE_F4 => F4,
            sys::sapp_keycode_SAPP_KEYCODE_F5 => F5,
            sys::sapp_keycode_SAPP_KEYCODE_F6 => F6,
            sys::sapp_keycode_SAPP_KEYCODE_F7 => F7,
            sys::sapp_keycode_SAPP_KEYCODE_F8 => F8,
            sys::sapp_keycode_SAPP_KEYCODE_F9 => F9,
            sys::sapp_keycode_SAPP_KEYCODE_F10 => F10,
            sys::sapp_keycode_SAPP_KEYCODE_F11 => F11,
            sys::sapp_keycode_SAPP_KEYCODE_F12 => F12,
            sys::sapp_keycode_SAPP_KEYCODE_F13 => F13,
            sys::sapp_keycode_SAPP_KEYCODE_F14 => F14,
            sys::sapp_keycode_SAPP_KEYCODE_F15 => F15,
            sys::sapp_keycode_SAPP_KEYCODE_F16 => F16,
            sys::sapp_keycode_SAPP_KEYCODE_F17 => F17,
            sys::sapp_keycode_SAPP_KEYCODE_F18 => F18,
            sys::sapp_keycode_SAPP_KEYCODE_F19 => F19,
            sys::sapp_keycode_SAPP_KEYCODE_F20 => F20,
            sys::sapp_keycode_SAPP_KEYCODE_F21 => F21,
            sys::sapp_keycode_SAPP_KEYCODE_F22 => F22,
            sys::sapp_keycode_SAPP_KEYCODE_F23 => F23,
            sys::sapp_keycode_SAPP_KEYCODE_F24 => F24,
            sys::sapp_keycode_SAPP_KEYCODE_F25 => F25,
            sys::sapp_keycode_SAPP_KEYCODE_KP_0 => KP0,
            sys::sapp_keycode_SAPP_KEYCODE_KP_1 => KP1,
            sys::sapp_keycode_SAPP_KEYCODE_KP_2 => KP2,
            sys::sapp_keycode_SAPP_KEYCODE_KP_3 => KP3,
            sys::sapp_keycode_SAPP_KEYCODE_KP_4 => KP4,
            sys::sapp_keycode_SAPP_KEYCODE_KP_5 => KP5,
            sys::sapp_keycode_SAPP_KEYCODE_KP_6 => KP6,
            sys::sapp_keycode_SAPP_KEYCODE_KP_7 => KP7,
            sys::sapp_keycode_SAPP_KEYCODE_KP_8 => KP8,
            sys::sapp_keycode_SAPP_KEYCODE_KP_9 => KP9,
            sys::sapp_keycode_SAPP_KEYCODE_KP_DECIMAL => KPDecimal,
            sys::sapp_keycode_SAPP_KEYCODE_KP_DIVIDE => KPDivide,
            sys::sapp_keycode_SAPP_KEYCODE_KP_MULTIPLY => KPMultiply,
            sys::sapp_keycode_SAPP_KEYCODE_KP_SUBTRACT => KPSubtract,
            sys::sapp_keycode_SAPP_KEYCODE_KP_ADD => KPAdd,
            sys::sapp_keycode_SAPP_KEYCODE_KP_ENTER => KPEnter,
            sys::sapp_keycode_SAPP_KEYCODE_KP_EQUAL => KPEqual,
            sys::sapp_keycode_SAPP_KEYCODE_LEFT_SHIFT => LeftShift,
            sys::sapp_keycode_SAPP_KEYCODE_LEFT_CONTROL => LeftControl,
            sys::sapp_keycode_SAPP_KEYCODE_LEFT_ALT => LeftAlt,
            sys::sapp_keycode_SAPP_KEYCODE_LEFT_SUPER => LeftSuper,
            sys::sapp_keycode_SAPP_KEYCODE_RIGHT_SHIFT => RightShift,
            sys::sapp_keycode_SAPP_KEYCODE_RIGHT_CONTROL => RightControl,
            sys::sapp_keycode_SAPP_KEYCODE_RIGHT_ALT => RightAlt,
            sys::sapp_keycode_SAPP_KEYCODE_RIGHT_SUPER => RightSuper,
            sys::sapp_keycode_SAPP_KEYCODE_MENU => Menu,
            _ => Invalid,
        }
    }
}