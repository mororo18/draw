use imgui as ig;
use stb;

use super::window::{
    Window,
    Event,
    Key,
    Button,
    MouseCursor,
};

use crate::renderer::canvas::{Canvas, VertexSimpleAttributes, Color, Rectangle};
use crate::renderer::scene::{Texture, TextureMap};
use crate::renderer::linalg::Vec2;

pub
struct Gui {
    imgui:  ig::Context,
    width: usize,
    height: usize,
    font_texture: Texture,

    hide_native_cursor: bool,
    current_mouse_cursor: Option<ig::MouseCursor>,
}

impl Gui {

    pub
    const FONT_SIZE: f32 = 17.0;

    pub
    fn update_display_size (&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.io().display_size = [width as f32, height as f32];
    }

    pub
    fn new (width: usize, height: usize) -> Self {

        let mut imgui =  ig::Context::create();
        let io = imgui.io_mut();
        io.display_size = [width as f32, height as f32];
        io.display_framebuffer_scale = [1.0, 1.0];

        imgui.style_mut().use_classic_colors();

        let font_atlas = imgui.fonts(); 

        font_atlas.add_font(&[
            ig::FontSource::DefaultFontData {
                config: Some(ig::FontConfig {
                    //rasterizer_multiply: 1.4,
                    //oversample_h: 4,
                    //oversample_v: 4,
                    ..ig::FontConfig::default()
                }),
            },
        ]);

        let font_atlas_texture = font_atlas.build_rgba32_texture();

        stb::image_write::stbi_write_png(
            unsafe{ std::ffi::CStr::from_ptr(c"textura-font.png".as_ptr()) }, 
            font_atlas_texture.width  as _,
            font_atlas_texture.height as _,
            4, 
            font_atlas_texture.data,
            (font_atlas_texture.width * 4) as _,
        );

        let f_texture =  Texture::with_diffuse_map(
            TextureMap::new(
                font_atlas_texture.data.to_vec(),
                font_atlas_texture.width  as _,
                font_atlas_texture.height as _,
                4,
            )
        );

        assert!(font_atlas.is_built());

        imgui.set_ini_filename(None);

        Self {
            width,
            height,
            imgui,
            font_texture: f_texture,

            hide_native_cursor: false,
            current_mouse_cursor: None,
        }
    }

    fn io(&mut self) -> &mut ig::Io {
        self.imgui.io_mut()
    }

    fn update_mouse_cursor(&mut self, win: &mut Window) {

        if self.hide_native_cursor != self.io().mouse_draw_cursor {
            self.hide_native_cursor = self.io().mouse_draw_cursor;

            if self.hide_native_cursor {
                win.hide_mouse_cursor();
            } else {
                win.show_mouse_cursor();
            }
        }

        if self.current_mouse_cursor != self.imgui.mouse_cursor() {
            self.current_mouse_cursor = self.imgui.mouse_cursor();

            if let Some(cursor) = self.current_mouse_cursor {
                let native_cursor = match cursor {
                    ig::MouseCursor::Arrow      => MouseCursor::Arrow,
                    ig::MouseCursor::TextInput  => MouseCursor::TextInput,
                    ig::MouseCursor::ResizeAll  => MouseCursor::ResizeAll,
                    ig::MouseCursor::ResizeNS   => MouseCursor::ResizeNS,
                    ig::MouseCursor::ResizeEW   => MouseCursor::ResizeEW,
                    ig::MouseCursor::ResizeNESW => MouseCursor::ResizeNESW,
                    ig::MouseCursor::ResizeNWSE => MouseCursor::ResizeNWSE,
                    ig::MouseCursor::Hand       => MouseCursor::Hand,
                    ig::MouseCursor::NotAllowed => MouseCursor::NotAllowed,
                };

                win.update_mouse_cursor(native_cursor);
            }
        }
    }

