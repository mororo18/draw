use itertools::Either;
use std::ops::{Mul, Add, Sub};
use std::cmp::Ordering;

use crate::renderer::linalg::{
    Vec2,
    Vec3,
    Vec4,
};

// TODO: resolver dependencia cruzada :(
use crate::renderer::scene::Texture;

trait ColorOp {
    fn color_multiply(self, rhs: Self) -> Self;
}

impl ColorOp for Vec3 {
    fn color_multiply(self, rhs: Self) -> Self {
        Self::new([
            self.x() * rhs.x(),
            self.y() * rhs.y(),
            self.z() * rhs.z(),
        ])
    }
}


#[derive(Clone, Copy, Debug)]
pub
enum Color {
    White, 
    Black,

    Red,
    Green,
    Blue,

    Grey
}

impl Color {
    pub
    fn as_pixel(&self) -> Pixel {
        match self {
            Color::White => Pixel::white(),
            Color::Black => Pixel::black(),
            Color::Red   => Pixel::red(),
            Color::Green => Pixel::green(),
            Color::Blue  => Pixel::blue(),
            Color::Grey  => Pixel::new(128, 128, 128),
        }
    }

}

#[derive(Clone, Copy)]
struct Pixel {
    // little endian
    b:  u8,
    g:  u8,
    r:  u8,

    padd: u8,
}

// TODO: criar func "from_hex(cod: str)" e constantes com cores
impl Pixel {
    pub
    fn new (r: u8, g: u8, b: u8) -> Self {
        Self {r:r, g:g, b:b, padd: 0}
    }

    pub
    fn blend(a: Self, b: Self) -> Self {
        let norm_a = a.normalized_as_vec3();
        let norm_b = b.normalized_as_vec3();

        let blend = norm_a.color_multiply(norm_b);

        Self::from_normalized_vec3(blend)
    }

    pub
    fn normalized_as_vec3(&self) -> Vec3 {
        Vec3::new([
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ])
    }

    pub
    fn from_normalized_vec3(src: Vec3) -> Self {
        let scaled = src * 255.0;
        Self::new(
            scaled.x() as u8,
            scaled.y() as u8,
            scaled.z() as u8,
        )
    }

    pub
    fn as_vec4(&self) -> Vec4 {
        Vec4::new([
            self.b as f32,
            self.g as f32,
            self.r as f32,
            self.padd as f32
        ])
    }

    pub fn set_red   (&mut self, r: u8) {self.r = r;}
    pub fn set_green (&mut self, g: u8) {self.g = g;}
    pub fn set_blue  (&mut self, b: u8) {self.b = b;}
    pub fn set_padd  (&mut self, p: u8) {self.padd = p;}

    pub fn red  () -> Self {Pixel::new(255,0,0)}
    pub fn green() -> Self {Pixel::new(0,255,00)}
    pub fn blue () -> Self {Pixel::new(0,0,255)}
    pub fn white() -> Self {Pixel::new(255,255,255)}
    pub fn black() -> Self {Pixel::new(0,0,0)}
    pub fn azul_bb() -> Self {Pixel::new(155,186,255)}
}

impl Add for Pixel {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let r_sum = self.r + rhs.r;
        let g_sum = self.g + rhs.g;
        let b_sum = self.b + rhs.b;

        Pixel {
            r: r_sum,
            g: g_sum,
            b: b_sum,

            padd: 0,
        }
    }
}

impl Mul<f32> for Pixel {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let r = ((self.r as f32) * rhs) as u8;
        let g = ((self.g as f32) * rhs) as u8;
        let b = ((self.b as f32) * rhs) as u8;

        Pixel {
            r: r,
            g: g,
            b: b,
            padd: 0,
        }
    }
}

impl Mul<Pixel> for f32 {
    type Output = Pixel;

    fn mul(self, rhs: Pixel) -> Pixel {
        rhs * self 
    }
}

struct PixelPos {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone)]
pub
struct VertexAttributes {
    depth:  f32,
    color:  Color,
    pub screen_coord:   Vec2,
    normal:         Vec3,
    light:          Vec3,
    eye:            Vec3,
    texture_coord:  Vec3,
}

impl VertexAttributes {
    pub
    fn new (screen_coord: Vec2,
            color:  Color,
            depth:  f32,
            normal: Vec3,
            light:  Vec3,
            eye:    Vec3,
            txt_coord: Vec3) -> Self 
    {
        Self {
            screen_coord: screen_coord,
            color:  color,
            normal: normal,
            light:  light,
            eye:    eye,
            depth:  depth,
            texture_coord:  txt_coord,
        }
    }

    pub
    fn zeros () -> Self 
    {
        Self {
            screen_coord: Vec2::new(0., 0.),
            color:  Color::White,
            normal: Vec3::zeros(),
            light:  Vec3::zeros(),
            eye:    Vec3::zeros(),
            depth:  0.0,
            texture_coord:  Vec3::zeros(),
        }
    }
}

