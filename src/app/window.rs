// shared memory iamge -> https://www.x.org/releases/X11R7.7/doc/xextproto/shm.html
// https://handmade.network/forums/articles/t/2834-tutorial_a_tour_through_xlib_and_related_technologies
// events -> https://www.oreilly.com/library/view/xlib-reference-manual/9780937175262/13_appendix-e.html
// xlib c header -> https://codebrowser.dev/gtk/include/X11/X.h.html
// demowindow example -> https://docs.rs/x11/latest/src/input/input.rs.html#121
// input example -> https://who-t.blogspot.com/2009/05/xi2-recipes-part-1.html

use x11::xlib;
use x11::xinput2;
use x11::keysym::*;
use std::os::raw::*; 
use std::mem::MaybeUninit;
use std::alloc::{alloc_zeroed, Layout};

use std::ffi::CString;


/*
#[derive(PartialEq)]
pub
enum Key {
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,

    Unknown,
}
*/
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

    Sym((u32, u32)),
}

#[allow(non_upper_case_globals)]
impl Key {
    pub
    fn from_keysym(keysym: u32) -> Self {
        match keysym {
            XK_Tab => Key::Tab,
            XK_Left => Key::LeftArrow,
            XK_Right => Key::RightArrow,
            XK_Up => Key::UpArrow,
            XK_Down => Key::DownArrow,
            XK_Prior => Key::PageUp,
            XK_Next => Key::PageDown,
            XK_Home => Key::Home,
            XK_End => Key::End,
            XK_Insert => Key::Insert,
            XK_Delete => Key::Delete,
            XK_BackSpace => Key::Backspace,
            XK_space => Key::Space,
            XK_Return => Key::Enter,
            XK_Escape => Key::Escape,
            XK_quoteright => Key::Apostrophe,
            XK_comma => Key::Comma,
            XK_minus => Key::Minus,
            XK_period => Key::Period,
            XK_slash => Key::Slash,
            XK_semicolon => Key::Semicolon,
            XK_equal => Key::Equal,
            XK_bracketleft => Key::LeftBracket,
            XK_backslash => Key::Backslash,
            XK_bracketright => Key::RightBracket,
            XK_quoteleft => Key::GraveAccent,
            XK_Caps_Lock => Key::CapsLock,
            XK_Scroll_Lock => Key::ScrollLock,
            XK_Num_Lock => Key::NumLock,
            XK_Print => Key::PrintScreen,
            XK_Pause => Key::Pause,
            XK_KP_0 => Key::Keypad0,
            XK_KP_1 => Key::Keypad1,
            XK_KP_2 => Key::Keypad2,
            XK_KP_3 => Key::Keypad3,
            XK_KP_4 => Key::Keypad4,
            XK_KP_5 => Key::Keypad5,
            XK_KP_6 => Key::Keypad6,
            XK_KP_7 => Key::Keypad7,
            XK_KP_8 => Key::Keypad8,
            XK_KP_9 => Key::Keypad9,
            XK_KP_Decimal => Key::KeypadDecimal,
            XK_KP_Divide => Key::KeypadDivide,
            XK_KP_Multiply => Key::KeypadMultiply,
            XK_KP_Subtract => Key::KeypadSubtract,
            XK_KP_Add => Key::KeypadAdd,
            XK_KP_Enter => Key::KeypadEnter,
            XK_KP_Equal => Key::KeypadEqual,
            XK_Control_L => Key::LeftCtrl,
            XK_Shift_L => Key::LeftShift,
            XK_Alt_L => Key::LeftAlt,
            XK_Super_L => Key::LeftSuper,
            XK_Control_R => Key::RightCtrl,
            XK_Shift_R => Key::RightShift,
            XK_Alt_R => Key::RightAlt,
            XK_Super_R => Key::RightSuper,
            XK_Menu => Key::Menu,
            XK_0 => Key::Num0,
            XK_1 => Key::Num1,
            XK_2 => Key::Num2,
            XK_3 => Key::Num3,
            XK_4 => Key::Num4,
            XK_5 => Key::Num5,
            XK_6 => Key::Num6,
            XK_7 => Key::Num7,
            XK_8 => Key::Num8,
            XK_9 => Key::Num9,
            XK_a => Key::A,
            XK_b => Key::B,
            XK_c => Key::C,
            XK_d => Key::D,
            XK_e => Key::E,
            XK_f => Key::F,
            XK_g => Key::G,
            XK_h => Key::H,
            XK_i => Key::I,
            XK_j => Key::J,
            XK_k => Key::K,
            XK_l => Key::L,
            XK_m => Key::M,
            XK_n => Key::N,
            XK_o => Key::O,
            XK_p => Key::P,
            XK_q => Key::Q,
            XK_r => Key::R,
            XK_s => Key::S,
            XK_t => Key::T,
            XK_u => Key::U,
            XK_v => Key::V,
            XK_w => Key::W,
            XK_x => Key::X,
            XK_y => Key::Y,
            XK_z => Key::Z,
            XK_F1 => Key::F1,
            XK_F2 => Key::F2,
            XK_F3 => Key::F3,
            XK_F4 => Key::F4,
            XK_F5 => Key::F5,
            XK_F6 => Key::F6,
            XK_F7 => Key::F7,
            XK_F8 => Key::F8,
            XK_F9 => Key::F9,
            XK_F10 => Key::F10,
            XK_F11 => Key::F11,
            XK_F12 => Key::F12,
            XF86XK_Back => Key::AppBack,
            XF86XK_Forward => Key::AppForward,
            _ => Key::Unknown,
        }
    }
}