    fn process_events (&mut self, events: &Vec<Event>) {
        let io = self.io();
        for event in events.iter() {
            match event {
                Event::MouseMotion(mouse_info) => {
                    io.mouse_pos = [mouse_info.x as f32, mouse_info.y as f32];
                },

                Event::ButtonPress(button_event) => {
                    let idx = match button_event {
                        Button::MouseLeft   => 0,
                        Button::MouseRight  => 1,
                        Button::MouseMiddle => 2,
                        _ => 0,
                    };

                    io.mouse_down[idx] = true;
                },

                Event::ButtonRelease(button_event) => {

                    let idx = match button_event {
                        Button::MouseLeft   => 0,
                        Button::MouseRight  => 1,
                        Button::MouseMiddle => 2,
                        _ => 0,
                    };

                    io.mouse_down[idx] = false;
                },

                Event::KeyPress(key) => {
                    if let Some(ig_key) = Self::key_to_imgui_key(key) {
                        io.add_key_event(ig_key, true);
                    }
                    /*
                    match key {
                        Key::Sym(keysym)   => {
                            let ksym = match io.key_shift {
                                false => keysym.0,
                                true  => keysym.1,
                            };

                            let kindex = ksym as usize;
                            let kchar = std::char::from_u32(ksym).unwrap();

                            io.keys_down[kindex] = true;
                            io.add_input_character(kchar);
                            // latin-1 range
                            if ksym < 0xff {
                            }
                        },

                    }
                    */
                },

                Event::KeyRelease(key) => {
                    if let Some(ig_key) = Self::key_to_imgui_key(key) {
                        io.add_key_event(ig_key, false);
                    }
                    /*
                    match key {
                        Key::Sym(keysym)   => {
                            let ksym = match io.key_shift {
                                false => keysym.0,
                                true  => keysym.1,
                            };

                            let kindex = ksym as usize;
                            let kchar = std::char::from_u32(ksym).unwrap();

                            // latin-1 range
                            if ksym < 0xff {
                                io.keys_down[kindex] = false;
                            }
                        },

                        Key::Code(keycode) => {},
                    }
                    */
                },

                _ => {},
            }
        }
    }

    pub
    fn new_frame (&mut self, win: &mut Window, events: &Vec<Event>, delta_time: std::time::Duration) {
        let io = self.io();
        io.update_delta_time(delta_time);

        self.process_events(events);

        self.update_mouse_cursor(win);
    }

    pub
    fn build_ui (&mut self) {
        let ui = self.imgui.new_frame();

        let ig_win = ui.window("janela")
                        .menu_bar(true);


        ig_win.build(|| {
            let _w = ui.push_item_width(-140.0);
            ui.text(format!("dear imgui says hello. ({})", imgui::dear_imgui_version()));

            if let Some(_menu_bar) = ui.begin_menu_bar() {

                if let Some(menu) = ui.begin_menu("Menu") {
                    ui.menu_item_config("(dummy_menu)").enabled(false).build();
                    ui.menu_item("New");
                    ui.menu_item_config("Open").shortcut("Ctrl+O").build();

                    menu.end();
                }
            }
        });
    }

