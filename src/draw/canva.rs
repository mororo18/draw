use itertools::Either;
use std::ops::{Mul, Add};
use std::cmp;
use std::cmp::Ordering;

use crate::draw::alglin::{Vec2};

pub
enum Color {
    White,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy)]
struct Pixel {
    // little endian
    b:  u8,
    g:  u8,
    r:  u8,

    padd: u8,
}

impl Pixel {
    pub
    fn new (r: u8, g: u8, b: u8) -> Self {
        Pixel {r:r, g:g, b:b, padd: 255}
    }

    // TODO: criar func "from_hex(cod: str)" e constantes com cores

    pub fn set_red   (&mut self, r: u8) {self.r = r;}
    pub fn set_green (&mut self, g: u8) {self.g = g;}
    pub fn set_blue  (&mut self, b: u8) {self.b = b;}
    pub fn set_padd  (&mut self, p: u8) {self.padd = p;}

    pub fn red  () -> Self {Pixel::new(255,0,0)}
    pub fn green() -> Self {Pixel::new(0,255,00)}
    pub fn blue () -> Self {Pixel::new(0,0,255)}
    pub fn white() -> Self {Pixel::new(255,255,255)}
    pub fn black() -> Self {Pixel::new(0,0,0)}
}

impl Add for Pixel {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let r_sum = (self.r + rhs.r) as u32;
        let g_sum = (self.g + rhs.g) as u32;
        let b_sum = (self.b + rhs.b) as u32;

        Pixel {
            r: cmp::min(255, r_sum) as u8,
            g: cmp::min(255, g_sum) as u8,
            b: cmp::min(255, b_sum) as u8,

            padd: 0,
        }
    }
}

impl Mul<f64> for Pixel {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        let r = (self.r as f64) * rhs;
        let g = (self.g as f64) * rhs;
        let b = (self.b as f64) * rhs;

        Pixel {
            r: if r > 255.0 {255_u8} else {r as u8},
            g: if g > 255.0 {255_u8} else {g as u8},
            b: if b > 255.0 {255_u8} else {b as u8},

            padd: 0,
        }
    }
}

impl Mul<Pixel> for f64 {
    type Output = Pixel;

    fn mul(self, rhs: Pixel) -> Pixel {
        rhs * self 
    }
}

struct PixelPos {
    x: usize,
    y: usize,
}

pub
struct Canva {
    frame: Vec<Pixel>,
    width: usize,
    height: usize,

    depth_frame: Vec<f32>,
    depth_max:  f32,
}

impl Canva {
    pub
    fn new (width: usize, height: usize) -> Self {
        let len: usize = width * height;
        let frame = vec![Pixel::black(); len];

        Canva {
            frame: frame,
            width: width,
            height: height,

            depth_frame: vec![],
            depth_max: 0.0,
        }
    }

    pub
    fn enable_depth(&mut self, depth: f32) {
        self.depth_frame = vec![depth; self.frame.len()];
        self.depth_max = depth;
    }

    pub
    fn get_pixel_depth(&self, x: usize, y: usize) -> f32{
        assert!(self.in_bounds(x, y));
        self.depth_frame[self.width * y + x]
    }

    pub
    fn set_pixel_depth(&mut self, x: usize, y: usize, depth: f32) {
        assert!(self.in_bounds(x, y));
        self.depth_frame[self.width * y + x] = depth;
    }


    pub
    fn clear(&mut self) {
        let len: usize = self.width * self.height;
        self.frame = vec![Pixel::black(); len];

        if self.depth_frame.len() > 0 {
            self.enable_depth(self.depth_max);
        }
    }