#[derive(PartialEq)]
pub
enum Button {
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

impl MouseCursor {
    fn as_c_str(self) -> *const i8 {

        match self {
            MouseCursor::Arrow      => c"arrow".as_ptr(),
            MouseCursor::TextInput  => c"xterm".as_ptr(),
            MouseCursor::ResizeAll  => c"fleur".as_ptr(),
            MouseCursor::ResizeNS   => c"sb_v_double_arrow".as_ptr(),
            MouseCursor::ResizeEW   => c"sb_h_double_arrow".as_ptr(),
            MouseCursor::ResizeNESW => c"bottom_left_corner".as_ptr(),
            MouseCursor::ResizeNWSE => c"bottom_right_corner".as_ptr(),
            MouseCursor::Hand       => c"hand1".as_ptr(),
            MouseCursor::NotAllowed => c"circle".as_ptr(),
        }

    }
}



#[derive(PartialEq)]
pub
enum Event {
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
pub
struct MouseInfo {
    pub x:  i32,
    pub y:  i32,
    pub dx: i32,
    pub dy: i32,
}

pub
struct X11Info {
    pixel_bits:  usize,
    pixel_bytes: usize,

    display: *mut xlib::Display,
    root:               c_long,
    screen:             c_int,
    screen_bit_depth:   c_int,
    visinfo:            xlib::XVisualInfo,

    window:             xlib::Window,
    window_attr:        xlib::XSetWindowAttributes,
    window_buffer:      *mut xlib::XImage,
    window_buffer_size: usize,
    mem:                *mut u8,

    default_gc:         xlib::GC,
    wm_delete_window:   xlib::Atom,
}

pub
struct Window {
    width:  usize,
    height: usize,
    min_width:  usize,
    min_height: usize,
    max_width:  usize,
    max_height: usize,

    pos_x: i32,
    pos_y: i32,

    x11:        X11Info,

