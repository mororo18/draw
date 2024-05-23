mod draw;
mod window;

use window::{
    Window,
    Event,
};

use draw::Canva;

fn main() {

    let width = 800;
    let height = 600;
    let mut win =  Window::new(width, height);

    let mut window_open = true;

    let mut canva = Canva::new(width, height);

    while window_open {
        let events: Vec<Event> = win.handle();

        for e in events.iter() {
            if *e == Event::CloseWindow {
                window_open = false;
            }
        }

        //let ptr: *const u8 = canva.as_ptr();
        //let sz: usize = canva.size_bytes();
        //
        //let a = vec![0_u8; 800*600*4];
        win.write_frame_from_slice(canva.as_bytes_slice());

    }

    canva.size_bytes();
}
