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


#[derive(PartialEq)]
pub
enum Key {
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,

    Unknown,
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
    RedimWindow,
    MouseMotion(MouseInfo),
}

#[derive(Clone, PartialEq)]
pub
struct MouseInfo {
    x:  i32,
    y:  i32,
    dx: i32,
    dy: i32,
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
                                    | xlib::PointerMotionMask;

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

        let to_c_string = 
        |str_: &str| -> *const i8 {
            CString::new(str_).unwrap().into_raw() as *const i8
        };

        /**/
        unsafe{xlib::XStoreName(display, window, to_c_string("draw"));}

        /* Defines the minimum and maximum dimensions of the window */
        {
            let mut hints = unsafe{MaybeUninit::<xlib::XSizeHints>::zeroed().assume_init()};

            if (min_width > 0) && (min_height > 0) { hints.flags |= xlib::PMinSize; }
            if (max_width > 0) && (max_height > 0) { hints.flags |= xlib::PMaxSize; }

            hints.min_width = min_width;
            hints.min_height = min_height;
            hints.max_width = max_width;
            hints.max_height = max_height;

            unsafe{xlib::XSetWMNormalHints(display, 
                                    window, 
                                    &mut hints as *mut _)};
        }
        
        /**/
        unsafe{xlib::XMapWindow(display, window);}

        // maximiza a janela
        /*
        let _ = {  
            let mut ev: XClientMessageEvent = MaybeUninit::<>::zeroed().assume_init();
            let  wmState: xlib::Atom = XInternAtom(display, to_c_string("_NET_WM_STATE"), 0);
            let  maxH: xlib::Atom  =  XInternAtom(display, to_c_string("_NET_WM_STATE_MAXIMIZED_HORZ"), 0);
            let  maxV: xlib::Atom  =  XInternAtom(display, to_c_string("_NET_WM_STATE_MAXIMIZED_VERT"), 0);

            if wmState == AllocNone  as u64{ 0}
            else {

            ev.type_ = ClientMessage;
            ev.format = 32;
            ev.window = window;
            ev.message_type = wmState;
            ev.data.as_longs_mut()[0] = 2 as i64; // _NET_WM_STATE_TOGGLE 2 according to spec; Not defined in my headers
            ev.data.as_longs_mut()[1] = maxH as i64;
            ev.data.as_longs_mut()[2] = maxV as i64;
            ev.data.as_longs_mut()[3] = 1 as i64;

            XSendEvent(display, 
                        XDefaultRootWindow(display), 
                        0,
                        SubstructureNotifyMask,
                        (&mut ev as *mut XClientMessageEvent).cast::<XEvent>())
            };
        };
        */

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

        Window {
            width:      width,
            min_width:  min_width as _,
            max_width:  max_width as _,

            height:     height,
            min_height: min_height as _,
            max_height: max_height as _,

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

            let kcode_left  = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Left.into()).into()};
            let kcode_right = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Right.into()).into()};
            let kcode_up    = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Up.into()).into()};
            let kcode_down  = unsafe{xlib::XKeysymToKeycode(self.x11.display, XK_Down.into()).into()};

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
                        events.push(Event::RedimWindow);

                    }

                },

                xlib::KeyPress => {
                    let e: xlib::XKeyEvent = From::from(ev);

                    let mut key_press = Key::Unknown;

                    if e.keycode == kcode_left  {key_press = Key::LeftArrow;}
                    if e.keycode == kcode_right {key_press = Key::RightArrow;}
                    if e.keycode == kcode_up    {key_press = Key::UpArrow;}
                    if e.keycode == kcode_down  {key_press = Key::DownArrow;}

                    events.push(Event::KeyPress(key_press));
                },

                xlib::KeyRelease => {
                    let e: xlib::XKeyEvent = From::from(ev);

                    let mut key_press = Key::Unknown;

                    if e.keycode == kcode_left  {key_press = Key::LeftArrow;}
                    if e.keycode == kcode_right {key_press = Key::RightArrow;}
                    if e.keycode == kcode_up    {key_press = Key::UpArrow;}
                    if e.keycode == kcode_down  {key_press = Key::DownArrow;}

                    events.push(Event::KeyRelease(key_press));
                },

                xlib::MotionNotify => {
                    let e: xlib::XMotionEvent = From::from(ev);

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

    //pub
    //fn get_pitch(&self) -> usize {self.width * self.pixel_bytes}

}
