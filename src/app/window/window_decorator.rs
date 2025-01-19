use crate::renderer::canvas::{Canvas, Color, Rectangle, VertexSimpleAttributes};
use crate::renderer::linalg::Vec2;
use crate::renderer::scene::{Texture, TextureMap};

// Clockwise
#[derive(Clone, Copy, Debug, PartialEq)]
enum Decoration {
    TitleBar = 1,
    RightSideBar = 2,
    BottomSideBar = 4,
    LeftSideBar = 8,
    TopSideBar = 16,
}

#[derive(Clone)]
struct DecorationIntersection(pub u32);

impl std::fmt::Display for DecorationIntersection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Intersection ({:#018b})", self.0)
    }
}

impl DecorationIntersection {
    fn empty() -> Self {
        Self(0)
    }

    fn add(&self, decoration: Decoration) -> Self {
        let mut new = self.clone();
        new.0 |= decoration as u32;
        new
    }

    /*
    fn contains(&self, decoration: Decoration) -> bool {
        self.0 & decoration as u32 != 0
    }
    */

    fn as_area(&self) -> DecorationArea {
        let top = Self::empty().add(Decoration::TopSideBar);
        let left = Self::empty().add(Decoration::LeftSideBar);
        let right = Self::empty().add(Decoration::RightSideBar);
        let bottom = Self::empty().add(Decoration::BottomSideBar);

        let top_left = top.add(Decoration::LeftSideBar);
        let top_right = top.add(Decoration::RightSideBar);
        let bottom_left = bottom.add(Decoration::LeftSideBar);
        let bottom_right = bottom.add(Decoration::RightSideBar);

        let title_bar = Self::empty().add(Decoration::TitleBar);

        if self.0 == title_bar.0 {
            return DecorationArea::TitleBar;
        }

        if self.0 == top.0 {
            return DecorationArea::Edge(WindowEdges::Top);
        }
        if self.0 == left.0 {
            return DecorationArea::Edge(WindowEdges::Left);
        }
        if self.0 == right.0 {
            return DecorationArea::Edge(WindowEdges::Right);
        }
        if self.0 == bottom.0 {
            return DecorationArea::Edge(WindowEdges::Bottom);
        }

        if self.0 == top_left.0 {
            return DecorationArea::Edge(WindowEdges::TopLeft);
        }
        if self.0 == top_right.0 {
            return DecorationArea::Edge(WindowEdges::TopRight);
        }
        if self.0 == bottom_left.0 {
            return DecorationArea::Edge(WindowEdges::BottomLeft);
        }
        if self.0 == bottom_right.0 {
            return DecorationArea::Edge(WindowEdges::BottomRight);
        }

        DecorationArea::None
    }
}

#[derive(Debug)]
pub enum WindowEdges {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

#[derive(Debug)]
pub enum DecorationArea {
    Edge(WindowEdges),
    TitleBar,
    None,
}

pub struct WindowDecorator {
    canvas: Canvas,

    width: i32,
    height: i32,
    title_bar_height: i32,
    side_bar_thickness: i32,

    window_rect: Rectangle,
    content_rect: Rectangle,

    decoration_areas: [(Decoration, Rectangle); 5],
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
        let left_bar_y = 0;
        //let left_bar_y = side_bar_thickness;

        let right_bar_x = side_bar_thickness + content_width;
        let right_bar_y = 0;
        //let right_bar_y = side_bar_thickness;

        let flip_rect_y = |y: i32, rect_height: i32| -> i32 { win_height - y - rect_height };

