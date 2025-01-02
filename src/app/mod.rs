mod gui;
mod window;

use rfd::FileDialog;
use std::ffi::{CStr, CString};

use crate::renderer::canvas::Canvas;
use crate::renderer::scene::{Object, ObjectInfo, Scene};

use gui::*;

use window::{Event, Key, Window, X11Window};

//struct AppState();

enum CameraNavigation {
    Free,
    Locked,
}

enum ImgFileFormat {
    Jpeg,
    Png,
}

enum GuiAction {
    Open,
    ExportAs(ImgFileFormat),
    ListModelsInfo,
}

const PIXEL_BYTES: usize = 4;

const CAMERA_FOWARDS: u8 = 1;
const CAMERA_BACKWARDS: u8 = 2;
const CAMERA_LEFT: u8 = 4;
const CAMERA_RIGHT: u8 = 8;
const CAMERA_UPWARDS: u8 = 16;
const CAMERA_DOWNWARDS: u8 = 32;

pub struct Application {
    gui: Gui,
    win: X11Window,
    scene: Scene,
    canvas: Canvas,

    current_frame: Vec<u8>,

    width: usize,
    height: usize,

    camera_mode: CameraNavigation,

    camera_moving_direction: u8,
}

impl Application {
    pub fn new() -> Self {
        let width = 800;
        let height = 600;

        let win: X11Window = Window::new(width, height);
        let (screen_width, screen_height) = win.get_screen_dim();

        Self {
            gui: Gui::new(width, height),
            scene: Scene::new(screen_width, screen_height),
            win,
            canvas: Canvas::new(width, height),
            current_frame: vec![0; width * height * PIXEL_BYTES],
            width,
            height,
            camera_mode: CameraNavigation::Locked,
            camera_moving_direction: 0,
        }
    }

