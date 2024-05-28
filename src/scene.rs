use crate::draw::{
    Canva,
    Vec2,
};

use std::ops::{
    Mul,
};

#[derive(Debug, Copy, Clone)]
struct Vec4 {
    a: [f64; 4],
}

impl Vec4 {
    pub
    fn new (data: [f64; 4]) -> Self {
        Self {a: data}
    }

    pub
    fn zeros () -> Self {
        Self {a: [0.0; 4]}
    }
}

#[derive(Debug, Copy, Clone)]
struct Matrix4 {
    a: [[f64; 4]; 4],
}

impl Matrix4 {
    pub
    fn new (data: [[f64; 4]; 4]) -> Self {
        Self {a: data}
    }

    pub
    fn zeros () -> Self {
        Self {a: [[0.0; 4]; 4]}
    }
}

// TODO: ver ->
// https://medium.com/@kilichbekhaydarov/toward-an-optimal-matrix-multiplication-algorithm-4f024baa1206
impl Mul for Matrix4 {
    type Output = Self;

    fn mul (self, rhs: Self) -> Self {
        let mut c = Self::zeros();

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    c.a[i][j] += self.a[i][k] * rhs.a[k][j];
                }
            }
        }

        c
    }
}

impl Mul<Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul (self, rhs: Vec4) -> Vec4 {
        let mut out = Vec4::zeros();

        for i in 0..4 {
            for j in 0..4 {
                out.a[i] += self.a[i][j] * rhs.a[j];
            }
        }

        out
    }
}


pub
struct Scene {
    canva: Canva,
    width: usize,
    height: usize,
    // camera
    // objetos
}

impl Scene {

    pub
    fn new (width: usize, height: usize) -> Self {
        Self {
            canva: Canva::new(width, height),
            width: width,
            height: height,
        }
    }

    pub
    fn render (&mut self) {
        let a = Vec2::<f64>::new(0.0, 0.0);
        let b = Vec2::<f64>::new(750.0, 250.0);

        let n_x: f64 = self.width as _;
        let n_y: f64 = self.height as _;

        let mut M_vp = Matrix4::new([[n_x / 2.0, 0.0, 0.0, (n_x-1.) / 2.0],
                                    [0.0, n_y / 2.0, 0.0, (n_y-1.) / 20.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]]);
        
        let mut M_orth = Matrix4::new([[n_x / 2.0, 0.0, 0.0, (n_x-1.0) / 2.0],
                                    [0.0, n_y / 2.0, 0.0, (n_y-1.) / 20.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]]);

        let vec_test = Vec4::new([0.0, 1.0, 625326.23, 98.3]);
        let mut M_ret = M_vp * M_orth;
        let mut V_ret = M_vp * vec_test;

        println!("{:?}", M_ret);
        println!("{:?}", V_ret);
        self.canva.draw_line(a, b);
    }

    pub
    fn frame_as_bytes_slice(&self) -> &[u8] {
        self.canva.as_bytes_slice()
    }

}
