use crate::draw::{
    Canva,
    Vec2,
    Color,
};

use std::ops::{
    Add,
    Div,
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
        Self::new([0., 0., 0., 0.])
    }

    pub
    fn as_vec2(&self) -> Vec2<f64> {
        Vec2::<f64>::new(self.a[0], self.a[1])
    }

    fn as_vec3(&self) -> Vec3 {
        Vec3::new([
            self.a[0], 
            self.a[1],
            self.a[2]
        ])
    }

}


#[derive(Debug, Copy, Clone)]
struct Vec3 {
    a: [f64; 3],
}

impl Vec3 {
    pub
    fn new (data: [f64; 3]) -> Self {
        Self {a: data}
    }

    pub
    fn zeros () -> Self {
        Self {a: [0.0; 3]}
    }

    pub
    fn as_vec2(&self) -> Vec2<f64> {
        Vec2::<f64>::new(self.a[0], self.a[1])
    }

    pub
    fn as_vec4(&self) -> Vec4 {
        Vec4::new([
            self.a[0], 
            self.a[1], 
            self.a[2], 
            1.0
        ])
    }

    pub
    fn norm (&self) -> f64 {
        let sum = self.a[0].powi(2) +
                  self.a[1].powi(2) +
                  self.a[2].powi(2);

        sum.sqrt()
    }

    pub
    fn cross(&self, arg: Self) -> Self {
        let a_0 = self.a[0];
        let a_1 = self.a[1];
        let a_2 = self.a[2];

        let b_0 = arg.a[0];
        let b_1 = arg.a[1];
        let b_2 = arg.a[2];

        let c_0 = (a_1 * b_2) - (a_2 * b_1);
        let c_1 = (a_2 * b_0) - (a_0 * b_2);
        let c_2 = (a_0 * b_1) - (a_1 * b_0);

        Self::new([c_0, c_1, c_2])
    }

    pub fn x (&self) -> f64 {self.a[0]}
    pub fn y (&self) -> f64 {self.a[1]}
    pub fn z (&self) -> f64 {self.a[2]}

}
impl Add for Vec4 {
    type Output = Self;

