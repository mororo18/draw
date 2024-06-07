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
    label: String,
}

impl Triangle {
    pub
    fn new (points: [Vec3; 3], label: & str) -> Self {
        Self {
            points: points,
            color: Color::White,
            label: String::from(label),
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
    fn get_center (&self) -> Vec3 {
        let mut sum = Vec3::zeros();

        for tri in self.triangles.iter() {
            for p in tri.points.iter() {
                sum = sum + *p;
            }
        }

        sum / (3. * self.triangles.len() as f64)
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
            ],
            "basee"
            );

        let f_a = Triangle::new([
                bot + vert_a,
                bot + vert_b,
                bot + vert_o
            ],
            "f_A"
            );

        let f_b = Triangle::new([
                bot + vert_c,
                bot + vert_b,
                bot + vert_o
            ],
            "f_B"
            );

        let f_c = Triangle::new([
                bot + vert_c,
                bot + vert_a,
                bot + vert_o
            ],
            "f_C"
            );

        Self {
            triangles: vec![f_a, f_b, f_c, base],
            //triangles: vec![base, f_c, f_b, f_a],
        }
    }

}

struct Camera {
    position: Vec3,
    direction: Vec3,

    // view volume oposite vertexes
    right_top_near:      Vec3,  // -> {r, t, n}
    left_bottom_further: Vec3,  // -> {l, b, f}

    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    pub
    fn new (pos:                 Vec3, 
            dir:                 Vec3, 
            right_top_near:      Vec3, 
            left_bottom_further: Vec3) -> Self 
    {
        Self {
            position: pos,
            direction: dir,
            right_top_near:      right_top_near,  // -> {r, t, n}
            left_bottom_further: left_bottom_further,  // -> {l, b, f}
            u: Vec3::zeros(),
            v: Vec3::zeros(),
            w: Vec3::zeros(),
        }
    }

    pub fn get_pos       (&self) -> Vec3 {self.position}
    pub fn get_direction (&self) -> Vec3 {self.direction}

    pub fn get_rightmost_visible  (&self) -> f64 {self.right_top_near.x()}
    pub fn get_topmost_visible    (&self) -> f64 {self.right_top_near.y()}
    pub fn get_nearest_visible    (&self) -> f64 {self.right_top_near.z()}

    pub fn get_leftmost_visible   (&self) -> f64 {self.left_bottom_further.x()}
    pub fn get_bottommost_visible (&self) -> f64 {self.left_bottom_further.y()}
    pub fn get_furtherest_visible (&self) -> f64 {self.left_bottom_further.z()}

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
        //let camera_pos = Vec3::new([0., 2., 4.]);
        let camera_pos = Vec3::new([0.0, 2., 4.0]);
        let camera_dir = Vec3::new([0., -0.2, -1.]);

        let n: f64 = 0.0;      // nearest
        let f: f64 = -20.0;       // furtherest

        let r: f64 = 10.0;      // right-most
        let l: f64 = -10.0;     // left-most

        let t: f64 = 10.0;      // top-most
        let b: f64 = -10.0;     // bottom-most

        assert!(n > f);
        assert!(r > l);
        assert!(t > b);


        // nearest face of the view volume
        let right_top_near      = Vec3::new([ r, t, n]);
        let left_bottom_further = Vec3::new([ l, b, f]); 
        let camera = Camera::new(camera_pos, camera_dir, right_top_near, left_bottom_further);


        let mut canva = Canva::new(width, height);
        canva.enable_depth(40.0);

