mod window;
mod gui;

use crate::renderer::scene::Scene;
use crate::renderer::canvas::Canvas;

use gui::*;

use window::{
    Window,
    Event,
    Key,
};


pub
struct Application {
    gui: Gui,
    win: Window,
    scene: Scene,
    canvas: Canvas,
}

impl Application {
    pub
    fn new () -> Self {
        let width = 800;
        let height = 600;
        let mut canvas = Canvas::new(width, height);
        canvas.init_depth(100000.0);


        Self {
            gui:    Gui::new(),
            scene:  Scene::new(width, height),
            win:    Window::new(width, height),
            canvas: canvas,
        }
    }

    pub
    fn run (&mut self) {

        let frame_rate: f32 = 60.0;
        let dt_ms = 1000.0 / frame_rate;

        let mut now = std::time::Instant::now();

        let mut window_open = true;

        while window_open {
            let events: Vec<Event> = self.win.handle();

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
                                self.scene.camera_up();
                            },
                            Key::DownArrow => {
                                println!("Down");
                                self.scene.camera_down();
                            },
                            Key::LeftArrow => {
                                println!("Left");
                                self.scene.camera_left();
                            },
                            Key::RightArrow => {
                                println!("Right");
                                self.scene.camera_right();
                            },

                            _ => {println!("Unknow")},
                        };
                    },

                    Event::KeyRelease(_key) => {
                    },

                    _ => {},
                };

            }



            let elapsed = now.elapsed().as_millis() as f32;
            if elapsed > dt_ms {
                now = std::time::Instant::now();
                println!("FPS {}", 1000.0 / elapsed);
                //scene.camera_right();

                self.scene.render(&mut self.canvas);
                //let render_elapsed = now.elapsed().as_millis() as f32;
                //println!("Rendering percentage {}%", render_elapsed * 100.0 / dt_ms);

                self.gui.render(&mut self.canvas, &events, elapsed);

                let frame_slice = self.canvas.as_bytes_slice();
                self.win.write_frame_from_slice(frame_slice);

            }
        }
    }
}
