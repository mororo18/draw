use imgui as ig;
use super::window::{
    Event,
    Key,
};


pub
struct Gui {
    imgui:  ig::Context,
    width: usize,
    height: usize,
}

impl Gui {

    pub
    const FONT_SIZE: f32 = 13.0;

    pub
    fn new () -> Self {
        let width = 800;
        let height = 600;

        let mut imgui =  ig::Context::create();

        let io = imgui.io_mut();
        io.display_size = [width as f32, height as f32];

        let font_atlas = imgui.fonts(); 

        font_atlas.add_font(&[
            ig::FontSource::TtfData {
                data: include_bytes!("/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf"),
                size_pixels: Self::FONT_SIZE,
                config: Some(ig::FontConfig::default()),
            },
        ]);

        font_atlas.build_rgba32_texture();

        assert!(font_atlas.is_built());

        imgui.set_ini_filename(None);


        Self {
            width: width,
            height: height,
            imgui: imgui,
        }
    }

    pub
    fn render(&mut self, frame_buffer: &[u8], events: &Vec<Event>, delta_time: f32) {
        let ui = self.imgui.new_frame();

        let popup_title = "meu popup";

        ui.begin_popup(popup_title);
        ui.open_popup(popup_title);

        let draw_data = self.imgui.render();

        fn type_of<T>(_: &T) -> String {
            String::from(std::any::type_name::<T>())
        }

        println!("draw lsit counts {}", draw_data.draw_lists_count());
        for draw_list in draw_data.draw_lists() {

            let idx_buffer = draw_list.idx_buffer();
            let vtx_buffer = draw_list.vtx_buffer();

            // ver -> https://docs.rs/imgui/latest/imgui/enum.DrawCmd.html
            //     -> https://github.com/ocornut/imgui/blob/master/backends/imgui_impl_opengl2.cpp#L202
            for cmd in draw_list.commands() {


                // 1o) clipar 
                // 2o) desenhar
            }
        }

    }
}
