use crate::renderer::canvas::{Canvas, Color, Rectangle, VertexSimpleAttributes};
use crate::renderer::linalg::Vec2;
use crate::renderer::scene::{Texture, TextureMap};

use more_asserts::*;

// TODO: Maybe use this crate
// https://docs.rs/enumflags2/latest/enumflags2/
// Clockwise
#[derive(Clone, Copy, Debug, PartialEq)]
enum DecorationBitFlag {
    TitleBar = 1,
    RightSideBar = 2,
    BottomSideBar = 4,
    LeftSideBar = 8,
    TopSideBar = 16,
    CloseButton = 32,
    MaximizeButton = 64,
    MinimizeButton = 128,
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

    fn add(&self, decoration: DecorationBitFlag) -> Self {
        let mut new = self.clone();
        new.0 |= decoration as u32;
        new
    }

    /*
    fn contains(&self, decoration: DecorationBitFlag) -> bool {
        self.0 & decoration as u32 != 0
    }
    */

    fn as_area(&self) -> DecorationArea {
        let top = Self::empty().add(DecorationBitFlag::TopSideBar);
        let left = Self::empty().add(DecorationBitFlag::LeftSideBar);
        let right = Self::empty().add(DecorationBitFlag::RightSideBar);
        let bottom = Self::empty().add(DecorationBitFlag::BottomSideBar);

        let top_left = top.add(DecorationBitFlag::LeftSideBar);
        let top_right = top.add(DecorationBitFlag::RightSideBar);
        let bottom_left = bottom.add(DecorationBitFlag::LeftSideBar);
        let bottom_right = bottom.add(DecorationBitFlag::RightSideBar);

        let title_bar = Self::empty().add(DecorationBitFlag::TitleBar);
        let close_btn = title_bar.add(DecorationBitFlag::CloseButton);
        let max_btn = title_bar.add(DecorationBitFlag::MaximizeButton);
        let min_btn = title_bar.add(DecorationBitFlag::MinimizeButton);

        if self.0 == title_bar.0 {
            return DecorationArea::TitleBar;
        }

        if self.0 == close_btn.0 {
            return DecorationArea::CloseButton;
        }
        if self.0 == max_btn.0 {
            return DecorationArea::MaximizeButton;
        }
        if self.0 == min_btn.0 {
            return DecorationArea::MinimizeButton;
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

struct Decoration {
    id: DecorationBitFlag,
    rect: Rectangle,
    visibility: bool,
}

impl Decoration {
    pub fn new(id: DecorationBitFlag, rect: Rectangle) -> Self {
        Self {
            id,
            rect,
            visibility: true,
        }
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
    CloseButton,
    MaximizeButton,
    MinimizeButton,
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

    decorations: Vec<Decoration>,

    is_maximized: bool,
}

impl WindowDecorator {
    pub const BUTTON_RADIUS: i32 = 10;
    pub fn new(
        content_dimensions: (i32, i32),
        title_bar_height: i32,
        side_bar_thickness: i32,
    ) -> Self {

        // TODO: Find better name for these variables
        let (content_width, content_height) = content_dimensions;
        let content_x = side_bar_thickness;
        let content_y = title_bar_height;

        let win_width = content_width + side_bar_thickness * 2;
        let win_height = content_height + side_bar_thickness * 2 + title_bar_height;

        let flip_rect_y = |y: i32, rect_height: i32| -> i32 {
            assert!(win_height - y - rect_height >= 0);
            win_height - y - rect_height
        };

        let vertical_bar_width = side_bar_thickness;
        let vertical_bar_height = win_height;

        let horizontal_bar_width = win_width;
        let horizontal_bar_height = side_bar_thickness;

        let title_bar_x = 0;
        let title_bar_y = 0;
        let title_bar_width = content_width;

        let title_bar = Decoration::new(
            DecorationBitFlag::TitleBar,
            Rectangle::new(
                title_bar_x,
                flip_rect_y(title_bar_y, title_bar_height),
                title_bar_width,
                title_bar_height,
            )
        );


        let top_bar_x = 0;
        let top_bar_y = 0;

        let top_side_bar = Decoration::new(
            DecorationBitFlag::TopSideBar,
            Rectangle::new(
                top_bar_x,
                flip_rect_y(top_bar_y, horizontal_bar_height),
                horizontal_bar_width,
                horizontal_bar_height,
            ),
        );

        let bottom_bar_x = 0;
        let bottom_bar_y = side_bar_thickness + content_height + title_bar_height;
        dbg!(bottom_bar_y);

        let bottom_side_bar = Decoration::new(
            DecorationBitFlag::BottomSideBar,
            Rectangle::new(
                bottom_bar_x,
                dbg!(flip_rect_y(bottom_bar_y, horizontal_bar_height)),
                horizontal_bar_width,
                horizontal_bar_height,
            ),
        );

        let left_bar_x = 0;
        let left_bar_y = 0;

        let left_side_bar = Decoration::new(
            DecorationBitFlag::LeftSideBar,
            Rectangle::new(
                left_bar_x as _,
                flip_rect_y(left_bar_y, vertical_bar_height) as _,
                vertical_bar_width as _,
                vertical_bar_height as _,
            ),
        );

        let right_bar_x = side_bar_thickness + content_width;
        let right_bar_y = 0;

        let right_side_bar = Decoration::new(
            DecorationBitFlag::RightSideBar,
            Rectangle::new(
                right_bar_x as _,
                flip_rect_y(right_bar_y, vertical_bar_height) as _,
                vertical_bar_width as _,
                vertical_bar_height as _,
            ),
        );

        let mut buttons = Self::make_buttons(&title_bar);
        let mut decorations = vec![
            title_bar,
            top_side_bar,
            bottom_side_bar,
            right_side_bar,
            left_side_bar,
        ];

        decorations.append(&mut buttons);

        Self {
            width: win_width,
            height: win_height,
            title_bar_height,
            side_bar_thickness,
            canvas: Canvas::new(win_width as _, win_height as _),

            decorations,

            is_maximized: false,

            window_rect: Rectangle::new(0, 0, win_width as _, win_height as _),
            content_rect: Rectangle::new(
                content_x as _,
                content_y as _,
                content_width as _,
                content_height as _,
            ),
        }
    }

    fn make_buttons(title_bar: &Decoration) -> Vec<Decoration> {
        assert_eq!(title_bar.id, DecorationBitFlag::TitleBar);

        const SPACE_BETWEEN: i32 = 20;
        let button_diameter = 2 * Self::BUTTON_RADIUS;

        let buttons_y = title_bar.rect.y + (title_bar.rect.height / 2) - Self::BUTTON_RADIUS;

        let title_bar_end_x = title_bar.rect.x_max();

        let close_x = title_bar_end_x - SPACE_BETWEEN - button_diameter;
        let maximize_x = close_x - SPACE_BETWEEN - button_diameter;
        let minimize_x = maximize_x - SPACE_BETWEEN - button_diameter;

        let close_button = Decoration::new(
            DecorationBitFlag::CloseButton,
            Rectangle::new(
                close_x,
                buttons_y,
                button_diameter,
                button_diameter
            )
        );

        let maximize_button = Decoration::new(
            DecorationBitFlag::MaximizeButton,
            Rectangle::new(
                maximize_x,
                buttons_y,
                button_diameter,
                button_diameter
            )
        );

        let minimize_button = Decoration::new(
            DecorationBitFlag::MinimizeButton,
            Rectangle::new(
                minimize_x,
                buttons_y,
                button_diameter,
                button_diameter
            )
        );

        vec![close_button, maximize_button, minimize_button]
    }

    fn get_decoration_mut(&mut self, decoration_id: DecorationBitFlag) -> Option<&mut Decoration> {
        for decoration in self.decorations.iter_mut() {
            if decoration.id == decoration_id {
                return Some(decoration);
            }
        }

        None
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

        for decoration in self.decorations.iter() {
            if decoration.rect.contains(x, y) && decoration.visibility {
                intersection = intersection.add(decoration.id);
            }
        }

        intersection.as_area()
    }

    pub fn render(&mut self) {
        self.canvas.fill_color(Color::Transparent);
        for decoration in self.decorations.iter() {
            if decoration.visibility {
                self.canvas.draw_rect(decoration.rect.clone(), Color::Red);
            }
        }
    }

    pub fn frame_as_bytes_slice(&self) -> &[u8] {
        self.canvas.as_bytes_slice()
    }

    // FIXME: thes method doesnt make sense
    pub fn set_maximized(&mut self, is_maximized: bool) {
        self.is_maximized = is_maximized;
    }

    // FIXME: This is the same as the initialization function.
    // REWRITE!!
    pub fn resize_window_frame(&mut self, width: i32, height: i32) {

        if width < 0 || height < 0 {
            return;
        }

        // TODO: Find better name for these variables
        let (content_x,
            content_y,
            content_width,
            content_height) = if self.is_maximized {
            (
                0,
                self.title_bar_height,
                width,
                height - self.title_bar_height
            )
        } else {
            (
                self.side_bar_thickness, 
                self.title_bar_height + self.side_bar_thickness,
                width - self.side_bar_thickness * 2,
                height - (self.side_bar_thickness * 2 + self.title_bar_height)
            )
        };

        self.width = width;
        self.height = height;

        let flip_rect_y = |y: i32, rect_height: i32| -> i32 {
            //assert_ge!(height - y - rect_height, 0);
            height - y - rect_height
        };

        let vertical_bar_width = self.side_bar_thickness;
        let vertical_bar_height = height;

        let horizontal_bar_width = width;
        let horizontal_bar_height = self.side_bar_thickness;

        let title_bar_x = content_x;
        let title_bar_y = content_y - self.title_bar_height;
        let title_bar_width = content_width;

        let title_bar = Decoration::new(
            DecorationBitFlag::TitleBar,
            Rectangle::new(
                title_bar_x as _,
                flip_rect_y(title_bar_y, self.title_bar_height) as _,
                title_bar_width as _,
                self.title_bar_height as _,
            )
        );


        let top_bar_x = 0;
        let top_bar_y = 0;

        let mut top_side_bar = Decoration::new(
            DecorationBitFlag::TopSideBar,
            Rectangle::new(
                top_bar_x,
                flip_rect_y(top_bar_y, horizontal_bar_height) as _,
                horizontal_bar_width as _,
                horizontal_bar_height as _,
            ),
        );

        let bottom_bar_x = 0;
        let bottom_bar_y = self.side_bar_thickness + content_height + self.title_bar_height;

        let mut bottom_side_bar = Decoration::new(
            DecorationBitFlag::BottomSideBar,
            Rectangle::new(
                bottom_bar_x as _,
                flip_rect_y(bottom_bar_y, horizontal_bar_height) ,
                horizontal_bar_width as _,
                horizontal_bar_height as _,
            ),
        );

        let left_bar_x = 0;
        let left_bar_y = 0;

        let mut left_side_bar = Decoration::new(
            DecorationBitFlag::LeftSideBar,
            Rectangle::new(
                left_bar_x as _,
                flip_rect_y(left_bar_y, vertical_bar_height) as _,
                vertical_bar_width as _,
                vertical_bar_height as _,
            ),
        );

        let right_bar_x = self.side_bar_thickness + content_width;
        let right_bar_y = 0;

        let mut right_side_bar = Decoration::new(
            DecorationBitFlag::RightSideBar,
            Rectangle::new(
                right_bar_x as _,
                flip_rect_y(right_bar_y, vertical_bar_height) as _,
                vertical_bar_width as _,
                vertical_bar_height as _,
            ),
        );

        if self.is_maximized {
            top_side_bar.visibility = false;
            bottom_side_bar.visibility = false;
            left_side_bar.visibility = false;
            right_side_bar.visibility = false;
        }

        let mut buttons = Self::make_buttons(&title_bar);
        self.decorations = vec![
            title_bar,
            top_side_bar,
            bottom_side_bar,
            left_side_bar,
            right_side_bar,
        ];

        self.decorations.append(&mut buttons);

        self.canvas = Canvas::new(width as _, height as _);

        self.window_rect = Rectangle::new(0, 0, width as _, height as _);
        self.content_rect = Rectangle::new(
            content_x as _,
            content_y as _,
            content_width as _,
            content_height as _,
        );
    }
}