        Self {
            canva:   canva,
            width:   width,
            height:  height,
            camera:  camera,
            objects: vec![Object::inv_piramid(Vec3::new([0., 0., 4.5]))],
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
    fn camera_left(&mut self) {
        self.camera.rotate_origin(-1.);
    }

    pub
    fn camera_right(&mut self) {
        self.camera.rotate_origin(1.);
    }

    fn gen_transform_matrix(&mut self) -> Matrix4 {
        let n_x: f64 = self.width as _;
        let n_y: f64 = self.height as _;

        let n = self.camera.get_nearest_visible();
        let f = self.camera.get_furtherest_visible();

        let r = self.camera.get_rightmost_visible();
        let l = self.camera.get_leftmost_visible();

        let t = self.camera.get_topmost_visible();
        let b = self.camera.get_bottommost_visible();

        let M_viewport = Matrix4::new([
            [n_x / 2.0,        0.0,  0.0,  (n_x-1.0) / 2.0],
            [      0.0,  n_y / 2.0,  0.0,  (n_y-1.0) / 2.0],
            [      0.0,        0.0,  1.0,              0.0],
            [      0.0,        0.0,  0.0,              1.0]
        ]);

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
        let camera_pos = self.camera.get_pos();
        println!("camera position {camera_pos:?}");

        let M = self.gen_transform_matrix();
        let M_cam = self.camera.get_matrix_base().transposed();


        let n = self.camera.get_nearest_visible();
        let f = self.camera.get_furtherest_visible();

        let r = self.camera.get_rightmost_visible();
        let l = self.camera.get_leftmost_visible();

        let t = self.camera.get_topmost_visible();
        let b = self.camera.get_bottommost_visible();

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

        //let test_point = (A + B + C + D + E + F + G + H) / 8.0;;
        let test_point = (M_cam * Vec3::new([(r+l)/2., (t+b)/2., (n+f)/2.]).as_vec4()).as_vec3() + camera_pos;

        println!("test_point {:?} ", test_point);


        fn get_plane_eq (A: Vec3, B: Vec3, C: Vec3, test_point: Vec3) -> impl FnMut(Vec3) -> f64 {
            let p_vec = B - A;
            let q_vec = C - B;

            let mut normal = p_vec.cross(q_vec);
            let mut k = - normal.dot(A);

            let mut test_value = normal.dot(test_point) + k;

            // a condicao de validez eh que a origem gere um valor positivo,
            // ou seja, ela esta dentro do volume de visao
            
            if test_value < 0.0 {
                //println!("ordem inserida erradaaaa");
                // tem que ver se isso n vai entrar um looping infinito
                normal = q_vec.cross(p_vec);
                k = - normal.dot(A);

                test_value = normal.dot(test_point) + k;

                if test_value < 0.0 {
                    println!("ta erradooouuu");
                    loop {}
                }

            }

            let func = move |point: Vec3| -> f64 {normal.dot(point) + k};

            return func;
        }

        let mut func_n  = get_plane_eq(A, B, C, test_point);
        let mut func_f  = get_plane_eq(E, F, G, test_point);

        let mut func_r  = get_plane_eq(E, F, A, test_point);
        let mut func_l  = get_plane_eq(G, H, D, test_point);

        let mut func_t  = get_plane_eq(E, D, A, test_point);
        let mut func_b  = get_plane_eq(B, C, F, test_point);

        //let M_cam_base = self.camera.get_matrix_base();
        let mut clipping = |tri: &Triangle| -> bool { //-> Option<(Vec3, Vec3, Vec3)> {
            let mut ret = false;

            for point in tri.points.iter() {
                ret |=
                func_n(*point) <= 0.0 ||
                func_f(*point) <= 0.0 ||

                func_r(*point) <= 0.0 ||
                func_l(*point) <= 0.0 ||

                func_t(*point) <= 0.0 ||
                func_b(*point) <= 0.0;

            }

            ret
        };


        for obj in self.objects.iter() {
            println!("pirmide pos {:?}", obj.get_center());
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

                // vis'ao ortogonal
                let camera_dir = self.camera.get_direction().normalized();
                let a_depth: f32 = camera_dir.dot(camera_pos - tri.points[0]).abs() as _;
                let b_depth: f32 = camera_dir.dot(camera_pos - tri.points[1]).abs() as _;
                let c_depth: f32 = camera_dir.dot(camera_pos - tri.points[2]).abs() as _;

                /*
                let a_depth: f32 = camera_pos.dist(tri.points[0]) as _;
                let b_depth: f32 = camera_pos.dist(tri.points[1]) as _;
                let c_depth: f32 = camera_pos.dist(tri.points[2]) as _;
                */

                println!("========= {} ========", tri.label);
                println!("a {a:?}"); 
                println!("a_depth {a_depth}");
                println!("b {b:?}"); 
                println!("b_depth {b_depth}");
                println!("c {c:?}"); 
                println!("c_depth {c_depth}");

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