    pub
    fn render(&mut self, canvas: &mut Canvas) {


        canvas.disable_depth_update();

        let draw_data = self.imgui.render();

        let invert_y = |y: f32| -> f32 {
            self.height as f32 - 1.0 - y
        };

        for draw_list in draw_data.draw_lists() {

            let idx_buffer = draw_list.idx_buffer();
            let vtx_buffer = draw_list.vtx_buffer();

            // ver -> https://docs.rs/imgui/latest/imgui/enum.DrawCmd.html
            //     -> https://github.com/ocornut/imgui/blob/master/backends/imgui_impl_opengl2.cpp#L202
            for draw_cmd in draw_list.commands() {

                match draw_cmd {
                    ig::DrawCmd::Elements { count, cmd_params }  => {
                        let mut clip_rect: [f32; 4] = cmd_params.clip_rect;
                        let texture_id: usize = cmd_params.texture_id.id();
                        let vtx_offset: usize = cmd_params.vtx_offset;
                        let idx_offset: usize = cmd_params.idx_offset;

                        /*
                        dbg!(count);
                        dbg!(texture_id);
                        dbg!(vtx_offset);
                        dbg!(idx_offset);
                        */

                        if clip_rect[0] > clip_rect[2] || clip_rect[1] > clip_rect[3] { continue }

                        // invert the y
                        clip_rect[1] = self.height as f32 - clip_rect[1];
                        clip_rect[3] = self.height as f32 - clip_rect[3];

                        // 2o) desenhar
                        let idx_buff_slice = &idx_buffer[idx_offset..idx_offset+count];
                        for indexed_tri in idx_buff_slice.chunks_exact(3) {
                            let a_idx = indexed_tri[0] as usize + vtx_offset;
                            let b_idx = indexed_tri[1] as usize + vtx_offset;
                            let c_idx = indexed_tri[2] as usize + vtx_offset;

                            let a_vtx = VertexSimpleAttributes {
                                screen_coord: Vec2::new(
                                              vtx_buffer[a_idx].pos[0], 
                                    invert_y( vtx_buffer[a_idx].pos[1]),
                                ),
                                color: Color::Custom(
                                    vtx_buffer[a_idx].col[..3].try_into().unwrap()
                                ),
                                texture_coord: Vec2::new(
                                    vtx_buffer[a_idx].uv[0], 
                                    1.0 - vtx_buffer[a_idx].uv[1]
                                ),
                                alpha: vtx_buffer[a_idx].col[3] as f32 / 255.0,
                            };

                            let b_vtx = VertexSimpleAttributes {
                                screen_coord: Vec2::new(
                                              vtx_buffer[b_idx].pos[0], 
                                    invert_y( vtx_buffer[b_idx].pos[1]),
                                ),
                                color: Color::Custom(
                                    vtx_buffer[b_idx].col[..3].try_into().unwrap()
                                ),
                                texture_coord: Vec2::new(
                                    vtx_buffer[b_idx].uv[0], 
                                    1.0 - vtx_buffer[b_idx].uv[1]
                                ),
                                alpha: vtx_buffer[b_idx].col[3] as f32 / 255.0,
                            };

                            let c_vtx = VertexSimpleAttributes {
                                screen_coord: Vec2::new(
                                              vtx_buffer[c_idx].pos[0], 
                                    invert_y( vtx_buffer[c_idx].pos[1] ),
                                ),
                                color: Color::Custom(
                                    vtx_buffer[c_idx].col[..3].try_into().unwrap()
                                ),
                                texture_coord: Vec2::new(
                                    vtx_buffer[c_idx].uv[0], 
                                    1.0 - vtx_buffer[c_idx].uv[1]
                                ),
                                alpha: vtx_buffer[c_idx].col[3] as f32 / 255.0,
                            };


                            canvas.draw_triangle(
                                a_vtx,
                                b_vtx,
                                c_vtx,
                                Some(&self.font_texture),
                                Some(Rectangle::from_coords(
                                    clip_rect[0] as usize,
                                    clip_rect[1] as usize,
                                    clip_rect[2] as usize,
                                    clip_rect[3] as usize,
                                ))
                            );
                        }
                    },
                    ig::DrawCmd::ResetRenderState => {},
                    _ => {},
                }
            }
        }
    }

