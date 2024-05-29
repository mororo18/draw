use crate::draw::canva::{
    Canva,
    Color,
};

use crate::draw::alglin::{
    Vec2,
    Vec3,
    Vec4,
    Matrix4,
};


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
                bot + vert_c,
                bot + vert_b,
                bot + vert_o
            ]);

        let f_c = Triangle::new([
                bot + vert_c,
                bot + vert_a,
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
    fn get_pos(&self) -> Vec3 {self.position}

    pub
    fn set_pos(&mut self, pos: Vec3) {
        self.position = pos;
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
        let camera_pos = Vec3::new([0., 2., 2.]);
        let camera_dir = Vec3::new([0., -0.5, -1.]);
        let mut canva = Canva::new(width, height);
        canva.enable_depth(20.0);

        Self {
            canva:   canva,
            width:   width,
            height:  height,
            camera:  Camera::new(camera_pos, camera_dir),
            objects: vec![Object::inv_piramid(Vec3::zeros())],
        }
    }

    pub
    fn camera_up(&mut self) {
        let cam_pos = self.camera.get_pos();
        self.camera.set_pos(cam_pos + Vec3::new([0., 0.05, 0.]));
    }

    pub
    fn camera_down(&mut self) {
        let cam_pos = self.camera.get_pos();
        self.camera.set_pos(cam_pos + Vec3::new([0., -0.05, 0.]));
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

        /*
        println!("M_viewpoint {:?}", M_vp);
        println!("vec_pos {:?}", vec_test);
        println!("vec_windowed {:?}", M_vp * vec_test.as_vec4());
        */

        //assert!(false);


        let n: f64 = 5.0;
        let f: f64 = -5.0;

        let r: f64 = 5.0;
        let l: f64 = -5.0;

        let t: f64 = 5.0;
        let b: f64 = -5.0;

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
        let camera_pos = self.camera.get_pos();

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

                let a_depth: f32 = camera_pos.dist(tri.points[0]) as _;
                let b_depth: f32 = camera_pos.dist(tri.points[1]) as _;
                let c_depth: f32 = camera_pos.dist(tri.points[2]) as _;

                self.canva.draw_triangle_with_depth(a, b, c, a_depth, b_depth, c_depth);
                //self.canva.draw_line(a, b);
                //self.canva.draw_line(b, c);
                //self.canva.draw_line(a, c);
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
