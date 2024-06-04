use crate::draw::canva::{
    Canva,
    Color,
};

use crate::draw::linalg::{
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
        let height: f64 = 3.0;
        let side = 3.0_f64;
        let l = (side / 2.0) * (2.0 / f64::sqrt(3.0));

        let c_x = 0.0_f64;
        let c_z = l;

        let a_x = (side / 2.0);
        let a_z = -l / 2.0;

        let b_x = -(side / 2.0);
        let b_z = -l / 2.0;

        let vert_a = Vec3::new([a_x, height, a_z]);
        let vert_b = Vec3::new([b_x, height, b_z]);
        let vert_c = Vec3::new([c_x, height, c_z]);
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

    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    pub
    fn new (pos: Vec3, dir: Vec3) -> Self {
        Self {
            position: pos,
            direction: dir,
            u: Vec3::zeros(),
            v: Vec3::zeros(),
            w: Vec3::zeros(),
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
    fn get_matrix_base(&self) -> Matrix4 {
        let u = self.u;
        let v = self.v;
        let w = self.w;

        Matrix4::new([
            [u.x(), u.y(),  u.z(), 0.0],
            [v.x(), v.y(),  v.z(), 0.0],
            [w.x(), w.y(),  w.z(), 0.0],
            [0.0,     0.0,    0.0, 1.0]
        ])
    }

    pub
    fn gen_matrix(&mut self) -> Matrix4 {
        let pos = self.position;
        let g = self.direction;
        let t = Vec3::new([0.,  1.,  0.]);

        let w = (g / g.norm()) * (-1.0);

        let t_x_w = t.cross(w);
        let u = t_x_w / t_x_w.norm();

        let v = w.cross(u);

        self.u = u;
        self.v = v;
        self.w = w;

        let M_cam_base = self.get_matrix_base();

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

    fn gen_transform_matrix(&mut self) -> Matrix4 {
        let n_x: f64 = self.width as _;
        let n_y: f64 = self.height as _;

        let M_viewport = Matrix4::new([
            [n_x / 2.0,        0.0,  0.0,  (n_x-1.0) / 2.0],
            [      0.0,  n_y / 2.0,  0.0,  (n_y-1.0) / 2.0],
            [      0.0,        0.0,  1.0,              0.0],
            [      0.0,        0.0,  0.0,              1.0]
        ]);

        let n: f64 = 5.0;
        let f: f64 = -5.0;

        let r: f64 = 5.0;
        let l: f64 = -5.0;

        let t: f64 = 5.0;
        let b: f64 = -5.0;

        assert!(n > f);
        assert!(r > l);
        assert!(t > b);

        let M_orth = Matrix4::new([
            [2.0 / (r-l),          0.0,          0.0,  -(r+l) / (r-l)],
            [        0.0,  2.0 / (t-b),          0.0,  -(t+b) / (t-b)],
            [        0.0,          0.0,  2.0 / (n-f),  -(n+f) / (n-f)],
            [        0.0,          0.0,          0.0,             1.0]
        ]);

        let M_cam = self.camera.gen_matrix();

        let P = Matrix4::new([
            [   -n,  0.0,       0.0,      0.0],
            [  0.0,   -n,       0.0,      0.0],
            [  0.0,  0.0,  -(n + f),  (n * f)],
            [  0.0,  0.0,       1.0,      0.0]
        ]);

        let M = M_viewport * M_orth * M_cam;
        //let M = M_viewport * M_orth * P * M_cam;

        M
    }

    pub
    fn render (&mut self) {


        self.canva.clear();
        self.camera.rotate_origin(1.);
        let camera_pos = self.camera.get_pos();

        let M = self.gen_transform_matrix();
        let M_cam = self.camera.get_matrix_base().transposed();


        let n: f64 = 5.0;
        let f: f64 = -5.0;

        let r: f64 = 5.0;
        let l: f64 = -5.0;

        let t: f64 = 5.0;
        let b: f64 = -5.0;

        assert!(n > f);
        assert!(r > l);
        assert!(t > b);

        // nearest face of the transformed view volume
        let A = (M_cam * Vec3::new([ r, t, n]).as_vec4()).as_vec3() + camera_pos;      // direita superior frente
        let B = (M_cam * Vec3::new([ r, b, n]).as_vec4()).as_vec3() + camera_pos;
        let C = (M_cam * Vec3::new([ l, b, n]).as_vec4()).as_vec3() + camera_pos;
        let D = (M_cam * Vec3::new([ l, t, n]).as_vec4()).as_vec3() + camera_pos;

        // furtherest face of the transformed view volume
        let E = (M_cam * Vec3::new([ r, t, f]).as_vec4()).as_vec3() + camera_pos;
        let F = (M_cam * Vec3::new([ r, b, f]).as_vec4()).as_vec3() + camera_pos;
        let G = (M_cam * Vec3::new([ l, b, f]).as_vec4()).as_vec3() + camera_pos;
        let H = (M_cam * Vec3::new([ l, t, f]).as_vec4()).as_vec3() + camera_pos;

        let test_point = (A + B + C + D + E + F + G + H) / 8.0;;
        //let test_point = (M_cam * Vec3::zeros().as_vec4()).as_vec3() + camera_pos;

        println!("test_point {:?} ", test_point);


        fn get_plane_eq (A: Vec3, B: Vec3, C: Vec3, test_point: Vec3) -> impl FnMut(Vec3) -> f64 {
            let p_vec = B - A;
            let q_vec = C - B;

            let normal = p_vec.cross(q_vec);

            let k = - normal.dot(A);

            let func = move |point: Vec3| -> f64 {normal.dot(point) + k};

            // a condicao de validez eh que a origem gere um valor positivo,
            // ou seja, ela esta dentro do volume de visao
            
            if func(test_point) < 0.0 {
                println!("ordem inserida erradaaaa");
                // tem que ver se isso n vai entrar um looping infinito
                return get_plane_eq(C, B, A, test_point);
            } 

            println!("ordem inserida certaaaa");
            func
        }

        let mut func_n  = get_plane_eq(A, B, C, test_point);
        let mut func_f  = get_plane_eq(E, F, G, test_point);

        let mut func_r  = get_plane_eq(E, F, A, test_point);
        let mut func_l  = get_plane_eq(G, H, D, test_point);

        let mut func_t  = get_plane_eq(E, D, A, test_point);
        let mut func_b  = get_plane_eq(B, C, F, test_point);

        //let M_cam_base = self.camera.get_matrix_base();
        let mut clipping = |tri: &Triangle| -> bool {
            let a = tri.points[0];
            let b = tri.points[1];
            let c = tri.points[2];

            func_n(a) <= 0.0 || func_n(b) <= 0.0 || func_n(c) <= 0.0 ||
            func_f(a) <= 0.0 || func_f(b) <= 0.0 || func_f(c) <= 0.0 ||

            func_r(a) <= 0.0 || func_r(b) <= 0.0 || func_r(c) <= 0.0 ||
            func_l(a) <= 0.0 || func_l(b) <= 0.0 || func_l(c) <= 0.0 ||

            func_t(a) <= 0.0 || func_t(b) <= 0.0 || func_t(c) <= 0.0 ||
            func_b(a) <= 0.0 || func_b(b) <= 0.0 || func_b(c) <= 0.0 
        };


        for obj in self.objects.iter() {
            for tri in obj.triangles.iter() {
                if clipping(tri) == true {
                    println!("clipping");
                    continue
                }

                let a_vec4  = M * tri.points[0].as_vec4();
                let b_vec4  = M * tri.points[1].as_vec4();
                let c_vec4  = M * tri.points[2].as_vec4();

                let a_w = a_vec4.get_w();
                let b_w = b_vec4.get_w();
                let c_w = c_vec4.get_w();

                let a  = a_vec4.as_vec2();
                let b  = b_vec4.as_vec2();
                let c  = c_vec4.as_vec2();

                let a_depth: f32 = camera_pos.dist(tri.points[0]) as _;
                let b_depth: f32 = camera_pos.dist(tri.points[1]) as _;
                let c_depth: f32 = camera_pos.dist(tri.points[2]) as _;

                self.canva.draw_triangle_with_depth(a / a_w, 
                                                    b / b_w, 
                                                    c / c_w, 
                                                    a_depth, 
                                                    b_depth, 
                                                    c_depth);

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