    mouse_grabbed:      bool,
    mouse_info:     MouseInfo,
}

impl Window {
    pub 
    fn new (width: usize, height:usize) -> Self {

        assert!(
            env!("XDG_SESSION_TYPE") == "x11",
            "Wayland is not supported."
        );

        let min_width    = width as i32;
        let min_height   = height as i32;
        let max_width    = 0;
        let max_height   = 0;

        /* Abre display padrao */
        let display: *mut xlib::Display = unsafe{xlib::XOpenDisplay(std::ptr::null())};
        if display.is_null() {panic!("N'ao foi possivel abrir display");}

        /* Default root window and default screen */
        let root            : c_ulong   = unsafe{xlib::XDefaultRootWindow(display)};
        let default_screen  : c_int     = unsafe{xlib::XDefaultScreen(display)};

        /* Match Visual Info */
        let screen_bit_depth: c_int = 24;
        let mut visinfo  = unsafe{MaybeUninit::<xlib::XVisualInfo>::zeroed().assume_init()};
        let match_visual = unsafe{xlib::XMatchVisualInfo(display, 
                                                default_screen, 
                                                screen_bit_depth, 
                                                xlib::TrueColor, 
                                                &mut visinfo as *mut _)};
        if  match_visual == 0 {panic!("No matching visual info");}

        /* window attributes */
        let mut window_attr          = unsafe{MaybeUninit::<xlib::XSetWindowAttributes>::zeroed().assume_init()};
        window_attr.bit_gravity      = xlib::StaticGravity;
        window_attr.background_pixel = 0;
        window_attr.colormap         = unsafe{xlib::XCreateColormap(display, 
                                                            root,
                                                            visinfo.visual, 
                                                            xlib::AllocNone)};
        window_attr.event_mask       = xlib::StructureNotifyMask 
                                    | xlib::KeyPressMask 
                                    | xlib::KeyReleaseMask
                                    | xlib::PointerMotionMask
                                    | xlib::ButtonPressMask
                                    | xlib::ButtonReleaseMask;

        /* tells the what attributes we are using */
        let attribute_mask           = xlib::CWBitGravity 
                                    | xlib::CWBackPixel 
                                    | xlib::CWColormap 
                                    | xlib::CWEventMask;

        /* Create the window */
        let  window: xlib::Window   = unsafe{xlib::XCreateWindow(display, root,
                                                        0, 0,
                                                        width as _, 
                                                        height as _, 
                                                        0,
                                                        visinfo.depth, 
                                                        xlib::InputOutput as u32,
                                                        visinfo.visual, 
                                                        attribute_mask, 
                                                        &mut window_attr as *mut _)};

        if window == 0 {panic!("Window wasn't created properly");}

        let to_c_string_mut = 
        |str_: &str| -> *mut i8 {
            CString::new(str_).unwrap().into_raw() as *mut i8
        };

        let to_c_string = 
        |str_: &str| -> *const i8 {
            CString::new(str_).unwrap().into_raw() as *const i8
        };

        // Set window name
        unsafe{xlib::XStoreName(display, window, to_c_string("draw"));}

        // Set icon name
        let class_hint = unsafe { xlib::XAllocClassHint() };
        if !class_hint.is_null() {
            unsafe {
                (*class_hint).res_name = to_c_string_mut("draw");
                (*class_hint).res_class = to_c_string_mut("draw");

                xlib::XSetClassHint(display, window, class_hint);
                xlib::XFree(class_hint as _);
            }
        }

        // TODO: Set icon 
        // https://stackoverflow.com/questions/10699927/xlib-argb-window-icon

        /* Defines the minimum and maximum dimensions of the window */
        {
            let mut hints = unsafe{MaybeUninit::<xlib::XSizeHints>::zeroed().assume_init()};

            if (min_width > 0) && (min_height > 0) { hints.flags |= xlib::PMinSize; }
            if (max_width > 0) && (max_height > 0) { hints.flags |= xlib::PMaxSize; }

            hints.min_width  = min_width;
            hints.min_height = min_height;
            hints.max_width  = max_width;
            hints.max_height = max_height;

            unsafe{xlib::XSetWMNormalHints(display, 
                                    window, 
                                    &mut hints as *mut _)};
        }

        


        
        /**/
        unsafe{xlib::XMapWindow(display, window);}


        // https://github.com/glfw/glfw/blob/master/src/x11_window.c#L498
        // Xinput Events 

        unsafe {
            let mut event_mask: xinput2::XIEventMask = MaybeUninit::<_>::zeroed().assume_init();

            let mut mask = vec![0_u8; xinput2::XIMaskLen(xinput2::XI_RawMotion)];

            event_mask.deviceid = xinput2::XIAllMasterDevices;
            event_mask.mask_len = mask.len() as _;
            event_mask.mask = mask.as_mut_slice().as_mut_ptr();
            xinput2::XISetMask(&mut mask, xinput2::XI_RawMotion);

            xinput2::XISelectEvents(display, root, &mut event_mask as *mut _, 1);

        }

        /**/
        unsafe{xlib::XFlush(display);}



        /* allocates memory and creates the window buffer */
        let pixel_bits = 32_i32;
        let pixel_bytes = pixel_bits / 8;
        let window_buffer_size = ((width * height) as u32) * (pixel_bytes as u32);

        let layout = Layout::array::<i8>(window_buffer_size as usize)
                                        .expect("layout deu merda");
        let mem: *mut u8  = unsafe{alloc_zeroed(layout)};

        let window_buffer: *mut xlib::XImage = unsafe{xlib::XCreateImage(display,  
                                                                    visinfo.visual, 
                                                                    visinfo.depth as u32,
                                                                    xlib::ZPixmap,
                                                                    0, 
                                                                    mem as *mut _, 
                                                                    width as _, 
                                                                    height as _,
                                                                    pixel_bits, 0)};
        // graphics context
        let default_gc: xlib::GC = unsafe{xlib::XDefaultGC(display, default_screen)};

        // special way for the window manager to tell you that the window close button was
        // pressed without actually closing the window itself. 
        let mut wm_delete_window: xlib::Atom = 
                                  unsafe{xlib::XInternAtom(display, to_c_string("WM_DELETE_WINDOW"), 0)};
        let could_set_prot = unsafe{xlib::XSetWMProtocols(display, 
                                                    window, 
                                                    &mut wm_delete_window as *mut _, 1)};
        if  could_set_prot == 0 {panic!("Couldn't register WM_DELETE_WINDOW property");}

        // get window pos
        let win_attr = unsafe {
            let mut xwa = MaybeUninit::<xlib::XWindowAttributes>::zeroed().assume_init();
            xlib::XGetWindowAttributes(display, window, &mut xwa as *mut _);
            xwa
        };

        let pos_x = win_attr.x;
        let pos_y = win_attr.y;

        Window {
            width:      width,
            min_width:  min_width as _,
            max_width:  max_width as _,

            height:     height,
            min_height: min_height as _,
            max_height: max_height as _,

            pos_x,
            pos_y,

            x11: X11Info {
                pixel_bits: pixel_bits as _,
                pixel_bytes: pixel_bytes as _,

                display:            display,
                root:               root as _,
                screen:             default_screen,
                screen_bit_depth:   screen_bit_depth,
                visinfo:            visinfo,

                window:             window,
                window_attr:        window_attr,
                window_buffer:      window_buffer,
                window_buffer_size: window_buffer_size as _,
                mem:                mem,

                default_gc:         default_gc,
                wm_delete_window:   wm_delete_window,
            },

            mouse_grabbed:      false,
            mouse_info:     MouseInfo {x: 0, y: 0, dx: 0, dy: 0},
        }


    }

