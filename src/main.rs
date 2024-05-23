mod draw;
mod window;

use window::{
    Window,
    Event,
};

use draw::{
    Canva,
    Vec2,
};

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

        let a = Vec2::<f64>::new(200.35, 200.46);
        let b = Vec2::<f64>::new(700.35, 250.46);

        println!("{:?}", a);
        canva.draw_line(a, b);

        win.write_frame_from_slice(canva.as_bytes_slice());
    }

}
