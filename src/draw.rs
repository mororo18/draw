#![feature(ascii_char)] 

enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone)]
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
        Pixel {r:r, g:g, b:b, padd:0}
    }

    // TODO: criar func "from_hex(cod: str)" e constantes com cores

    pub
    fn set_red (&mut self, r: u8) {self.r = r;}

    pub
    fn set_green (&mut self, g: u8) {self.g = g;}

    pub
    fn set_blue (&mut self, b: u8) {self.b = b;}

    pub
    fn red() -> Self {Pixel::new(255,0,0)}

    pub
    fn green() -> Self {Pixel::new(0,255,00)}

    pub
    fn blue() -> Self {Pixel::new(0,0,255)}

    pub
    fn white() -> Self {Pixel::new(255,255,255)}

    pub
    fn black() -> Self {Pixel::new(0,0,0)}
}

#[derive(Debug, Copy, Clone)]
pub
struct Vec2<T> {
    x: T,
    y: T,
}

impl<T> Vec2<T> {
    pub
    fn new (x: T, y: T) -> Self {Vec2 {x:x, y:y}}
}

type PixelPos = Vec2<usize>;

pub
struct Canva {
    frame: Vec<Pixel>,
    width: usize,
    height: usize,
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
        }
    }

    pub
    fn draw_line(&mut self, a: Vec2<f64>, b: Vec2<f64>) {
        let a_center = Self::pos_map_center(a);
        let b_center = Self::pos_map_center(b);

        let m = (b_center.y - a_center.y) / (b_center.x - a_center.x);

        let f = |x: f64, y:f64| -> f64 {
            (a_center.y - b_center.y) * x + 
            (b_center.x - a_center.x) * y +
            (a_center.x * b_center.y) - 
            (b_center.x * a_center.y)
        };


        println!("m = {m}");
        if 0.0 < m && m <= 1.0 {
            self.midpoint_draw(a_center, b_center, f);
            println!("midpoint");
        }
    }

    fn midpoint_draw<F>(&mut self, a: Vec2<f64>, b: Vec2<f64>, mut f:  F ) 
    where
        F: FnMut(f64, f64) -> f64,
    {
        let mut y = a.y as usize;
        let mut d = f(a.x + 1.0, a.y + 0.5);

        let col_first = a.x as usize;
        let col_last  = b.x as usize;

        for x in col_first..=col_last {
            self.draw_pixel_coord(x,y);

            if d < 0.0 {
                y += 1;
                d += (b.x - a.x) + (a.y - b.y);
            } else {
                d += a.y - b.y;
            }
        }
    }

    pub
    fn draw_dot (&mut self, pos: Vec2<f64>) {
        let pixel_pos = Self::img_map(pos);
        self.draw_pixel(pixel_pos);
    }

    pub 
    fn pos_map_center (pos: Vec2<f64>) -> Vec2<f64> {
        let mut x_center = (pos.x + 0.5).floor();
        let mut y_center = (pos.y + 0.5).floor();

        // Debug
        if x_center < -0.1 || y_center < -0.1 {panic!("posicao invalida")}

        x_center = if x_center < -0.5 {0.0} else {x_center};
        y_center = if y_center < -0.5 {0.0} else {y_center};
        
        Vec2 {
            x: x_center,
            y: y_center,
        }
    }

    pub
    fn img_map (pos: Vec2<f64>) -> PixelPos {
        let pos_center = Self::pos_map_center(pos);
        PixelPos {
            x: pos_center.x as _,
            y: pos_center.y as _,
        }
    }

    pub
    fn draw_pixel_coord (&mut self, x: usize, y: usize) {
        if self.in_bounds(x, y) == false {panic!("drawing out of bounds")}

        let y_inv = self.height - y - 1;
        self.frame[self.width * y_inv + x] = Pixel::white();
    }

    pub
    fn draw_pixel (&mut self, pos: PixelPos) {
        self.draw_pixel_coord(pos.x, pos.y);
    }

    fn in_bounds (&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
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

