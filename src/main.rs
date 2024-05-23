mod draw;
mod window;

use window::{
    Window,
    Event,
};

fn main() {
    let mut win =  Window::new(800, 600);

    let mut window_open = true;

    while window_open {
        let events: Vec<Event> = win.handle();

        for e in events.iter() {
            if *e == Event::CloseWindow {
                window_open = false;
            }
        }

    }
}
