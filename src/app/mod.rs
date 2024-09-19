mod window;
mod gui;

use crate::renderer::scene::Scene;
use crate::renderer::canvas::Canvas;

use gui::*;

use window::{
    Window,
    Event,
};


pub
struct Application {
    gui: Gui,
    win: Window,
    scene: Scene,
    canvas: Canvas,
    width: usize,
    height: usize,
}

impl Application {
    pub
    fn new () -> Self {
        let width = 800;
        let height = 600;


        let win = Window::new(width, height);
        let (screen_width, screen_height) = win.get_screen_dim();

        Self {
            gui:    Gui::new(width, height),
            scene:  Scene::new(screen_width, screen_height),
            win,
            canvas: Canvas::new(width, height),
            width,
            height,
        }
    }

    pub
    fn run (&mut self) {

        let (screen_width, screen_height) = self.win.get_screen_dim();
        self.canvas.init_depth(100000.0);
        self.canvas.apply_offset(
            ((screen_width  - self.width)  / 2) as _,
            ((screen_height - self.height) / 2) as _,
        );


        let frame_rate: f32 = 60.0;
        let dt_ms = 1000.0 / frame_rate;

        let mut now = std::time::Instant::now();

        let mut window_open = true;

        let mut frame_events: Vec<Event> = vec![];

        while window_open {
            let events: Vec<Event> = self.win.handle();

            for e in events.iter() {

                match e {
                    Event::CloseWindow => {
                        window_open = false;
                    },

                    Event::RedimWindow((width, height)) => {
                        self.width  = *width;
                        self.height = *height;

                        self.canvas.resize(self.width, self.height);
                        self.canvas.apply_offset(
                            ((screen_width  - self.width)  / 2) as _,
                            ((screen_height - self.height) / 2) as _,
                        );

                        self.gui.update_display_size(self.width, self.height);

                        println!("redim {width} x {height}");
                    },

                    Event::ReposWindow((x, y)) => {
                        println!("{x} x {y}");
                    },

                    Event::KeyPress(key) => {
                        /*
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
                        */
                    },

                    Event::KeyRelease(_key) => {
                    },

                    _ => {},
                };

            }


            frame_events.extend(events);

            let elapsed = now.elapsed();
            let ms_elapsed = elapsed.as_millis() as f32;
            //if ms_elapsed > dt_ms {
            if true {
                now = std::time::Instant::now();
                //println!("FPS {}", 1000.0 / ms_elapsed);
                //scene.camera_right();

                self.scene.render(&mut self.canvas);
                //let render_elapsed = now.elapsed().as_millis() as f32;
                //println!("Rendering percentage {}%", render_elapsed * 100.0 / dt_ms);

                self.gui.new_frame(&mut self.win, &frame_events, elapsed); 
                self.gui.build_ui();
                self.gui.render(&mut self.canvas);
                frame_events.clear();

                let frame_slice = self.canvas.as_bytes_slice();
                self.win.write_frame_from_slice(frame_slice);

            }
        }
    }
}