    // criar funcao "handle" que retorna um enum, ou vec de enums, contendo os eventos recebidos
    //
    pub
    fn handle(&mut self) -> Vec<Event> {

        unsafe{xlib::XPutImage(self.x11.display, 
            self.x11.window,
            self.x11.default_gc, 
            self.x11.window_buffer, 0, 0, 0, 0,
            self.width as _, 
            self.height as _)};

        let mut ev = unsafe{MaybeUninit::<xlib::XEvent >::zeroed().assume_init()};

        let mut size_change = false;
        let mut events: Vec<Event> = Vec::new();

        while unsafe{xlib::XPending(self.x11.display)} > 0 {
            unsafe{xlib::XNextEvent(self.x11.display, &mut ev);}

            /*
            let kcode_left  = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Left.into()).into()};
            let kcode_right = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Right.into()).into()};
            let kcode_up    = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Up.into()).into()};
            let kcode_down  = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Down.into()).into()};
            */

            match ev.get_type() {

                xlib::GenericEvent => {
                    let mut cookie: xlib::XGenericEventCookie = From::from(ev);
                    if unsafe { xlib::XGetEventData(self.x11.display, &mut cookie) } != xlib::True {
                        println!("Failed to retrieve event data");
                        continue;
                    }

                    match cookie.evtype {
                        // exemplos
                        // https://github.com/comex/Dolphin-work/blob/master/Source/Core/InputCommon/ControllerInterface/Xlib/XInput2.cpp#L257
                        // https://docs.rs/x11/latest/src/input/input.rs.html#386
                        xinput2::XI_RawMotion => {
                            //println!("RawMotion");
                            let raw_ev: &xinput2::XIRawEvent = unsafe { std::mem::transmute(cookie.data) };

                            let mut delta_x = 0.0;
                            let mut delta_y = 0.0;

                            let mask =
                                unsafe { std::slice::from_raw_parts(raw_ev.valuators.mask, raw_ev.valuators.mask_len as usize) };
                            if xinput2::XIMaskIsSet(&mask, 0) {
                                let delta_delta = unsafe {*raw_ev.raw_values.offset(0)};
                                // test for inf and nan
                                if delta_delta == delta_delta && 1.0+delta_delta != delta_delta {
                                    delta_x += delta_delta;
                                }
                            }

                            if xinput2::XIMaskIsSet(mask, 1) {
                                let delta_delta = unsafe { *raw_ev.raw_values.offset(1) };
                                // test for inf and nan
                                if delta_delta == delta_delta && 1.0+delta_delta != delta_delta {
                                    delta_y += delta_delta;
                                }
                            }

                            //println!("raw delta ({delta_x}, {delta_y})");

                            let mouse_info = MouseInfo {
                                x: self.mouse_info.x, 
                                y: self.mouse_info.y,
                                dx: delta_x as i32,
                                dy: delta_y as i32,
                            };

                            self.mouse_info = mouse_info.clone();

                            events.push(Event::MouseMotion(mouse_info));

                        },

                        _ => println!("Unknown xinput evet {}", cookie.evtype),
                    }
                },

                xlib::ButtonPress => {
                    let e: xlib::XButtonEvent = From::from(ev);
                    println!("Press");

                    let button_event = 
                    match e.button {
                        xlib::Button1 => {println!(" Mouse Left "); Event::ButtonPress(Button::MouseLeft)},
                        xlib::Button2 => {println!(" Wheel Click ");Event::ButtonPress(Button::MouseMiddle)},
                        xlib::Button3 => {println!(" Mouse Right ");Event::ButtonPress(Button::MouseRight)},
                        xlib::Button4 => {println!(" Wheel Up ");   Event::ButtonPress(Button::WheelUp)},
                        xlib::Button5 => {println!(" Wheel Down "); Event::ButtonPress(Button::WheelDown)},
                        _ => Event::Empty,
                    };

                    events.push(button_event);

                },

                xlib::ButtonRelease => {
                    let e: xlib::XButtonEvent = From::from(ev);
                    println!("Release");

                    let button_event = 
                    match e.button {
                        xlib::Button1 => {println!(" Mouse Left "); Event::ButtonRelease(Button::MouseLeft)},
                        xlib::Button2 => {println!(" Wheel Click ");Event::ButtonRelease(Button::MouseMiddle)},
                        xlib::Button3 => {println!(" Mouse Right ");Event::ButtonRelease(Button::MouseRight)},
                        xlib::Button4 => {println!(" Wheel Up ");   Event::ButtonRelease(Button::WheelUp)},
                        xlib::Button5 => {println!(" Wheel Down "); Event::ButtonRelease(Button::WheelDown)},
                        _ => Event::Empty,
                    };

                    events.push(button_event);

                },

                xlib::ClientMessage => {
                    let e: xlib::XClientMessageEvent = From::from(ev);

                    if e.data.get_long(0) as xlib::Atom == self.x11.wm_delete_window {
                        unsafe{xlib::XDestroyWindow(self.x11.display, self.x11.window);}
                    }

                    events.push(Event::CloseWindow);
                },

                xlib::ConfigureNotify => {
                    let e: xlib::XConfigureEvent = From::from(ev);

                    if self.width != e.width as _ ||
                        self.height != e.height as _ 
                    {
                        self.width = e.width as _;
                        self.height = e.height as _;

                        size_change = true;
                        events.push(Event::RedimWindow((self.width, self.height)));
                    }

                    if self.pos_x != e.x ||
                        self.pos_y != e.y 
                    {
                        self.pos_x = e.x;
                        self.pos_y = e.y;

                        events.push(Event::ReposWindow((e.x, e.y)));
                    }

                },

                xlib::KeyPress => {
                    let mut e: xlib::XKeyEvent = From::from(ev);

                    let keysym = unsafe { xlib::XLookupKeysym( &mut e as *mut _, 0 ) as u32 };

                    match Key::from_keysym(keysym) {
                        Key::F11 => { self.toggle_fullscreen() },
                        _ => {},
                    };

                    events.push(Event::KeyPress(Key::from_keysym(keysym)));
                },

                xlib::KeyRelease => {
                    let mut e: xlib::XKeyEvent = From::from(ev);

                    let keysym = unsafe { xlib::XLookupKeysym( &mut e as *mut _, 0 ) as u32 };

                    events.push(Event::KeyRelease(Key::from_keysym(keysym)));
                },

                xlib::MotionNotify => {
                    let e: xlib::XPointerMovedEvent = From::from(ev);

                    //println!("motion Notify");
                    // https://gitlab.winehq.org/wine/wine/-/blob/master/dlls/winex11.drv/mouse.c#L1405
                    // https://github.com/blender/blender/blob/b04c0da6f04cbd3f38c0d8a5fd137375209a1fc1/intern/ghost/intern/GHOST_SystemX11.cc#L1756
                    // https://github.com/glfw/glfw/blob/master/src/x11_window.c#L2851
                    //
                    //
                    // libxi-dev
                    // libxfixes-dev
                    //
                    //
                    // desabilitar wayland
                    // https://github.com/debauchee/barrier/issues/1659
                    //
                    // outra discussao interessante sobre wayland (GDK_BACKEND=x11) (echo $XDG_SESSION_TYPE)
                    // https://forums.thedarkmod.com/index.php?/topic/21691-incorrect-mouse-movement-in-3d-2d-views-on-plasma-wayland/page/2/

                    if self.mouse_info.x != e.x ||
                        self.mouse_info.y != e.y
                    {

                        let mouse_info = MouseInfo {
                            x: e.x, 
                            y: e.y,
                            dx: self.mouse_info.dx,
                            dy: self.mouse_info.dy,
                        };

                        self.mouse_info = mouse_info.clone();

                        events.push(Event::MouseMotion(mouse_info));
                    }
                    
                },

                xlib::ReparentNotify => println!("ReparentNotify"),
                xlib::MapNotify => println!("MapNotify"),

                _ => println!("Unknown notify {}", ev.get_type()),
            }

        }

        if size_change {

            unsafe { xlib::XDestroyImage(self.x11.window_buffer) }; // Free's the memory we malloced;

            ////loop {}

            println!("{} x {}", self.width, self.height);
            self.x11.window_buffer_size = (self.width * self.height * self.x11.pixel_bytes) as usize;
            let layout = Layout::array::<i8>( self.x11.window_buffer_size ).expect("layout deu merda");
            self.x11.mem = unsafe { alloc_zeroed(layout) };

            self.x11.window_buffer = unsafe {
                xlib::XCreateImage(self.x11.display, 
                    self.x11.visinfo.visual, 
                    self.x11.visinfo.depth as u32,
                    xlib::ZPixmap, 
                    0, 
                    self.x11.mem as *mut _, 
                    self.width as _, 
                    self.height as _,
                    self.x11.pixel_bits as _, 
                    0)
            };
        }

        return events;
    }

