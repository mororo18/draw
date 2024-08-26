use imgui as ig;
use stb;
use super::window::{
    Window,
    Event,
    Key,
    MouseCursor,
};

use crate::renderer::canvas::{Canvas, VertexSimpleAttributes, Color};
use crate::renderer::scene::{Texture, TextureMap};
use crate::renderer::linalg::{Vec2, Vec3, Vec4, Matrix4};

pub
struct Gui {
    imgui:  ig::Context,
    width: usize,
    height: usize,
    font_texture: Texture,

    hide_native_cursor: bool,
    custom_mouse_cursor: Option<ig::MouseCursor>,
}

impl Gui {

    pub
    const FONT_SIZE: f32 = 17.0;

    pub
    fn new () -> Self {
        let width = 800;
        let height = 600;

        let mut imgui =  ig::Context::create();
        let io = imgui.io_mut();
        //io.backend_flags = ig::BackendFlags::RENDERER_HAS_VIEWPORTS;
        io.display_size = [width as f32, height as f32];
        //io.display_size = [1.0, 1.0];
        //io.display_framebuffer_scale = [width as f32, height as f32];
        //
        imgui.style_mut().use_classic_colors();

        let font_atlas = imgui.fonts(); 

        font_atlas.add_font(&[
            ig::FontSource::DefaultFontData {
                config: Some(ig::FontConfig {
                    rasterizer_multiply: 1.5,
                    oversample_h: 4,
                    oversample_v: 4,
                    ..ig::FontConfig::default()
                }),
            },
            /*
            ig::FontSource::TtfData {
                data: include_bytes!("/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf"),
                size_pixels: Self::FONT_SIZE,
                config: Some(ig::FontConfig {
                    rasterizer_multiply: 1.5,
                    oversample_h: 4,
                    oversample_v: 4,
                    ..ig::FontConfig::default()
                }),
            },
            */
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
            width: width,
            height: height,
            imgui: imgui,
            font_texture: f_texture,

            hide_native_cursor: false,
            custom_mouse_cursor: None,
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

        if self.custom_mouse_cursor != self.imgui.mouse_cursor() {
            self.custom_mouse_cursor = self.imgui.mouse_cursor();

            if let Some(cursor) = self.custom_mouse_cursor {
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

    pub
    fn new_frame(&mut self, win: &mut Window, events: &Vec<Event>, delta_time: std::time::Duration) {
        let io = self.io();

        io.update_delta_time(delta_time);

        self.update_mouse_cursor(win);
    }

    pub
    fn render(&mut self, canvas: &mut Canvas) {


        canvas.disable_depth_update();
        let ui = self.imgui.new_frame();

        ui.separator();
        ui.button("finalmente");

        let draw_data = self.imgui.render();

        /*
        let n_x: f32 = self.width as _;
        let n_y: f32 = self.height as _;

        let n = 1.0;
        let f = -1.0;

        let r = ;
        let l = ;

        let t = camera_window.top;
        let b = camera_window.bottom;
        */


        /*
        let matrix_viewport = Matrix4::new([
            [n_x / 2.0,        0.0,  0.0,  (n_x-1.0) / 2.0],
            [      0.0,  n_y / 2.0,  0.0,  (n_y-1.0) / 2.0],
            [      0.0,        0.0,  1.0,              0.0],
            [      0.0,        0.0,  0.0,              1.0]
        ]);
        let matrix_orth = Matrix4::new([
            [2.0 / (r-l),          0.0,          0.0,  -(r+l) / (r-l)],
            [        0.0,  2.0 / (t-b),          0.0,  -(t+b) / (t-b)],
            [        0.0,          0.0,  2.0 / (n-f),  -(n+f) / (n-f)],
            [        0.0,          0.0,          0.0,             1.0]
        ]);
        */






        fn type_of<T>(_: &T) -> String {
            String::from(std::any::type_name::<T>())
        }

        println!("draw data display size     {:?}", draw_data.display_size);
        println!("draw data display position {:?}", draw_data.display_pos);
        println!("draw data frame buff scale {:?}", draw_data.framebuffer_scale);

        for draw_list in draw_data.draw_lists() {

            let idx_buffer = draw_list.idx_buffer();
            let vtx_buffer = draw_list.vtx_buffer();

            // ver -> https://docs.rs/imgui/latest/imgui/enum.DrawCmd.html
            //     -> https://github.com/ocornut/imgui/blob/master/backends/imgui_impl_opengl2.cpp#L202
            for (idx, draw_cmd) in draw_list.commands().enumerate() {
                //dbg!(idx);

                match draw_cmd {
                    ig::DrawCmd::Elements { count, cmd_params }  => {
                        let clip_rect: [f32; 4] = cmd_params.clip_rect;
                        let texture_id: usize = cmd_params.texture_id.id();
                        let vtx_offset: usize = cmd_params.vtx_offset;
                        let idx_offset: usize = cmd_params.idx_offset;

                        /*
                        dbg!(count);
                        dbg!(texture_id);
                        dbg!(vtx_offset);
                        dbg!(idx_offset);
                        dbg!(clip_rect);
                        */

                        // TODO: clipar (criar argumento para passar a func draw do Canvas).
                        
                        // 2o) desenhar
                        let idx_buff_slice = &idx_buffer[idx_offset..idx_offset+count];
                        for indexed_tri in idx_buff_slice.chunks_exact(3) {
                            let a_idx = indexed_tri[0] as usize + vtx_offset;
                            let b_idx = indexed_tri[1] as usize + vtx_offset;
                            let c_idx = indexed_tri[2] as usize + vtx_offset;

                            /*
                            let a_screen_coord =  (matrix_viewport * Vec4::new([
                                vtx_buffer[a_idx].pos[0], 
                                vtx_buffer[a_idx].pos[1],
                                0.0,
                                0.0,
                            ])).as_vec2();

                            let b_screen_coord =  (matrix_viewport * Vec4::new([
                                vtx_buffer[b_idx].pos[0], 
                                vtx_buffer[b_idx].pos[1],
                                0.0,
                                0.0,
                            ])).as_vec2();

                            let c_screen_coord =  (matrix_viewport * Vec4::new([
                                vtx_buffer[c_idx].pos[0], 
                                vtx_buffer[c_idx].pos[1],
                                0.0,
                                0.0,
                            ])).as_vec2();
                            */


                            let a_vtx = VertexSimpleAttributes {
                                screen_coord: Vec2::new(
                                    vtx_buffer[a_idx].pos[0], 
                                    vtx_buffer[a_idx].pos[1],
                                ),
                                /*
                                screen_coord: a_screen_coord,
                                */
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
                                    vtx_buffer[b_idx].pos[1],
                                ),
                                /*
                                screen_coord: b_screen_coord,
                                */
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
                                    vtx_buffer[c_idx].pos[1],
                                ),
                                
                                /*
                                screen_coord: c_screen_coord,
                                */
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
                            );
                        }
                        
                    },
                    ig::DrawCmd::ResetRenderState => {},
                    _ => {},
                }

            }
        }

    }
}