    pub
    fn draw_quadrilat(&mut self, a: Vec2, b: Vec2, c: Vec2, d: Vec2) {
        let a_center = Self::pos_map_center(a);
        let b_center = Self::pos_map_center(b);
        let c_center = Self::pos_map_center(c);
        let d_center = Self::pos_map_center(d);

        let mut vertex_list = vec![a_center, b_center, c_center, d_center];
        vertex_list.as_mut_slice()
                    .sort_by(|a, b| 
                        if a.x > b.x {Ordering::Less}
                        else         {Ordering::Greater});

        let vertex_x_min = vertex_list.pop().expect("");
        vertex_list.as_mut_slice()
                    .sort_by_key(|v| (v.dist(vertex_x_min) * 100.0) as usize);
        let vertex_further = vertex_list.pop().expect("");

        let v_a = vertex_list[0];
        let v_b = vertex_list[1];

        self.draw_triangle(vertex_x_min, v_a, v_b);
        self.draw_triangle(vertex_further, v_a, v_b);
    }

    pub
    fn draw_triangle(&mut self, a: Vec2, b: Vec2, c: Vec2) {

        let a_center = Self::pos_map_center(a);
        let b_center = Self::pos_map_center(b);
        let c_center = Self::pos_map_center(c);

        let color_a = Pixel::red();
        let color_b = Pixel::green();
        let color_c = Pixel::blue();

        let f_ab = |x: f64, y:f64| -> f64 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };

        let f_bc = |x: f64, y:f64| -> f64 {
            (b_center.y - c_center.y) * x + 
            (c_center.x - b_center.x) * y +
            (b_center.x * c_center.y) - 
            (c_center.x * b_center.y)
        };

        let f_ca = |x: f64, y:f64| -> f64 {
            (c_center.y - a_center.y) * x + 
            (a_center.x - c_center.x) * y +
            (c_center.x * a_center.y) - 
            (a_center.x * c_center.y)
        };

        let min = |x: f64, y: f64, z: f64| -> f64 {
            let mut ret = f64::INFINITY;
            vec![x, y, z].iter()
                .for_each(|v| if *v < ret {ret = *v;});

            ret
        };

        let max = |x: f64, y: f64, z: f64| -> f64 {
            let mut ret = -f64::INFINITY;
            vec![x, y, z].iter()
                .for_each(|v| if *v > ret {ret = *v;});

            ret
        };

        let x_min = min(a_center.x, b_center.x, c_center.x) as usize;
        let y_min = min(a_center.y, b_center.y, c_center.y) as usize;

        let x_max = max(a_center.x, b_center.x, c_center.x) as usize;
        let y_max = max(a_center.y, b_center.y, c_center.y) as usize;

        let f_alpha = f_bc(a_center.x, a_center.y);
        let f_beta  = f_ca(b_center.x, b_center.y);
        let f_gama  = f_ab(c_center.x, c_center.y);

