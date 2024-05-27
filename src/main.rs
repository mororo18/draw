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

use std::time::Instant;

fn main() {

    let width = 800;
    let height = 600;
    let mut win =  Window::new(width, height);

    let mut window_open = true;

    let mut canva = Canva::new(width, height);


    let mut now = Instant::now();
    let frame_rate: f64 = 60.0;
    let dt_ms = 1000.0 / frame_rate;

    // CREATE SCENE = OBJECTS + CAMERA

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

        let a = Vec2::<f64>::new(0.0, 0.0);
        let b = Vec2::<f64>::new(750.0, 250.0);
        let c = Vec2::<f64>::new(400.0, 50.0);
        let center_d = Vec2::<f64>::new(380.0, 300.0);


        let elapsed = now.elapsed().as_millis() as f64;
        if elapsed > dt_ms {
            now = Instant::now();
            println!("{}", 1000.0 / elapsed);

            //canva.draw_triangle(a, b, c);
            canva.draw_quadrilat(b, center_d, c, a);

            win.write_frame_from_slice(canva.as_bytes_slice());
        }
        /*
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
        */


        //let b = Vec2::<f64>::new(0.0, 400.0);
        //println!("{:?}", a);
        //canva.draw_line(j, k);
        //canva.draw_line(i, j);
      //let c = Vec2::<f64>::new(0.0, 1.0);
      //let d = Vec2::<f64>::new(799.0, 199.0);
      //canva.draw_line(d, c);
    }

}
