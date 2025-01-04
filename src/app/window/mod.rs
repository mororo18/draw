#[cfg(wayland_impl)]
mod wayland_impl;
#[cfg(x11_impl)]
mod x11_impl;

#[cfg(wayland_impl)]
pub use wayland_impl::WaylandWindow;
#[cfg(x11_impl)]
pub use x11_impl::X11Window;

#[cfg(x11_impl)]
pub type AppWindow = X11Window;
#[cfg(wayland_impl)]
pub type AppWindow = WaylandWindow;

// TODO: rename this trait to leave 'Window' to the actual 'AppWindow' type
pub trait Window {
    fn new(width: usize, height: usize) -> Self;
    fn handle(&mut self) -> Vec<super::Event>;
    fn hide_mouse_cursor(&mut self);
    fn show_mouse_cursor(&mut self);
    fn toggle_fullscreen(&mut self);
    fn update_mouse_cursor(&mut self, cursor: MouseCursor);
    fn set_mouse_position(&mut self, x: i32, y: i32);
    fn write_frame_from_ptr(&mut self, src: *const u8, sz: usize);
    fn write_frame_from_slice(&mut self, src: &[u8]);
    fn get_window_position(&self) -> (i32, i32);
    fn get_screen_dim(&self) -> (usize, usize);
    fn get_window_dim(&self) -> (usize, usize);
}

#[derive(PartialEq)]
pub enum Key {
    Tab,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    PageUp,
    PageDown,
    Home,
    End,
    Insert,
    Delete,
    Backspace,
    Space,
    Enter,
    Escape,
    LeftCtrl,
    LeftShift,
    LeftAlt,
    LeftSuper,
    RightCtrl,
    RightShift,
    RightAlt,
    RightSuper,
    Menu,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Semicolon,
    Equal,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Keypad0,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad4,
    Keypad5,
    Keypad6,
    Keypad7,
    Keypad8,
    Keypad9,
    KeypadDecimal,
    KeypadDivide,
    KeypadMultiply,
    KeypadSubtract,
    KeypadAdd,
    KeypadEnter,
    KeypadEqual,
    AppBack,
    AppForward,
    Unknown,
    //Sym((u32, u32)),
}

#[derive(PartialEq)]
pub enum Button {
    MouseLeft,
    MouseRight,
    MouseMiddle,
    WheelUp,
    WheelDown,
}

#[derive(PartialEq)]
pub enum MouseCursor {
    Arrow,
    TextInput,
    ResizeAll,
    ResizeNS,
    ResizeEW,
    ResizeNESW,
    ResizeNWSE,
    Hand,
    NotAllowed,
}

// TODO: move this to x11_impl.rs
impl MouseCursor {
    fn as_c_str(self) -> *const i8 {
        match self {
            MouseCursor::Arrow => c"default".as_ptr(),
            MouseCursor::TextInput => c"xterm".as_ptr(),
            MouseCursor::ResizeAll => c"fleur".as_ptr(),
            MouseCursor::ResizeNS => c"sb_v_double_arrow".as_ptr(),
            MouseCursor::ResizeEW => c"sb_h_double_arrow".as_ptr(),
            MouseCursor::ResizeNESW => c"bottom_left_corner".as_ptr(),
            MouseCursor::ResizeNWSE => c"bottom_right_corner".as_ptr(),
            MouseCursor::Hand => c"hand1".as_ptr(),
            MouseCursor::NotAllowed => c"circle".as_ptr(),
        }
    }
}

#[derive(PartialEq)]
pub enum Event {
    CloseWindow,
    KeyPress(Key),
    KeyRelease(Key),

    ButtonPress(Button),
    ButtonRelease(Button),

    RedimWindow((usize, usize)),
    ReposWindow((i32, i32)),
    MouseMotion(MouseInfo),

    Empty,
}

#[derive(Clone, PartialEq)]
pub struct MouseInfo {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
}
