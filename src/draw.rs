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

    fn in_bounds (&self, x: usize, y: usize) -> bool {
        x < self.height && y < self.width
    }

    pub
    fn draw_pixel (&mut self, x: usize, y: usize) {
        if self.in_bounds(x, y) == false {panic!("drawing out of bounds")}
        self.frame[self.width * x + y] = Pixel::white();
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