        let decoration_areas = [
            (
                Decoration::TitleBar,
                Rectangle::new(
                    side_bar_thickness as _,
                    flip_rect_y(side_bar_thickness, content_y) as _,
                    content_width as _,
                    content_y as _,
                ),
            ),
            (
                Decoration::RightSideBar,
                Rectangle::new(
                    right_bar_x as _,
                    flip_rect_y(right_bar_y, win_height) as _,
                    side_bar_thickness as _,
                    win_height as _,
                ),
            ),
            (
                Decoration::BottomSideBar,
                Rectangle::new(
                    bottom_bar_x as _,
                    flip_rect_y(bottom_bar_y, side_bar_thickness) as _,
                    win_width as _,
                    side_bar_thickness as _,
                ),
            ),
            (
                Decoration::LeftSideBar,
                Rectangle::new(
                    left_bar_x as _,
                    flip_rect_y(left_bar_y, win_height) as _,
                    side_bar_thickness as _,
                    win_height as _,
                ),
            ),
            (
                Decoration::TopSideBar,
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
            title_bar_height,
            side_bar_thickness,
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

    pub fn inside_area(&self, x: i32, y: i32) -> DecorationArea {
        // FIXME: We need to refactor canvas::Rectangle to be able
        // to use signed integers.
        if x < 0 || y < 0 {
            return DecorationArea::None;
        }

        let invert_y = |y: i32| -> i32 { self.height - y - 1 };
        let y = invert_y(y);

        let mut intersection = DecorationIntersection::empty();

        for (decoration, rect) in self.decoration_areas.iter() {
            if rect.contains(x as usize, y as usize) {
                println!("add {:?} as {}", decoration, *decoration as u32);
                intersection = intersection.add(*decoration);
                println!("intersec {}", intersection);
            }
        }

        let area = intersection.as_area();
        println!("{:#?}", self.decoration_areas);
        println!("{:?}", area);
        println!("pointer coords ({}x{})", x, y);

        area
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

    // FIXME: This is the same as the initialization function.
    // REWRITE!!
    pub fn resize_content(&mut self, content_dimensions: (i32, i32)) {
        // TODO: Find better name for these variables
        let (content_width, content_height) = content_dimensions;

        let win_width = content_width + self.side_bar_thickness * 2;
        let win_height = content_height + self.side_bar_thickness * 2 + self.title_bar_height;
        self.width = win_width;
        self.height = win_height;

        let content_x = self.side_bar_thickness;
        let content_y = self.title_bar_height;

        let bottom_bar_x = 0;
        let bottom_bar_y = self.side_bar_thickness + content_height + self.title_bar_height;

        let left_bar_x = 0;
        let left_bar_y = 0;
        //let left_bar_y = self.side_bar_thickness;

        let right_bar_x = self.side_bar_thickness + content_width;
        let right_bar_y = 0;
        //let right_bar_y = self.side_bar_thickness;

        let flip_rect_y = |y: i32, rect_height: i32| -> i32 { win_height - y - rect_height };

        self.decoration_areas = [
            (
                Decoration::TitleBar,
                Rectangle::new(
                    self.side_bar_thickness as _,
                    flip_rect_y(self.side_bar_thickness, content_y) as _,
                    content_width as _,
                    content_y as _,
                ),
            ),
            (
                Decoration::RightSideBar,
                Rectangle::new(
                    right_bar_x as _,
                    flip_rect_y(right_bar_y, win_height) as _,
                    self.side_bar_thickness as _,
                    win_height as _,
                ),
            ),
            (
                Decoration::BottomSideBar,
                Rectangle::new(
                    bottom_bar_x as _,
                    flip_rect_y(bottom_bar_y, self.side_bar_thickness) as _,
                    win_width as _,
                    self.side_bar_thickness as _,
                ),
            ),
            (
                Decoration::LeftSideBar,
                Rectangle::new(
                    left_bar_x as _,
                    flip_rect_y(left_bar_y, win_height) as _,
                    self.side_bar_thickness as _,
                    win_height as _,
                ),
            ),
            (
                Decoration::TopSideBar,
                Rectangle::new(
                    0,
                    flip_rect_y(0, self.side_bar_thickness) as _,
                    win_width as _,
                    self.side_bar_thickness as _,
                ),
            ),
        ];

        self.canvas = Canvas::new(win_width as _, win_height as _);

        self.window_rect = Rectangle::new(0, 0, win_width as _, win_height as _);
        self.content_rect = Rectangle::new(
            content_x as _,
            content_y as _,
            content_width as _,
            content_height as _,
        );
    }
}