        for y in y_min..y_max {
            let y_f64 = y as f64;
            for x in x_min..x_max {
                let x_f64 = x as f64;

                let alpha: f64 = f_bc(x_f64,y_f64) / f_alpha;
                let beta:  f64 = f_ca(x_f64,y_f64) / f_beta;
                let gama:  f64 = f_ab(x_f64,y_f64) / f_gama;

                if alpha >= 0.0 &&
                    beta >= 0.0 &&
                    gama >= 0.0 
                {
                    if (alpha > 0.0 || f_alpha * f_bc(-1.0, -1.0) > 0.0) &&
                       (beta > 0.0  || f_beta  * f_ca(-1.0, -1.0) > 0.0) &&
                       (gama > 0.0  || f_gama  * f_ab(-1.0, -1.0) > 0.0)

                    {
                        let color_pixel = (alpha * color_a) +
                                          (beta  * color_b) +
                                          (gama  * color_c);

                        self.draw_pixel_coord(x, y, color_pixel);
                    }
                }
            }
        }


    }

    pub
    fn draw_triangle_with_depth(&mut self, a: Vec2, 
                                           b: Vec2, 
                                           c: Vec2,
                                           a_depth: f32,
                                           b_depth: f32,
                                           c_depth: f32)
    {

        let a_center = Self::pos_map_center(a);
        let b_center = Self::pos_map_center(b);
        let c_center = Self::pos_map_center(c);

        let color_a = Pixel::red();
        let color_b = Pixel::green();
        let color_c = Pixel::blue();

        let f_ab = |x: f64, y:f64| -> f64 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };

        let f_bc = |x: f64, y:f64| -> f64 {
            (b_center.y - c_center.y) * x + 
            (c_center.x - b_center.x) * y +
            (b_center.x * c_center.y) - 
            (c_center.x * b_center.y)
        };

        let f_ca = |x: f64, y:f64| -> f64 {
            (c_center.y - a_center.y) * x + 
            (a_center.x - c_center.x) * y +
            (c_center.x * a_center.y) - 
            (a_center.x * c_center.y)
        };

        let min = |x: f64, y: f64, z: f64| -> f64 {
            let mut ret = f64::INFINITY;
            vec![x, y, z].iter()
                .for_each(|v| if *v < ret {ret = *v;});

            ret
        };

        let max = |x: f64, y: f64, z: f64| -> f64 {
            let mut ret = -f64::INFINITY;
            vec![x, y, z].iter()
                .for_each(|v| if *v > ret {ret = *v;});

            ret
        };

        let x_min = min(a_center.x, b_center.x, c_center.x) as usize;
        let y_min = min(a_center.y, b_center.y, c_center.y) as usize;

        let x_max = max(a_center.x, b_center.x, c_center.x) as usize;
        let y_max = max(a_center.y, b_center.y, c_center.y) as usize;

        let f_alpha = f_bc(a_center.x, a_center.y);
        let f_beta  = f_ca(b_center.x, b_center.y);
        let f_gama  = f_ab(c_center.x, c_center.y);

        for y in y_min..y_max {
            let y_f64 = y as f64;
            for x in x_min..x_max {
                let x_f64 = x as f64;

                let alpha: f64 = f_bc(x_f64,y_f64) / f_alpha;
                let beta:  f64 = f_ca(x_f64,y_f64) / f_beta;
                let gama:  f64 = f_ab(x_f64,y_f64) / f_gama;

                if alpha >= 0.0 &&
                    beta >= 0.0 &&
                    gama >= 0.0 
                {
                    if (alpha > 0.0 || f_alpha * f_bc(-1.0, -1.0) > 0.0) &&
                       (beta > 0.0  || f_beta  * f_ca(-1.0, -1.0) > 0.0) &&
                       (gama > 0.0  || f_gama  * f_ab(-1.0, -1.0) > 0.0)

                    {
                        let color_pixel = (alpha * color_a) +
                                          (beta  * color_b) +
                                          (gama  * color_c);

                        let depth_pixel = (alpha * a_depth as f64) +
                                          (beta  * b_depth as f64) +
                                          (gama  * c_depth as f64);

                        self.draw_pixel_coord_with_depth(x, y, color_pixel, depth_pixel as _);
                    }
                }
            }
        }


    }

    pub
    fn draw_line(&mut self, a: Vec2, b: Vec2) {
        let a_center = Self::pos_map_center(a);
        let b_center = Self::pos_map_center(b);

        let m = (b_center.y - a_center.y) / (b_center.x - a_center.x);

        //println!("m = {m}");
        if 1.0 < m {
            self.midpoint_draw(a_center, b_center, 0);
        } else if 0.0 < m && m <= 1.0 {
            self.midpoint_draw(a_center, b_center, 1);
        } else if -1.0 < m && m <= 0.0 {
            self.midpoint_draw(a_center, b_center, 2);
        } else if m <= -1.0 {
            self.midpoint_draw(a_center, b_center, 3);
        }
    }

    fn midpoint_draw(&mut self, _a_center: Vec2, _b_center: Vec2, idx: usize) {
        // idx      m \in
        // ===================
        // 0        (1,   inf]      
        // 1        (0,     1]      
        // 2        (-1,    0]      
        // 3        (-inf, -1]      

        // Eh necessario que  a_center.x < b_center.x 
        //println!("idx = {idx}");

        let (a_center, b_center) = 
            match _a_center.x <= _b_center.x {
                true  => (_a_center, _b_center),
                false => (_b_center, _a_center),
            };

        let midpoint_x: [f64; 4] = [
            0.5,
            1.0,
            1.0,
            0.5,
        ];

        let midpoint_y: [f64; 4] = [
            1.0,
            0.5,
           -0.5,
           -1.0,
        ];


        let iter_axis_first: [_; 4] = [
            a_center.y as usize,
            a_center.x as usize,
            a_center.x as usize,
            a_center.y as usize
        ];

        let iter_axis_last: [_; 4] = [
            b_center.y as usize,
            b_center.x as usize,
            b_center.x as usize,
            b_center.y as usize
        ];

        let f = |x: f64, y:f64| -> f64 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };

        let inc_axis_begin = [
            a_center.x as i32,
            a_center.y as i32,
            a_center.y as i32,
            a_center.x as i32
        ];

        let increment: [i32; 4] = [
            1,
            1,
           -1,
            1,
        ];

        let mut inc_axis = inc_axis_begin[idx];
        let mut d = f(a_center.x + midpoint_x[idx],
                      a_center.y + midpoint_y[idx]); // alt

        let delta_y = (a_center.y - b_center.y);// *
            //if idx == 0 || idx == 3 {-1.0} else {1.0};
        let delta_x = if idx == 0 || idx == 1 {(b_center.x - a_center.x)} else {-(b_center.x - a_center.x)};

        fn iter_range(first: usize, last: usize, cond: bool) 
            -> Either<impl Iterator<Item = usize>, 
                      impl Iterator<Item = usize>> 
        {
            if cond     {Either::Right((last..=first).rev())}
            else        {Either::Left(first..=last)}
        }

        //println!("firs = {}\n last = {}", iter_axis_first[idx], iter_axis_last[idx]);
        // tem que arrumar essa gambiarra aq
        for iter_axis in iter_range(iter_axis_first[idx], iter_axis_last[idx], idx==3) {

            //println!("d = {d}, {idx}");

            let (x, y) = match idx {
                0 => (inc_axis, iter_axis as i32),
                1 => (iter_axis as i32, inc_axis),
                2 => (iter_axis as i32, inc_axis),
                3 => (inc_axis, iter_axis as i32),
                _ => (0, 0),
            };

            self.draw_pixel_coord(x as usize, y as usize, Pixel::white());

            let d_cond = match idx {
                0 => d > 0.0,
                1 => d < 0.0,
                2 => d > 0.0,
                3 => d < 0.0,
                _ => false,
            };

            if d_cond {
                inc_axis += increment[idx]; // alta
                d += delta_x  + delta_y; // alt
            } else {
                // necessita otimizacao (tqv no futuro)
                d += match idx {
                    0 => delta_x,
                    1 => delta_y,
                    2 => delta_y,
                    3 => delta_x,
                    _ => 0.0,
                };
            }
        }
    }
    // m \in (-1, 0]
    fn midpoint_draw_two(&mut self, a_center: Vec2, b_center: Vec2) {

        //assert!(a_center.x < b_center.x, "uso incorreto");
        //assert!(a_center.y > b_center.y, "uso incorreto");

        let f = |x: f64, y:f64| -> f64 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };

        let mut y = a_center.y as i32;
        let mut d = f(a_center.x + 1.0, a_center.y - 0.5); // alt

        let col_first = a_center.x as usize;
        let col_last  = b_center.x as usize;

        //println!("first {col_first}\nlast {col_last}");
        for x in col_first..=col_last {
            self.draw_pixel_coord(x,y as usize, Pixel::white());

            if d > 0.0 {
                y += -1; // alta
                d += - (b_center.x - a_center.x) + (a_center.y - b_center.y); // alt
            } else {
                d += a_center.y - b_center.y;
            }
        }
    }

    fn midpoint_draw_one(&mut self, a_center: Vec2, b_center: Vec2) {

        //assert!(a_center.x < b_center.x, "uso incorreto");
        //assert!(a_center.y < b_center.y, "uso incorreto");

        let f = |x: f64, y:f64| -> f64 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };


        let mut y = a_center.y as usize;
        let mut d = f(a_center.x + 1.0, a_center.y + 0.5);

        let col_first = a_center.x as usize;
        let col_last  = b_center.x as usize;

        for x in col_first..=col_last {
            self.draw_pixel_coord(x,y as usize, Pixel::white());

            if d < 0.0 {
                y += 1;
                d += (b_center.x - a_center.x) + (a_center.y - b_center.y);
            } else {
                d += a_center.y - b_center.y;
            }
        }
    }

    pub
    fn draw_dot (&mut self, pos: Vec2, color: Pixel) {
        let pixel_pos = Self::img_map(pos);
        self.draw_pixel(pixel_pos, color);
    }

    pub
    fn draw_white_dot (&mut self, pos: Vec2) {
        let pixel_pos = Self::img_map(pos);
        self.draw_pixel(pixel_pos, Pixel::white());
    }

    pub 
    fn pos_map_center (pos: Vec2) -> Vec2 {
        let mut x_center = (pos.x + 0.5).floor();
        let mut y_center = (pos.y + 0.5).floor();

        // Debug
        if x_center < -0.1 || y_center < -0.1 {
            panic!("posicao invalida ({x_center}, {y_center}) ");
        }

        x_center = if x_center < -0.5 {0.0} else {x_center};
        y_center = if y_center < -0.5 {0.0} else {y_center};
        
        Vec2 {
            x: x_center,
            y: y_center,
        }
    }

    pub
    fn img_map (pos: Vec2) -> PixelPos {
        let pos_center = Self::pos_map_center(pos);
        PixelPos {
            x: pos_center.x as _,
            y: pos_center.y as _,
        }
    }

    pub
    fn draw_pixel_coord_with_depth (&mut self, x: usize, y: usize, color: Pixel, depth: f32) {
        assert!(self.depth_frame.len() > 0, "Depth not initialized");

        if depth < self.get_pixel_depth(x, y) {
            self.draw_pixel_coord(x, y, color);
            self.set_pixel_depth(x, y, depth);
        }
    }

    pub
    fn draw_pixel_coord (&mut self, x: usize, y: usize, color: Pixel) {
        assert!(self.in_bounds(x, y),
                    "Drawing out of bounds. \
                    Point ({x}, {y}) doesnt fit ({0}, {1})",
                    self.width, self.height
        );

        //println!("Drawing {x}, {y}");

        let y_inv = self.height - y - 1;
        self.frame[self.width * y_inv + x] = color;
    }

    pub
    fn draw_pixel (&mut self, pos: PixelPos, color: Pixel) {
        self.draw_pixel_coord(pos.x, pos.y, color);
    }

    fn in_bounds (&self, x: usize, y: usize) -> bool {
        (x < self.width) && (y < self.height)
    }

    pub
    fn pixel_bytes() -> usize {std::mem::size_of::<Pixel>()}

    pub
    fn size_bytes(&self) -> usize {
        Self::pixel_bytes() * self.frame.len()
    }

    pub
    fn as_bytes_slice(&self) -> &[u8] {
        use std::slice;
        let ptr = self.frame.as_ptr().cast::<u8>();
        unsafe{slice::from_raw_parts(ptr, self.size_bytes())}
    }

    pub
    fn as_ptr(&self) -> *const u8 {self.frame.as_ptr().cast::<u8>()}
}

