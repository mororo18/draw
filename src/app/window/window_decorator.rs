use crate::renderer::canvas::{Canvas, Color, Rectangle, VertexSimpleAttributes};
use crate::renderer::linalg::Vec2;
use crate::renderer::scene::{Texture, TextureMap};

// Clockwise
#[derive(Clone, Debug, PartialEq)]
pub enum DecorationArea {
    TitleBar = 0,
    RightSideBar = 1,
    BottomSideBar = 2,
    LeftSideBar = 3,
    TopSideBar = 4,
}

pub struct WindowDecorator {
    canvas: Canvas,

    width: i32,
    height: i32,

    window_rect: Rectangle,
    content_rect: Rectangle,

    decoration_areas: [(DecorationArea, Rectangle); 5],
}

impl WindowDecorator {
    //pub const FONT_SIZE: f32 = 14.0;
    pub fn new(
        content_dimensions: (i32, i32),
        title_bar_height: i32,
        side_bar_thickness: i32,
    ) -> Self {
        // TODO: Find better name for these variables
        let (content_width, content_height) = content_dimensions;

        let win_width = content_width + side_bar_thickness * 2;
        let win_height = content_height + side_bar_thickness * 2 + title_bar_height;
        let content_x = side_bar_thickness;
        let content_y = title_bar_height;

        let bottom_bar_x = 0;
        let bottom_bar_y = side_bar_thickness + content_height + title_bar_height;

        let left_bar_x = 0;
        let left_bar_y = side_bar_thickness;

        let right_bar_x = side_bar_thickness + content_width;
        let right_bar_y = side_bar_thickness;

        let flip_rect_y = |y: i32, rect_height: i32| -> i32 { win_height - y - rect_height };

        let decoration_areas = [
            (
                DecorationArea::TitleBar,
                Rectangle::new(
                    side_bar_thickness as _,
                    flip_rect_y(side_bar_thickness, content_y) as _,
                    content_width as _,
                    content_y as _,
                ),
            ),
            (
                DecorationArea::RightSideBar,
                Rectangle::new(
                    right_bar_x as _,
                    flip_rect_y(right_bar_y, content_height + title_bar_height) as _,
                    side_bar_thickness as _,
                    (content_height + title_bar_height) as _,
                ),
            ),
            (
                DecorationArea::BottomSideBar,
                Rectangle::new(
                    bottom_bar_x as _,
                    flip_rect_y(bottom_bar_y, side_bar_thickness) as _,
                    win_width as _,
                    side_bar_thickness as _,
                ),
            ),
            (
                DecorationArea::LeftSideBar,
                Rectangle::new(
                    left_bar_x as _,
                    flip_rect_y(left_bar_y, content_height + title_bar_height) as _,
                    side_bar_thickness as _,
                    (content_height + title_bar_height) as _,
                ),
            ),
            (
                DecorationArea::TopSideBar,
                Rectangle::new(
                    0,
                    flip_rect_y(0, side_bar_thickness) as _,
                    win_width as _,
                    side_bar_thickness as _,
                ),
            ),
        ];

        Self {
            width: win_width,
            height: win_height,
            canvas: Canvas::new(win_width as _, win_height as _),

            decoration_areas,

            window_rect: Rectangle::new(0, 0, win_width as _, win_height as _),
            content_rect: Rectangle::new(
                content_x as _,
                content_y as _,
                content_width as _,
                content_height as _,
            ),
        }
    }

    pub fn inside_area(&self, x: i32, y: i32) -> Option<DecorationArea> {
        assert!(x > 0);
        assert!(y > 0);
        let invert_y = |y: i32| -> i32 { self.height - y - 1 };
        let y = invert_y(y);

        for (area_id, rect) in self.decoration_areas.iter() {
            if rect.contains(x as usize, y as usize) {
                return Some(area_id.clone());
            }
        }

        None
    }

    pub fn render(&mut self) {
        self.canvas.fill_color(Color::Transparent);
        for area in self.decoration_areas.iter() {
            let (_, rect) = area;
            self.canvas.draw_rect(rect.clone(), Color::Red);
        }
    }

    pub fn frame_as_bytes_slice(&self) -> &[u8] {
        self.canvas.as_bytes_slice()
    }
}