impl Add for VertexAttributes {
    type Output = Self;
    fn add (self, rhs: Self) -> Self {
        Self::new(
            self.screen_coord   + rhs.screen_coord,
            // TODO: tqv isso aq dps
            self.color, 
            self.depth          + rhs.depth,
            self.normal         + rhs.normal,
            self.light          + rhs.light,
            self.eye            + rhs.eye,
            self.texture_coord  + rhs.texture_coord,

        )
    }
}

impl Sub for VertexAttributes {
    type Output = Self;
    fn sub (self, rhs: Self) -> Self {
        Self::new(
            self.screen_coord   - rhs.screen_coord,
            // TODO: tqv isso aq dps
            self.color, 
            self.depth          - rhs.depth,
            self.normal         - rhs.normal,
            self.light          - rhs.light,
            self.eye            - rhs.eye,
            self.texture_coord  - rhs.texture_coord,

        )
    }
}

impl Mul<f32> for VertexAttributes {
    type Output = Self;
    fn mul (self, rhs: f32) -> Self {
        Self::new(
            self.screen_coord   * rhs,
            // TODO: tqv isso aq dps
            self.color, 
            self.depth          * rhs,
            self.normal         * rhs,
            self.light          * rhs,
            self.eye            * rhs,
            self.texture_coord  * rhs,

        )
    }
}


pub
struct Canvas {
    frame: Vec<Pixel>,
    width: usize,
    height: usize,

    depth_update_enabled: bool,
    depth_frame: Vec<f32>,

    depth_max:  f32,
}

impl Canvas {
    pub
    fn new (width: usize, height: usize) -> Self {
        let len: usize = width * height;
        let frame = vec![Pixel::black(); len];

        Self {
            frame: frame,
            width: width,
            height: height,

            depth_frame: vec![],
            depth_max: 0.0,
            depth_update_enabled: false,

        }
    }

    pub
    fn disable_depth_update(&mut self) {
        self.depth_update_enabled = false;
    }

    pub
    fn enable_depth_update(&mut self) {
        self.depth_update_enabled = true;
    }

    pub
    fn init_depth(&mut self, depth: f32) {
        self.depth_max = depth;

        if self.depth_frame.len() != self.frame.len() {
            self.depth_frame = vec![depth; self.frame.len()];

        } else if self.depth_frame.len() == self.frame.len() {
            self.depth_frame.iter_mut().for_each(|d| *d = depth);
        }
    }

    pub
    fn get_pixel_depth(&self, x: usize, y: usize) -> f32 {
        debug_assert!(self.in_bounds(x, y));
        unsafe{*self.depth_frame.get_unchecked(self.width * y + x)}
    }

    pub
    fn set_pixel_depth(&mut self, x: usize, y: usize, depth: f32) {
        debug_assert!(self.in_bounds(x, y));
        unsafe{*self.depth_frame.get_unchecked_mut(self.width * y + x) = depth;}
    }

    pub
    fn clear(&mut self) {
        self.frame.iter_mut().for_each(|pixel| *pixel = Pixel::azul_bb());

        if self.depth_frame.len() > 0 {
            self.init_depth(self.depth_max);
        }
    }