    pub
    fn hide_mouse_cursor(&mut self) {
        unsafe { x11::xfixes::XFixesHideCursor(self.x11.display, self.x11.window) };
        unsafe { xlib::XFlush(self.x11.display); }
    }

    pub
    fn show_mouse_cursor(&mut self) {
        unsafe { x11::xfixes::XFixesShowCursor(self.x11.display, self.x11.window) };
        unsafe { xlib::XFlush(self.x11.display); }
    }

    pub
    fn toggle_fullscreen(&mut self) {
        unsafe {  
            let mut ev: xlib::XClientMessageEvent = MaybeUninit::<>::zeroed().assume_init();
            let wm_state: xlib::Atom = xlib::XInternAtom(
                self.x11.display,
                CString::new("_NET_WM_STATE").unwrap().into_raw() as _,
                0
            );

            let fullscreen: xlib::Atom = xlib::XInternAtom(
                self.x11.display,
                CString::new("_NET_WM_STATE_FULLSCREEN").unwrap().into_raw(),
                0
            );

            if wm_state != xlib::AllocNone as u64 {
                ev.type_ = xlib::ClientMessage;
                ev.format = 32;
                ev.window = self.x11.window;
                ev.message_type = wm_state;
                ev.data.as_longs_mut()[0] = 2 as i64; // _NET_WM_STATE_TOGGLE 2 according to spec
                ev.data.as_longs_mut()[1] = fullscreen as i64;
                ev.data.as_longs_mut()[2] = 0;
                ev.data.as_longs_mut()[3] = 1 as i64;

                let _ = xlib::XSendEvent(
                    self.x11.display, 
                    xlib::XDefaultRootWindow(self.x11.display), 
                    0,
                    xlib::SubstructureNotifyMask,
                    (&mut ev as *mut xlib::XClientMessageEvent).cast::<xlib::XEvent>()
                );
            }
        }
    }

