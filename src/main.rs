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

        let y_top = 400;
        let y_bot = 200;

        let x_right = 500;
        let x_left = 300;

        let center_a = Vec2::<f64>::new(400.0, 320.0);
        let center_b = Vec2::<f64>::new(420.0, 300.0);
        let center_c = Vec2::<f64>::new(400.0, 280.0);
        let center_d = Vec2::<f64>::new(380.0, 300.0);

        for p in x_left..=x_right {
            let side = Vec2::<f64>::new(p as f64, y_top as f64);
            canva.draw_line(center_a, side);
        }

        for p in (y_bot..=y_top).rev() {
            let side = Vec2::<f64>::new(x_right as f64, p as f64);
            canva.draw_line(center_b, side);
        }

        for p in (x_left..=x_right).rev() {
            let side = Vec2::<f64>::new(p as f64, y_bot as f64);
            canva.draw_line(center_c, side);
        }

        for p in y_bot..=y_top {
            let side = Vec2::<f64>::new(x_left as f64, p as f64);
            canva.draw_line(center_d, side);
        }


        //let b = Vec2::<f64>::new(0.0, 400.0);
        //println!("{:?}", a);
        //canva.draw_line(j, k);
        //canva.draw_line(i, j);
      //let c = Vec2::<f64>::new(0.0, 1.0);
      //let d = Vec2::<f64>::new(799.0, 199.0);
      //canva.draw_line(d, c);

        win.write_frame_from_slice(canva.as_bytes_slice());
    }

}
