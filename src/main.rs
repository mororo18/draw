mod draw;
mod window;

use window::{
    Window,
    Event,
    Key,
};

use draw::scene::{
    Scene,
    Object,
};

use std::time::Instant;

fn main() {

    let width = 800;
    let height = 600;
    let mut win =  Window::new(width, height);

    let mut window_open = true;

    //let mut canva = Canva::new(width, height);
    let mut scene = Scene::new(width, height);


    let mut now = Instant::now();
    //let frame_rate: f32 = 1.0;
    let frame_rate: f32 = 60.0;
    let dt_ms = 1000.0 / frame_rate;

    // CREATE SCENE = OBJECTS + CAMERA

    while window_open {
        let events: Vec<Event> = win.handle();

        for e in events.iter() {

            match e {
                Event::CloseWindow => {
                    window_open = false;
                },

                Event::KeyPress(key) => {
                    print!("KeyPress ");
                    match key {
                        Key::UpArrow => {
                            println!("Up");
                            scene.camera_up();
                        },
                        Key::DownArrow => {
                            println!("Down");
                            scene.camera_down();
                        },
                        Key::LeftArrow => {
                            println!("Left");
                            scene.camera_left();
                        },
                        Key::RightArrow => {
                            println!("Right");
                            scene.camera_right();
                        },

                        _ => {},
                    };
                },

                Event::KeyRelease(key) => {
                    key;
                },

                _ => {},
            };

        }



        let elapsed = now.elapsed().as_millis() as f32;
        if elapsed > dt_ms {
            now = Instant::now();
            println!("{}", 1000.0 / elapsed);
            scene.camera_right();

            scene.render();

            let frame_slice = scene.frame_as_bytes_slice();
            win.write_frame_from_slice(frame_slice);
        }
    }

}
