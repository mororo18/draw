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


        canva.draw_pixel(300, 400);

        win.write_frame_from_slice(canva.as_bytes_slice());
    }

}
