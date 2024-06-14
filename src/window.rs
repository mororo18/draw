// shared memory iamge -> https://www.x.org/releases/X11R7.7/doc/xextproto/shm.html
// https://handmade.network/forums/articles/t/2834-tutorial_a_tour_through_xlib_and_related_technologies
// events -> https://www.oreilly.com/library/view/xlib-reference-manual/9780937175262/13_appendix-e.html
// xlib c header -> https://codebrowser.dev/gtk/include/X11/X.h.html
// demowindow example -> https://docs.rs/x11/latest/src/input/input.rs.html#121
// input example -> https://who-t.blogspot.com/2009/05/xi2-recipes-part-1.html

use x11::xlib;
use x11::keysym::*;
use std::os::raw::*; 
use std::ptr;
use std::mem::MaybeUninit;
use std::alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error, Layout};

use std::ffi::CString;

use std::result::*;

use std::time::UNIX_EPOCH;
use std::time::SystemTime;

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
pub
enum Event {
    CloseWindow,
    KeyPress(Key),
    KeyRelease(Key),
    RedimWindow,
}

pub
struct Window {
    width:  usize,
    height: usize,
    min_width:  usize,
    min_height: usize,
    max_width:  usize,
    max_height: usize,

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

impl Window {
    pub 
    fn new (width: usize, height:usize) -> Self {

        //let min_width    = 400;
        //let min_height   = 300;
        let min_width    = width as i32;
        let min_height   = height as i32;
        let max_width    = 0;
        let max_height   = 0;

        /* Abre display padrao */
        let mut display: *mut xlib::Display = unsafe{xlib::XOpenDisplay(std::ptr::null())};
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
                                    | xlib::KeyReleaseMask;

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
        unsafe{xlib::XStoreName(display, window, to_c_string("Hello, World!"));}

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

        /**/
        unsafe{xlib::XFlush(display);}



        /* allocates memory and creates the window buffer */
        let pixel_bits = 32_i32;
        let pixel_bytes = pixel_bits / 8;
        let mut window_buffer_size = ((width * height) as u32) * (pixel_bytes as u32);

        let layout = Layout::array::<i8>(window_buffer_size as usize)
                                        .expect("layout deu merda");
        let mut mem: *mut u8  = unsafe{alloc(layout)};

        let mut window_buffer: *mut xlib::XImage = unsafe{xlib::XCreateImage(display,  
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
        let mut WM_DELETE_WINDOW: xlib::Atom = 
                                  unsafe{xlib::XInternAtom(display, to_c_string("WM_DELETE_WINDOW"), 0)};
        let could_set_prot = unsafe{xlib::XSetWMProtocols(display, 
                                                    window, 
                                                    &mut WM_DELETE_WINDOW as *mut _, 1)};
        if  could_set_prot == 0 {panic!("Couldn't register WM_DELETE_WINDOW property");}

        Window {
            width:      width,
            min_width:  min_width as _,
            max_width:  max_width as _,

            height:     height,
            min_height: min_height as _,
            max_height: max_height as _,

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
            wm_delete_window:   WM_DELETE_WINDOW,
        }


    }

    // criar funcao "handle" que retorna um enum, ou vec de enums, contendo os eventos recebidos
    //
    pub
    fn handle(&mut self) -> Vec<Event> {
        unsafe{xlib::XPutImage(self.display, 
            self.window,
            self.default_gc, 
            self.window_buffer, 0, 0, 0, 0,
            self.width as _, 
            self.height as _)};

        let mut ev = unsafe{MaybeUninit::<xlib::XEvent >::zeroed().assume_init()};

        fn ptr_cast<T, U>(ev: &mut U) -> *mut T {
            (ev as *mut U).cast::<T>()
        }

        let mut size_change = false;
        let mut events: Vec<Event> = Vec::new();

        while unsafe{xlib::XPending(self.display)} > 0 {
            unsafe{xlib::XNextEvent(self.display, &mut ev as *mut _);}

            let kcode_left  = unsafe{xlib::XKeysymToKeycode(self.display, XK_Left.into()).into()};
            let kcode_right = unsafe{xlib::XKeysymToKeycode(self.display, XK_Right.into()).into()};
            let kcode_up    = unsafe{xlib::XKeysymToKeycode(self.display, XK_Up.into()).into()};
            let kcode_down  = unsafe{xlib::XKeysymToKeycode(self.display, XK_Down.into()).into()};

            match unsafe {ev.type_} {
                /*
                   DestroyNotify => {
                   println!("DestroyNotify");
                   let e: *mut XDestroyWindowEvent =  (&mut ev as *mut XEvent).cast::<XDestroyWindowEvent>() ;
                   if (*e).window == window {
                   window_open = false;
                   }

                   break;
                   },
                   */

                xlib::ClientMessage => {
                    let e = ptr_cast::<xlib::XClientMessageEvent, _>(&mut ev);
                    //let e: *mut xlib::XClientMessageEvent = (&mut ev as *mut xlib::XEvent).cast::<xlib::XClientMessageEvent>();
                    unsafe {
                        if (*e).data.as_longs()[0] as xlib::Atom == self.wm_delete_window {
                            unsafe{xlib::XDestroyWindow(self.display, self.window);}
                        }
                    }

                    //break;
                    events.push(Event::CloseWindow);
                },

                xlib::ConfigureNotify => {
                    let e = ptr_cast::<xlib::XConfigureEvent, _>(&mut ev);
                    //let e: *mut xlib::XConfigureEvent = (&mut ev as *mut xlib::XEvent).cast::<xlib::XConfigureEvent>();

                    unsafe {
                        if self.width != (*e).width as _ ||
                            self.height != (*e).height as _ 
                        {
                            self.width = (*e).width as _;
                            self.height = (*e).height as _;

                            size_change = true;
                            events.push(Event::RedimWindow);

                        }
                    }

                },

                xlib::KeyPress => {
                    //let e: *mut xlib::XKeyEvent = (&mut ev as *mut xlib::XEvent).cast::<xlib::XKeyEvent>();
                    let e = ptr_cast::<xlib::XKeyEvent, _>(&mut ev);

                    let mut key_press = Key::Unknown;
                    unsafe {
                        if (*e).keycode == kcode_left   {key_press = Key::LeftArrow;}
                        if (*e).keycode == kcode_right   {key_press = Key::RightArrow;}
                        if (*e).keycode == kcode_up   {key_press = Key::UpArrow;}
                        if (*e).keycode == kcode_down   {key_press = Key::DownArrow;}
                    }

                    events.push(Event::KeyPress(key_press));
                },

                xlib::KeyRelease => {
                    //let e: *mut xlib::XKeyEvent = (&mut ev as *mut xlib::XEvent).cast::<xlib::XKeyEvent>();
                    let e = ptr_cast::<xlib::XKeyEvent, _>(&mut ev);


                    let mut key_press = Key::Unknown;
                    unsafe {
                        if (*e).keycode == kcode_left   {key_press = Key::LeftArrow;}
                        if (*e).keycode == kcode_right   {key_press = Key::RightArrow;}
                        if (*e).keycode == kcode_up   {key_press = Key::UpArrow;}
                        if (*e).keycode == kcode_down   {key_press = Key::DownArrow;}
                    }

                    events.push(Event::KeyRelease(key_press));
                },

                xlib::ReparentNotify => println!("ReparentNotify"),
                xlib::ConfigureNotify => println!("ConfigureNotify"),
                xlib::MapNotify => println!("MapNotify"),

                _ => println!("Unknown notify {}", unsafe{ev.type_}),
            }

        }

        if size_change {

            size_change = false;
            unsafe{xlib::XDestroyImage(self.window_buffer)}; // Free's the memory we malloced;

            ////loop {}

            println!("{} x {}", self.width, self.height);
            self.window_buffer_size = self.width * self.height * self.pixel_bytes;
            let layout = Layout::array::<i8>(self.window_buffer_size as usize).expect("layout deu merda");
            self.mem = unsafe{alloc_zeroed(layout)};

            self.window_buffer = unsafe{xlib::XCreateImage(self.display, 
                self.visinfo.visual, 
                self.visinfo.depth as u32,
                xlib::ZPixmap, 
                0, 
                self.mem as *mut _, 
                self.width as _, 
                self.height as _,
                self.pixel_bits as _, 
                0)};
        }

        return events;
    }

    pub 
    fn write_frame_from_ptr(&mut self, src: *const u8, sz: usize) {

        let mem_len = self.width * self.height * self.pixel_bytes;
        if src.is_null() || sz > mem_len {panic!("frame overflow");}

        unsafe {
        self.mem.copy_from_nonoverlapping(src, sz);
        }
    }

    pub 
    fn write_frame_from_slice(&mut self, src: &[u8]) {
        self.write_frame_from_ptr(src.as_ptr() as *const _, src.len());
    }

    pub
    fn get_pitch(&self) -> usize {self.width * self.pixel_bytes}

}