    fn key_to_imgui_key(key: &Key) -> Option<ig::Key> {
        match key {
            Key::Tab => Some(ig::Key::Tab),
            Key::LeftArrow => Some(ig::Key::LeftArrow),
            Key::RightArrow => Some(ig::Key::RightArrow),
            Key::UpArrow => Some(ig::Key::UpArrow),
            Key::DownArrow => Some(ig::Key::DownArrow),
            Key::PageUp => Some(ig::Key::PageUp),
            Key::PageDown => Some(ig::Key::PageDown),
            Key::Home => Some(ig::Key::Home),
            Key::End => Some(ig::Key::End),
            Key::Insert => Some(ig::Key::Insert),
            Key::Delete => Some(ig::Key::Delete),
            Key::Backspace => Some(ig::Key::Backspace),
            Key::Space => Some(ig::Key::Space),
            Key::Enter => Some(ig::Key::Enter),
            Key::Escape => Some(ig::Key::Escape),
            Key::Apostrophe => Some(ig::Key::Apostrophe),
            Key::Comma => Some(ig::Key::Comma),
            Key::Minus => Some(ig::Key::Minus),
            Key::Period => Some(ig::Key::Period),
            Key::Slash => Some(ig::Key::Slash),
            Key::Semicolon => Some(ig::Key::Semicolon),
            Key::Equal => Some(ig::Key::Equal),
            Key::LeftBracket => Some(ig::Key::LeftBracket),
            Key::Backslash => Some(ig::Key::Backslash),
            Key::RightBracket => Some(ig::Key::RightBracket),
            Key::GraveAccent => Some(ig::Key::GraveAccent),
            Key::CapsLock => Some(ig::Key::CapsLock),
            Key::ScrollLock => Some(ig::Key::ScrollLock),
            Key::NumLock => Some(ig::Key::NumLock),
            Key::PrintScreen => Some(ig::Key::PrintScreen),
            Key::Pause => Some(ig::Key::Pause),
            Key::Keypad0 => Some(ig::Key::Keypad0),
            Key::Keypad1 => Some(ig::Key::Keypad1),
            Key::Keypad2 => Some(ig::Key::Keypad2),
            Key::Keypad3 => Some(ig::Key::Keypad3),
            Key::Keypad4 => Some(ig::Key::Keypad4),
            Key::Keypad5 => Some(ig::Key::Keypad5),
            Key::Keypad6 => Some(ig::Key::Keypad6),
            Key::Keypad7 => Some(ig::Key::Keypad7),
            Key::Keypad8 => Some(ig::Key::Keypad8),
            Key::Keypad9 => Some(ig::Key::Keypad9),
            Key::KeypadDecimal => Some(ig::Key::KeypadDecimal),
            Key::KeypadDivide => Some(ig::Key::KeypadDivide),
            Key::KeypadMultiply => Some(ig::Key::KeypadMultiply),
            Key::KeypadSubtract => Some(ig::Key::KeypadSubtract),
            Key::KeypadAdd => Some(ig::Key::KeypadAdd),
            Key::KeypadEnter => Some(ig::Key::KeypadEnter),
            Key::KeypadEqual => Some(ig::Key::KeypadEqual),
            Key::LeftCtrl => Some(ig::Key::LeftCtrl),
            Key::LeftShift => Some(ig::Key::LeftShift),
            Key::LeftAlt => Some(ig::Key::LeftAlt),
            Key::LeftSuper => Some(ig::Key::LeftSuper),
            Key::RightCtrl => Some(ig::Key::RightCtrl),
            Key::RightShift => Some(ig::Key::RightShift),
            Key::RightAlt => Some(ig::Key::RightAlt),
            Key::RightSuper => Some(ig::Key::RightSuper),
            Key::Menu => Some(ig::Key::Menu),
            Key::Num0 => Some(ig::Key::Alpha0),
            Key::Num1 => Some(ig::Key::Alpha1),
            Key::Num2 => Some(ig::Key::Alpha2),
            Key::Num3 => Some(ig::Key::Alpha3),
            Key::Num4 => Some(ig::Key::Alpha4),
            Key::Num5 => Some(ig::Key::Alpha5),
            Key::Num6 => Some(ig::Key::Alpha6),
            Key::Num7 => Some(ig::Key::Alpha7),
            Key::Num8 => Some(ig::Key::Alpha8),
            Key::Num9 => Some(ig::Key::Alpha9),
            Key::A => Some(ig::Key::A),
            Key::B => Some(ig::Key::B),
            Key::C => Some(ig::Key::C),
            Key::D => Some(ig::Key::D),
            Key::E => Some(ig::Key::E),
            Key::F => Some(ig::Key::F),
            Key::G => Some(ig::Key::G),
            Key::H => Some(ig::Key::H),
            Key::I => Some(ig::Key::I),
            Key::J => Some(ig::Key::J),
            Key::K => Some(ig::Key::K),
            Key::L => Some(ig::Key::L),
            Key::M => Some(ig::Key::M),
            Key::N => Some(ig::Key::N),
            Key::O => Some(ig::Key::O),
            Key::P => Some(ig::Key::P),
            Key::Q => Some(ig::Key::Q),
            Key::R => Some(ig::Key::R),
            Key::S => Some(ig::Key::S),
            Key::T => Some(ig::Key::T),
            Key::U => Some(ig::Key::U),
            Key::V => Some(ig::Key::V),
            Key::W => Some(ig::Key::W),
            Key::X => Some(ig::Key::X),
            Key::Y => Some(ig::Key::Y),
            Key::Z => Some(ig::Key::Z),
            Key::F1 => Some(ig::Key::F1),
            Key::F2 => Some(ig::Key::F2),
            Key::F3 => Some(ig::Key::F3),
            Key::F4 => Some(ig::Key::F4),
            Key::F5 => Some(ig::Key::F5),
            Key::F6 => Some(ig::Key::F6),
            Key::F7 => Some(ig::Key::F7),
            Key::F8 => Some(ig::Key::F8),
            Key::F9 => Some(ig::Key::F9),
            Key::F10 => Some(ig::Key::F10),
            Key::F11 => Some(ig::Key::F11),
            Key::F12 => Some(ig::Key::F12),
            _ => None,
        }
    }

}