    pub fn run(&mut self) {
        let (screen_width, screen_height) = self.win.get_screen_dim();
        self.canvas.init_depth(100000.0);
        self.canvas.apply_offset(
            ((screen_width - self.width) / 2) as _,
            ((screen_height - self.height) / 2) as _,
        );

        let frame_rate: f32 = 60.0;
        let dt_ms = 1000.0 / frame_rate;

        let mut now = std::time::Instant::now();

        let mut window_open = true;

        let mut frame_events: Vec<Event> = vec![];
        let mut user_action: Option<GuiAction>;

        while window_open {
            user_action = None;
            let events: Vec<Event> = self.win.handle();

            for e in events.iter() {
                match e {
                    Event::CloseWindow => {
                        window_open = false;
                    }

                    Event::RedimWindow((width, height)) => {
                        self.width = *width;
                        self.height = *height;

                        self.current_frame
                            .resize(self.width * self.height * PIXEL_BYTES, 0);

                        self.canvas.resize(self.width, self.height);
                        self.canvas.apply_offset(
                            ((screen_width - self.width) / 2) as _,
                            ((screen_height - self.height) / 2) as _,
                        );

                        self.gui.update_display_size(self.width, self.height);

                        println!("redim {width} x {height}");
                    }

                    Event::ReposWindow((x, y)) => {
                        println!("{x} x {y}");
                    }

                    Event::MouseMotion(mouse_info) => {
                        self.move_camera_direction(mouse_info.dx, mouse_info.dy);
                    }

                    Event::KeyPress(key) => {
                        print!("KeyPress ");
                        match key {
                            Key::F5 => self.toggle_camera_mode(),
                            _ => println!("Unknow"),
                        }

                        match self.camera_mode {
                            CameraNavigation::Free => match key {
                                Key::W | Key::UpArrow => self.add_camera_movement(CAMERA_FOWARDS),
                                Key::S | Key::DownArrow => {
                                    self.add_camera_movement(CAMERA_BACKWARDS)
                                }
                                Key::A | Key::LeftArrow => self.add_camera_movement(CAMERA_LEFT),
                                Key::D | Key::RightArrow => self.add_camera_movement(CAMERA_RIGHT),

                                Key::Space => self.add_camera_movement(CAMERA_UPWARDS),
                                Key::LeftShift => self.add_camera_movement(CAMERA_DOWNWARDS),

                                _ => {}
                            },

                            CameraNavigation::Locked => {
                                // TODO.
                            }
                        }
                    }

                    Event::KeyRelease(key) => {
                        match self.camera_mode {
                            CameraNavigation::Free => match key {
                                Key::W | Key::UpArrow => self.rm_camera_movement(CAMERA_FOWARDS),
                                Key::S | Key::DownArrow => {
                                    self.rm_camera_movement(CAMERA_BACKWARDS)
                                }
                                Key::A | Key::LeftArrow => self.rm_camera_movement(CAMERA_LEFT),
                                Key::D | Key::RightArrow => self.rm_camera_movement(CAMERA_RIGHT),

                                Key::Space => self.rm_camera_movement(CAMERA_UPWARDS),
                                Key::LeftShift => self.rm_camera_movement(CAMERA_DOWNWARDS),

                                _ => {}
                            },

                            CameraNavigation::Locked => {
                                // TODO.
                            }
                        }
                    }
                    _ => {}
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
                self.move_camera_position();

                self.scene.render(&mut self.canvas);

                // Stores a copy of the current frame before
                // the GUI is rendered.
                self.current_frame
                    .as_mut_slice()
                    .copy_from_slice(self.canvas.as_bytes_slice());

                self.gui.new_frame(&mut self.win, &frame_events, elapsed);
                self.gui.build_ui(&mut user_action);
                self.gui.render(&mut self.canvas);
                frame_events.clear();

                let frame_slice = self.canvas.as_bytes_slice();
                self.win.write_frame_from_slice(frame_slice);
            }

            if let Some(action) = user_action {
                match action {
                    GuiAction::ExportAs(img_fmt) => self.export_frame_as(img_fmt),
                    GuiAction::Open => {
                        let obj_info_ret = self.open_obj_file();
                        if let Some(obj_info) = obj_info_ret {
                            self.gui.add_obj(obj_info);
                        }
                    }
                    _ => {}
                }
            }

            match self.camera_mode {
                CameraNavigation::Free => {
                    self.win
                        .set_mouse_position((self.width / 2) as i32, (self.height / 2) as i32);
                }
                CameraNavigation::Locked => {
                    // TODO.
                }
            };
        }
    }

    fn toggle_camera_mode(&mut self) {
        self.camera_mode = match self.camera_mode {
            CameraNavigation::Free => {
                self.win.show_mouse_cursor();
                CameraNavigation::Locked
            }
            CameraNavigation::Locked => {
                self.win.hide_mouse_cursor();
                CameraNavigation::Free
            }
        };
    }

    fn add_camera_movement(&mut self, cam_direction: u8) {
        self.camera_moving_direction |= cam_direction;
    }

    fn rm_camera_movement(&mut self, cam_direction: u8) {
        self.camera_moving_direction &= !cam_direction;
    }

    // TODO: find better name to these func. maybe 'change_camera_position'
    fn move_camera_position(&mut self) {
        let foward = CAMERA_FOWARDS & self.camera_moving_direction != 0;
        let backward = CAMERA_BACKWARDS & self.camera_moving_direction != 0;
        let left = CAMERA_LEFT & self.camera_moving_direction != 0;
        let right = CAMERA_RIGHT & self.camera_moving_direction != 0;
        let up = CAMERA_UPWARDS & self.camera_moving_direction != 0;
        let down = CAMERA_DOWNWARDS & self.camera_moving_direction != 0;

        let delta = 1.5;

        if foward {
            self.scene.camera.move_foward(delta);
        }
        if backward {
            self.scene.camera.move_backward(delta);
        }
        if left {
            self.scene.camera.move_left(delta);
        }
        if right {
            self.scene.camera.move_right(delta);
        }
        if up {
            self.scene.camera.move_up(delta);
        }
        if down {
            self.scene.camera.move_down(delta);
        }
    }

    fn move_camera_direction(&mut self, dx: i32, dy: i32) {
        let sensibility = 3;
        match self.camera_mode {
            CameraNavigation::Free => self
                .scene
                .move_camera_direction(sensibility * dx, -sensibility * dy),
            CameraNavigation::Locked => {
                // TODO
            }
        }
    }

    fn open_obj_file(&mut self) -> Option<ObjectInfo> {
        let file = FileDialog::new()
            .add_filter("text", &["obj"])
            .set_directory("$HOME")
            .pick_file();

        if let Some(file_path) = file {
            let obj = Object::load_from_file(file_path.to_str().unwrap());

            Some(self.scene.add_obj(obj))
        } else {
            None
        }
    }

    fn export_frame_as(&self, img_fmt: ImgFileFormat) {
        let file_extensions = match img_fmt {
            ImgFileFormat::Jpeg => ["jpeg", "jpg"].as_slice(),
            ImgFileFormat::Png => ["png"].as_slice(),
        };

        let file = FileDialog::new()
            .add_filter("text", file_extensions)
            .set_directory("$HOME")
            .save_file();

        if let Some(file_path) = file {
            let mut output_path = file_path.clone();

            let output_extension = if let Some(given_extension) = file_path.extension() {
                let user_extension = given_extension.to_str();

                // verifies if user gave a valid extension
                if user_extension.is_some() && file_extensions.contains(&user_extension.unwrap()) {
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
            out_frame
                .as_mut_slice()
                .chunks_mut(PIXEL_BYTES)
                .for_each(|pixel_slice| pixel_slice.swap(0, 2));

            Self::write_img(
                out_frame.as_slice(),
                self.width,
                self.height,
                output_c_str,
                img_fmt,
            );
        }
    }

    fn write_img(data: &[u8], width: usize, height: usize, path: CString, img_fmt: ImgFileFormat) {
        type StbImageWriteFn = fn(&CStr, i32, i32, i32, &[u8], i32) -> Option<()>;

        let stbi_write: StbImageWriteFn = match img_fmt {
            ImgFileFormat::Jpeg => stb::image_write::stbi_write_jpg,
            ImgFileFormat::Png => stb::image_write::stbi_write_png,
        };

        // TODO: print error message
        let _ret = stbi_write(
            path.as_c_str(),
            width as _,
            height as _,
            PIXEL_BYTES as _,
            data,
            (width * PIXEL_BYTES) as _,
        );
    }
}