    pub
    fn draw_quadrilat(&mut self, a: Vec2, b: Vec2, c: Vec2, d: Vec2) {
        let a_center = self.pos_map_center(a);
        let b_center = self.pos_map_center(b);
        let c_center = self.pos_map_center(c);
        let d_center = self.pos_map_center(d);

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
    fn draw_triangle(&mut self, a_vertex: Vec2, b_vertex: Vec2, c_vertex: Vec2) {

        let a_center = self.pos_map_center(a_vertex);
        let b_center = self.pos_map_center(b_vertex);
        let c_center = self.pos_map_center(c_vertex);

        let color_a = Pixel::red();
        let color_b = Pixel::green();
        let color_c = Pixel::blue();

        let f_ab = |x: f32, y:f32| -> f32 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };

        let f_bc = |x: f32, y:f32| -> f32 {
            (b_center.y - c_center.y) * x + 
            (c_center.x - b_center.x) * y +
            (b_center.x * c_center.y) - 
            (c_center.x * b_center.y)
        };

        let f_ca = |x: f32, y:f32| -> f32 {
            (c_center.y - a_center.y) * x + 
            (a_center.x - c_center.x) * y +
            (c_center.x * a_center.y) - 
            (a_center.x * c_center.y)
        };

        let min = |x: f32, y: f32, z: f32| -> f32 {
            let mut ret = f32::INFINITY;
            vec![x, y, z].iter()
                .for_each(|v| if *v < ret {ret = *v;});

            ret
        };

        let max = |x: f32, y: f32, z: f32| -> f32 {
            let mut ret = -f32::INFINITY;
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

        let f_alpha_outside = f_bc(-1.0, -1.0);
        let f_beta_outside  = f_ca(-1.0, -1.0);
        let f_gama_outside  = f_ab(-1.0, -1.0);

        for y in y_min..=y_max {
            let y_f32 = y as f32;
            for x in x_min..=x_max {
                let x_f32 = x as f32;

                let alpha: f32 = f_bc(x_f32,y_f32) / f_alpha;
                let beta:  f32 = f_ca(x_f32,y_f32) / f_beta;
                let gama:  f32 = f_ab(x_f32,y_f32) / f_gama;

                if alpha >= 0.0 &&
                    beta >= 0.0 &&
                    gama >= 0.0 
                {
                    if (alpha > 0.0 || f_alpha * f_alpha_outside > 0.0) &&
                       (beta > 0.0  || f_beta  * f_beta_outside > 0.0) &&
                       (gama > 0.0  || f_gama  * f_gama_outside > 0.0)

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
    fn draw_triangle_with_attributes(&mut self,
                                   a_attr: VertexAttributes,
                                   b_attr: VertexAttributes,
                                   c_attr: VertexAttributes,
                                   texture: &Texture)
    {

        let a_center = self.pos_map_center(a_attr.screen_coord);
        let b_center = self.pos_map_center(b_attr.screen_coord);
        let c_center = self.pos_map_center(c_attr.screen_coord);

      //let a_pixel_color = a_attr.color.as_pixel();
      //let b_pixel_color = b_attr.color.as_pixel();
      //let c_pixel_color = c_attr.color.as_pixel();

        let a_depth = a_attr.depth;
        let b_depth = b_attr.depth;
        let c_depth = c_attr.depth;

        let f_ab = |x: f32, y:f32| -> f32 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };

        let f_bc = |x: f32, y:f32| -> f32 {
            (b_center.y - c_center.y) * x + 
            (c_center.x - b_center.x) * y +
            (b_center.x * c_center.y) - 
            (c_center.x * b_center.y)
        };

        let f_ca = |x: f32, y:f32| -> f32 {
            (c_center.y - a_center.y) * x + 
            (a_center.x - c_center.x) * y +
            (c_center.x * a_center.y) - 
            (a_center.x * c_center.y)
        };

        let min = |x: f32, y: f32, z: f32| -> f32 {
            let mut ret = f32::INFINITY;
            vec![x, y, z].iter()
                .for_each(|v| if *v < ret {ret = *v;});

            ret
        };

        let max = |x: f32, y: f32, z: f32| -> f32 {
            let mut ret = -f32::INFINITY;
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

        let f_alpha_outside = f_bc(-1.0, -1.0);
        let f_beta_outside  = f_ca(-1.0, -1.0);
        let f_gama_outside  = f_ab(-1.0, -1.0);

        for y in y_min..=y_max {
            let y_f32 = y as f32;
            for x in x_min..=x_max {
                let x_f32 = x as f32;

                let alpha: f32 = f_bc(x_f32,y_f32) / f_alpha;
                let beta:  f32 = f_ca(x_f32,y_f32) / f_beta;
                let gama:  f32 = f_ab(x_f32,y_f32) / f_gama;

                if alpha >= 0.0 &&
                    beta >= 0.0 &&
                    gama >= 0.0 
                {
                    if (alpha > 0.0 || f_alpha * f_alpha_outside > 0.0) &&
                       (beta > 0.0  || f_beta  * f_beta_outside > 0.0) &&
                       (gama > 0.0  || f_gama  * f_gama_outside > 0.0)

                    {





                        fn h_compute(light: Vec3, eye: Vec3) -> Vec3 {
                            let sum = light + eye;
                            sum.normalized()
                        }

                        fn cmp_max (a: f32, b: f32) -> f32 {
                            if a > b {return a;}
                            else     {return b;}
                        }


                        let pixel_depth = (alpha * a_depth) +
                                          (beta  * b_depth) +
                                          (gama  * c_depth);

                        /*
                        let pixel_color = (alpha * a_pixel_color) +
                                          (beta  * b_pixel_color) +
                                          (gama  * c_pixel_color);
                        */

                        // Texturasss ????
                        let pixel_texture_coord = 
                                          (a_attr.texture_coord * alpha) +
                                          (b_attr.texture_coord * beta) +
                                          (c_attr.texture_coord * gama);

                        let pixel_color_slice = texture.map_ka.get_rgb_slice(
                                                    pixel_texture_coord.x(),
                                                    pixel_texture_coord.y(),
                                                );
                        
                        let pixel_color: Vec3 = Pixel::new(
                                                pixel_color_slice[0],
                                                pixel_color_slice[1],
                                                pixel_color_slice[2],
                                            ).normalized_as_vec3();

                        // phong shadding ??

                        let pixel_normal = (a_attr.normal * alpha) +
                                           (b_attr.normal * beta) +
                                           (c_attr.normal * gama);

                        let pixel_light = (a_attr.light * alpha) +
                                          (b_attr.light * beta) +
                                          (c_attr.light * gama);

                        let pixel_eye = (a_attr.eye * alpha) +
                                        (b_attr.eye * beta) +
                                        (c_attr.eye * gama);

                        let pixel_halfway = h_compute(pixel_light, pixel_eye);

                        let power = 2;


                        let c_l = texture.ks;  // intensity term
                        let c_r = pixel_color * 0.5;  // diffuse reflectance
                        let c_a = pixel_color ;  // ambient term

                        //debug_assert!(c_l + c_a <= 1.0);

                        let color_normalized = c_r.color_multiply(c_a + c_l * (1.0 -  cmp_max(0.0 , pixel_light.dot(pixel_normal))) )
                                        + c_l * (pixel_halfway.dot(pixel_normal).powi(power));

                        let color = Pixel::from_normalized_vec3(color_normalized);

                        let pixel_opacity = texture.alpha;

                        self.draw_pixel_coord_with_depth(x, y, color, pixel_opacity, pixel_depth);

                    }
                }
            }
        }



    }

    pub
    fn draw_line(&mut self, a: Vec2, b: Vec2) {
        let a_center = self.pos_map_center(a);
        let b_center = self.pos_map_center(b);

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

        let midpoint_x: [f32; 4] = [
            0.5,
            1.0,
            1.0,
            0.5,
        ];

        let midpoint_y: [f32; 4] = [
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

        let f = |x: f32, y:f32| -> f32 {
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

        let delta_y = a_center.y - b_center.y;// *
        let delta_x = if idx == 0 || idx == 1 {b_center.x - a_center.x} else {-(b_center.x - a_center.x)};

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

    pub
    fn draw_dot (&mut self, pos: Vec2, color: Pixel) {
        let pixel_pos = self.img_map(pos);
        self.draw_pixel(pixel_pos, color);
    }

    pub
    fn draw_white_dot (&mut self, pos: Vec2) {
        let pixel_pos = self.img_map(pos);
        self.draw_pixel(pixel_pos, Pixel::white());
    }

    pub 
    fn pos_map_center (&self, pos: Vec2) -> Vec2 {
        let w_f32 = self.width  as f32 - 1.0;
        let h_f32 = self.height as f32 - 1.0;

        let x_center = (pos.x + 0.5).floor();
        let y_center = (pos.y + 0.5).floor();

        // Debug
        debug_assert!(
            x_center >= 0.0 && x_center <= w_f32 &&
            y_center >= 0.0 && y_center <= h_f32, 
            "posicao invalida ({x_center}, {y_center}) "
        );

        Vec2 {
            x: x_center,
            y: y_center,
        }
    }

    fn img_map (&self, pos: Vec2) -> PixelPos {
        let pos_center = self.pos_map_center(pos);
        PixelPos {
            x: pos_center.x as _,
            y: pos_center.y as _,
        }
    }

    fn draw_pixel_coord_with_depth (&mut self, x: usize, y: usize, color: Pixel, opacity: f32, depth: f32) {
        debug_assert!(self.depth_frame.len() > 0, "Depth not initialized");

        let new_color = if opacity < 1.0 {
            let bg_color = self.get_pixel_coord(x, y);
            bg_color * (1.0 - opacity) + color * opacity

        } else {
            color
        };

        if depth < self.get_pixel_depth(x, y) {
            self.draw_pixel_coord(x, y, new_color);

            if self.depth_update_enabled == true {
                self.set_pixel_depth(x, y, depth);
            }
        }
    }

    fn get_pixel_coord (&self, x: usize, y: usize) -> Pixel {
        debug_assert!(self.in_bounds(x, y),
                    "Drawing out of bounds. \
                    Point ({x}, {y}) doesnt fit ({0}, {1})",
                    self.width, self.height
        );

        let y_inv = self.height - y - 1;
        unsafe{*self.frame.get_unchecked(self.width * y_inv + x)}
    }

    pub
    fn draw_pixel_coord (&mut self, x: usize, y: usize, color: Pixel) {
        debug_assert!(self.in_bounds(x, y),
                    "Drawing out of bounds. \
                    Point ({x}, {y}) doesnt fit ({0}, {1})",
                    self.width, self.height
        );

        //println!("Drawing {x}, {y}");

        let y_inv = self.height - y - 1;
        unsafe{*self.frame.get_unchecked_mut(self.width * y_inv + x) = color;}
        //self.frame[self.width * y_inv + x] = color;
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

