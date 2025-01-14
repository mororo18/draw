use crate::renderer::scene::{Texture, TextureMap};
use crate::renderer::linalg::Vec2;
use crate::renderer::canvas::{Canvas, Color, Rectangle, VertexSimpleAttributes};

pub struct WindowDecorator {
    canvas: Canvas,

    width: i32,
    height: i32,
}

impl WindowDecorator {
    //pub const FONT_SIZE: f32 = 14.0;
    pub fn new(_content_dimensions: (i32, i32), window_dimensions: (i32, i32)) -> Self {

        let (win_width, win_height) = window_dimensions;

        Self {
            width: win_width,
            height: win_height,
            canvas: Canvas::new(win_width as _, win_height as _),
        }
    }

    pub fn render(&mut self) {
        self.canvas.fill_color(Color::Transparent);
        self.canvas.draw_rect(
            Rectangle::new(0, 0, (self.width -1) as _, (self.height - 1) as _),
            Color::Red
        );
    }

    pub fn frame_as_bytes_slice(&self) -> &[u8] {
        self.canvas.as_bytes_slice()
    }

}