    fn add (self, rhs: Self) -> Self {
        Self {
            a: [
            self.a[0] + rhs.a[0],
            self.a[1] + rhs.a[1],
            self.a[2] + rhs.a[2],
            self.a[3] + rhs.a[3]
            ],
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul (self, rhs: f64) -> Self {
        Self {
            a: [
            self.a[0] * rhs,
            self.a[1] * rhs,
            self.a[2] * rhs
            ],
        }
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div (self, rhs: f64) -> Self {
        Self {
            a: [
            self.a[0] / rhs,
            self.a[1] / rhs,
            self.a[2] / rhs
            ],
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add (self, rhs: Self) -> Self {
        Self {
            a: [
            self.a[0] + rhs.a[0],
            self.a[1] + rhs.a[1],
            self.a[2] + rhs.a[2]
            ],
        }
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

    pub
    fn rotate_x (theta: f64) -> Self {
        let sin = theta.sin();
        let cos = theta.cos();

        Self::new([
            [1.,    0.,     0.,     0.],
            [0.,   cos,   -sin,     0.],
            [0.,   sin,    cos,     0.],
            [0.,    0.,     0.,     1.]
        ])
    }

    pub
    fn rotate_y (theta: f64) -> Self {
        let sin = theta.sin();
        let cos = theta.cos();

        Self::new([
            [cos,    0.,   sin,     0.],
            [0.,     1.,    0.,     0.],
            [-sin,   0.,   cos,     0.],
            [0.,     0.,    0.,     1.]
        ])
    }

    pub
    fn rotate_z (theta: f64) -> Self {
        let sin = theta.sin();
        let cos = theta.cos();

        Self::new([
            [cos,  -sin,    0.,     0.],
            [sin,   cos,    0.,     0.],
            [0.,     0.,    1.,     0.],
            [0.,     0.,    0.,     1.]
        ])
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

struct Triangle {
    points: [Vec3; 3],
    color: Color,
}

impl Triangle {
    pub
    fn new (points: [Vec3; 3]) -> Self {
        Self {
            points: points,
            color: Color::White,
        }
    }
}

struct Object {
    triangles: Vec<Triangle>,
}

impl Object {
    pub
    fn new (triangles: Vec<Triangle>) -> Self {
        Self {
            triangles: triangles,
        }
    }

    pub
    fn inv_piramid (bot: Vec3) -> Self {
        let vert_a = Vec3::new([-1. , 2.,  0.2]);
        let vert_b = Vec3::new([ 1. , 2.,  0.2]);
        let vert_c = Vec3::new([ 0. , 2., -1.]);

        let vert_o = Vec3::zeros();

        let base = Triangle::new([
                bot + vert_a,
                bot + vert_b,
                bot + vert_c
            ]);

        let f_a = Triangle::new([
                bot + vert_a,
                bot + vert_b,
                bot + vert_o
            ]);

        let f_b = Triangle::new([
                bot + vert_b,
                bot + vert_c,
                bot + vert_o
            ]);

        let f_c = Triangle::new([
                bot + vert_a,
                bot + vert_c,
                bot + vert_o
            ]);

        Self {
            triangles: vec![f_a, f_b, f_c, base],
        }
    }

}

struct Camera {
    position: Vec3,
    direction: Vec3,
}

impl Camera {
    pub
    fn new (pos: Vec3, dir: Vec3) -> Self {
        Self {
            position: pos,
            direction: dir,
        }
    }

    pub
    fn rotate_origin(&mut self, theta: f64) {

        let new_pos = Matrix4::rotate_y(theta.to_radians()) * 
                        self.position.as_vec4();
        let new_dir = Matrix4::rotate_y(theta.to_radians()) * 
                        self.direction.as_vec4();

        self.position = new_pos.as_vec3();
        self.direction = new_dir.as_vec3();

    }

    pub
    fn gen_matrix(&self) -> Matrix4 {
        let pos = self.position;
        let g = self.direction;
        let t = Vec3::new([0.,  1.,  0.]);

        let w = (g / g.norm()) * (-1.0);

        let t_x_w = t.cross(w);
        let u = t_x_w / t_x_w.norm();

        let v = w.cross(u);

        let M_cam_base = Matrix4::new([
            [u.x(), u.y(),  u.z(), 0.0],
            [v.x(), v.y(),  v.z(), 0.0],
            [w.x(), w.y(),  w.z(), 0.0],
            [0.0,     0.0,    0.0, 1.0]
        ]);

        let M_cam_pos = Matrix4::new([
            [1.0,   0.0,    0.0,   -pos.x()],
            [0.0,   1.0,    0.0,   -pos.y()],
            [0.0,   0.0,    1.0,   -pos.z()],
            [0.0,   0.0,    0.0,        1.0]
        ]);

        let M_cam = M_cam_base * M_cam_pos;

        M_cam
    }
}

pub
struct Scene {
    canva: Canva,
    width: usize,
    height: usize,
    camera: Camera,
    // objetos
    objects: Vec<Object>,
}

impl Scene {

    pub
    fn new (width: usize, height: usize) -> Self {
        let camera_pos = Vec3::new([0., 2., 4.]);
        let camera_dir = Vec3::new([0., -0.5, -1.]);

        Self {
            canva:   Canva::new(width, height),
            width:   width,
            height:  height,
            camera:  Camera::new(camera_pos, camera_dir),
            objects: vec![Object::inv_piramid(Vec3::zeros())],
        }
    }

    pub
    fn render (&mut self) {

        let n_x: f64 = self.width as _;
        let n_y: f64 = self.height as _;

        let mut M_vp = Matrix4::new([[n_x / 2.0, 0.0, 0.0, (n_x-1.0) / 2.0],
                                    [0.0, n_y / 2.0, 0.0, (n_y-1.0) / 2.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]]);
        
        let vec_test = Vec3::new([-1.0, -1.0, 0.0]);

        println!("M_viewpoint {:?}", M_vp);
        println!("vec_pos {:?}", vec_test);
        println!("vec_windowed {:?}", M_vp * vec_test.as_vec4());

        //assert!(false);


        let n: f64 = 10.0;
        let f: f64 = -10.0;

        let r: f64 = 10.0;
        let l: f64 = -10.0;

        let t: f64 = 10.0;
        let b: f64 = -10.0;

        assert!(n > f);
        assert!(r > l);
        assert!(t > b);

        let mut M_orth = Matrix4::new([[2.0 / (r-l),  0.0, 0.0, -(r+l) / (r-l)],
                                       [0.0,        2.0/(t-b), 0.0, -(t+b) / (t-b)],
                                       [0.0,        0.0, 2.0/(n-f), -(n+f) / (n-f)],
                                       [0.0, 0.0, 0.0, 1.0]]);

        let a_i = Vec3::new([1.0, 3.5, -4.0]);
        let b_i = Vec3::new([-1.0, 5.0, 0.0]);
        let mut M = M_vp * M_orth;
        let mut p_point = M * a_i.as_vec4();
        let mut q_point = M * b_i.as_vec4();

        /*
        println!("M_vp {:?}", M_vp);
        println!("{:?}", M);
        println!("p point {:?}", p_point);
        println!("q point {:?}", q_point);
        */

        //let a_point = Vec2::<f64>::new(p_point.x(), p_point.y());
        //let b_point = Vec2::<f64>::new(q_point.x(), q_point.y());
        //self.canva.draw_line(a_point, b_point);
        //
        /*
        let cam_pos = Vec3::new([0., 6., 5.]);
        let g = Vec3::new([0., -1., -1.]);
        let t = Vec3::new([0.,  1.,  0.]);

        let w = (g / g.norm()) * (-1.0);
        let t_x_w = t.cross(w);
            
        let u = t_x_w / t_x_w.norm();

        let v = w.cross(u);

        let M_cam_base = Matrix4::new([
            [u.x(), u.y(),  u.z(), 0.0],
            [v.x(), v.y(),  v.z(), 0.0],
            [w.x(), w.y(),  w.z(), 0.0],
            [0.0,     0.0,    0.0, 1.0]
        ]);

        let M_cam_pos = Matrix4::new([
            [1.0,   0.0,    0.0,   -cam_pos.x()],
            [0.0,   1.0,    0.0,   -cam_pos.y()],
            [0.0,   0.0,    1.0,   -cam_pos.z()],
            [0.0,   0.0,    0.0,            1.0]
        ]);
        */

        self.canva.clear();
        self.camera.rotate_origin(1.);

        let M_cam = self.camera.gen_matrix();

        for obj in self.objects.iter() {
            for tri in obj.triangles.iter() {
                let a  = (M * M_cam * tri.points[0].as_vec4()).as_vec2();
                let b  = (M * M_cam * tri.points[1].as_vec4()).as_vec2();
                let c  = (M * M_cam * tri.points[2].as_vec4()).as_vec2();

                /*
                println!("M_orth * vert {:?}",  M_orth * tri.points[0]);
                println!("M_vp M_orth * vert {:?}", M_vp * M_orth * tri.points[0]);
                println!("M_orth * vert {:?}",  M_orth * tri.points[1]);
                println!("M_vp M_orth * vert {:?}", M_vp * M_orth * tri.points[1]);
                println!("M_orth * vert {:?}", M_orth * tri.points[2]);
                println!("M_vp M_orth * vert {:?}", M_vp * M_orth * tri.points[2]);
                */

                self.canva.draw_triangle(a, b, c);
                self.canva.draw_line(a, b);
                self.canva.draw_line(b, c);
                self.canva.draw_line(a, c);
            }
        }
    }

    pub
    fn draw_objects (&mut self) {
    }

    pub
    fn frame_as_bytes_slice(&self) -> &[u8] {
        self.canva.as_bytes_slice()
    }

}