    pub
    fn update_mouse_cursor(&mut self, cursor: MouseCursor) {
        let xlib_cursor: xlib::Cursor = unsafe { x11::xcursor::XcursorLibraryLoadCursor(self.x11.display, cursor.as_c_str()) };
        unsafe { xlib::XDefineCursor (self.x11.display, self.x11.window, xlib_cursor); }
        unsafe { xlib::XFlush(self.x11.display); }
    }


    pub
    fn set_mouse_position(&mut self, x: i32, y: i32) {
        assert!(0 <= x && x < self.width  as i32);
        assert!(0 <= y && y < self.height as i32);

        unsafe {
            xlib::XWarpPointer(
                self.x11.display, 
                self.x11.window, 
                self.x11.window, 
                0, 0, 
                0, 0, 
                x, 
                y
            );

            xlib::XFlush(self.x11.display);
        }

        // emular mouse warp no wayland
        // https://github.com/libsdl-org/SDL/commit/ad29875ee692deb9a3517f4d470bde4a83ff76ad
        // https://github.com/libsdl-org/SDL/commit/3a6d9c59f45a48d8d5a07e6f9428d45aa2069387
        // https://github.com/libsdl-org/SDL/issues/9793
        //
        // gambiarra Xwayland
        // https://github.com/libsdl-org/SDL/pull/9549
        // https://projects.blender.org/blender/blender/issues/53004#issuecomment-551561

    }

    pub 
    fn write_frame_from_ptr(&mut self, src: *const u8, sz: usize) {

        let mem_len = self.width * self.height * self.x11.pixel_bytes;
        if src.is_null() || sz > mem_len {panic!("frame overflow");}

        unsafe {
            self.x11.mem.copy_from_nonoverlapping(src, sz);
        }
    }

    pub 
    fn write_frame_from_slice(&mut self, src: &[u8]) {
        self.write_frame_from_ptr(src.as_ptr() as *const _, src.len());
    }

    pub
    fn get_window_position(&self) -> (i32, i32) {
        (self.pos_x, self.pos_y)
    }

    pub
    fn get_window_dim(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub
    fn get_screen_dim(&self) -> (usize, usize) {
        unsafe {
            let screen = xlib::XDefaultScreenOfDisplay(self.x11.display);
            let width:  usize = xlib::XWidthOfScreen(screen).try_into().unwrap();
            let height: usize = xlib::XHeightOfScreen(screen).try_into().unwrap();

            println!("{width} x {height}");
            (width, height)
        }
    }

    //pub
    //fn get_pitch(&self) -> usize {self.width * self.pixel_bytes}

}
