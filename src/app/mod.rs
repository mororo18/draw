mod window;
mod gui;

use std::ffi::{CString, CStr};
use rfd::FileDialog;

use crate::renderer::scene::{Scene, Object};
use crate::renderer::canvas::Canvas;

use gui::*;

use window::{
    Window,
    Event,
};

//struct AppState();

enum CameraNavigation {
    Free,
    Locked,
}

enum ImgFileFormat {
    Jpeg,
    Png,
}

enum UserAction {
    Open,
    ExportAs(ImgFileFormat),
}

const PIXEL_BYTES: usize = 4;

pub
struct Application {
    gui:    Gui,
    win:    Window,
    scene:  Scene,
    canvas: Canvas,

    current_frame: Vec<u8>,

    width:  usize,
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
            current_frame: vec![0; width * height * PIXEL_BYTES],
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

                        self.current_frame.resize(
                            self.width * self.height * PIXEL_BYTES, 0
                        );

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

                    Event::MouseMotion(mouse_info) => {
                        self.move_camera(mouse_info.dx, mouse_info.dy);
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

            let mut user_action: Option<UserAction> = None;

            let elapsed = now.elapsed();
            let ms_elapsed = elapsed.as_millis() as f32;
            //if ms_elapsed > dt_ms {
            if true {
                now = std::time::Instant::now();
                //println!("FPS {}", 1000.0 / ms_elapsed);
                //scene.camera_right();

                self.scene.render(&mut self.canvas);

                // Stores a copy of the current frame before
                // the GUI is rendered.
                self.current_frame
                    .as_mut_slice()
                    .copy_from_slice(
                        self.canvas.as_bytes_slice()
                    );


                self.gui.new_frame(&mut self.win, &frame_events, elapsed); 
                self.gui.build_ui(&mut user_action);
                self.gui.render(&mut self.canvas);
                frame_events.clear();

                let frame_slice = self.canvas.as_bytes_slice();
                self.win.write_frame_from_slice(frame_slice);

            }

            if let Some(action) = user_action {
                match action {
                    UserAction::ExportAs(img_fmt) => {
                        self.export_frame_as(img_fmt);
                    },
                    UserAction::Open => {
                        self.open_file();
                    },
                    _ => {},
                }
            }

        }
    }

    fn move_camera(&mut self, dx: i32, dy: i32) {
        self.scene.move_camera_direction(dx, -dy);
    }

    fn open_file(&mut self) {

        let file = FileDialog::new()
            .add_filter("text", &["obj"])
            .set_directory("$HOME")
            .pick_file();

        if let Some(file_path) = file {
            let obj = Object::load_from_file(
                file_path.to_str().unwrap()
            );

            self.scene.add_obj(obj);
        }
    }

    fn export_frame_as(&self, img_fmt: ImgFileFormat) {

        let file_extensions = match img_fmt {
            ImgFileFormat::Jpeg => { ["jpeg", "jpg"].as_slice() },
            ImgFileFormat::Png  => { ["png"].as_slice() },
        };

        let file = FileDialog::new()
            .add_filter("text", file_extensions)
            .set_directory("$HOME")
            .save_file();

        if let Some(file_path) = file {
            let mut output_path = file_path.clone();

            let output_extension =
                if let Some(given_extension) = file_path.extension() {
                    let user_extension = given_extension.to_str();

                    // verifies if user gave a valid extension
                    if user_extension.is_some()  &&
                        file_extensions.contains(
                            &user_extension.unwrap()
                        )
                    {
                        user_extension.unwrap()
                    } else {
                        file_extensions[0]
                    }

                } else {
                    file_extensions[0]
                };

            output_path.set_extension(output_extension);
            let output_c_str = CString::new(output_path.to_str().unwrap()).unwrap();

            // reverse the RGB order
            let mut out_frame = self.current_frame.clone();
            out_frame.as_mut_slice()
                .chunks_mut(PIXEL_BYTES)
                .for_each(|pixel_slice| { pixel_slice.swap(0, 2) });


            Self::write_img(out_frame.as_slice(), self.width, self.height, output_c_str, img_fmt);
        }
    }

    fn write_img(data: &[u8], width: usize, height: usize, path: CString, img_fmt: ImgFileFormat) {

        type StbImageWriteFn = fn(
            &CStr, 
            i32, 
            i32, 
            i32, 
            &[u8], 
            i32
        ) -> Option<()>;

        let stbi_write: StbImageWriteFn = match img_fmt {
            ImgFileFormat::Jpeg => { stb::image_write::stbi_write_jpg },
            ImgFileFormat::Png  => { stb::image_write::stbi_write_png },
        };

        // TODO: print error message
        let _ret = stbi_write(
            path.as_c_str(),
            width  as _,
            height as _,
            PIXEL_BYTES as _, 
            data,
            (width * PIXEL_BYTES) as _,
        );
    }
}
