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


#[derive(Clone)]
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

    // TODO: mudar esse conceito aq para janela 
    // (window view - right, let, top, bottom) ja que agora a visao
    // em perpectiva parece funcionar
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
    fn get_matrix_basis(&self) -> Matrix4 {
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

        let M_cam_basis = self.get_matrix_basis();

        let M_cam_pos = Matrix4::new([
            [1.0,   0.0,    0.0,   -pos.x()],
            [0.0,   1.0,    0.0,   -pos.y()],
            [0.0,   0.0,    1.0,   -pos.z()],
            [0.0,   0.0,    0.0,        1.0]
        ]);

        let M_cam = M_cam_basis * M_cam_pos;

        M_cam
    }
}

struct ViewPlane {
    points: [Vec3; 3],
    normal: Vec3,
    k: f64,
}

impl ViewPlane {
    pub
    fn new (points: [Vec3; 3], positive_point_dir: Vec3) -> Self {
        let A = points[0];
        let B = points[1];
        let C = points[2];

        let p_vec = B - A;
        let q_vec = C - B;

        let mut normal = p_vec.cross(q_vec);
        let mut k = - normal.dot(A);

        let mut test_value = normal.dot(positive_point_dir) + k;

        // a condicao de validez eh que a origem gere um valor positivo,
        // ou seja, ela esta dentro do volume de visao

        if test_value < 0.0 {
            //println!("ordem inserida erradaaaa");
            // tem que ver se isso n vai entrar um looping infinito
            normal = q_vec.cross(p_vec);
            k = - normal.dot(A);

            test_value = normal.dot(positive_point_dir) + k;

            if test_value < 0.0 {
                panic!("ta erradooouuu");
            }

        }

        Self {
            points: points,
            normal: normal,
            k: k,
        }
    }

    pub
    fn func(&self, point: Vec3) -> f64 {
        self.normal.dot(point) + self.k
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
        let camera_pos = Vec3::new([15.0, 0., 5.0]);
        let camera_dir = Vec3::new([-1., 0.0, 0.0]);

        let n: f64 = -10.0;      // nearest
        let f: f64 = n - 100.0;       // furtherest

        let r: f64 = 10.0;      // right-most
        let l: f64 = -10.0;     // left-most

        let t: f64 = 10.0;      // top-most
        let b: f64 = -10.0;     // bottom-most

        assert!(n < 0.0);
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
            objects: vec![Object::inv_piramid(Vec3::new([0., 0., 4.0]))],
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
            [   n,  0.0,       0.0,      0.0],
            [  0.0,   n,       0.0,      0.0],
            [  0.0,  0.0,  (n + f),  -(n * f)],
            [  0.0,  0.0,      1.0,      0.0]
        ]);

        //let M = M_viewport * M_orth * M_cam;
        let M = M_viewport * (M_orth * P) * M_cam;

        M
    }

    pub
    fn render (&mut self) {

        self.canva.clear();

        // TODO: atualmente essa funcao  gen_transform_matrix()
        // precisa ser chamada antes de Camera::get_matrix_basis()
        // tem que resolver isso ai patrao
        let M = self.gen_transform_matrix();

        let cam_basis_matrix =  self.camera.get_matrix_basis().transposed();
        let camera_pos = self.camera.get_pos();

        let n = self.camera.get_nearest_visible();
        let f = self.camera.get_furtherest_visible();

        let r = self.camera.get_rightmost_visible();
        let l = self.camera.get_leftmost_visible();

        let t = self.camera.get_topmost_visible();
        let b = self.camera.get_bottommost_visible();

        let P = Matrix4::new([
            [   n,  0.0,       0.0,      0.0],
            [  0.0,   n,       0.0,      0.0],
            [  0.0,  0.0,  (n + f),  -(n * f)],
            [  0.0,  0.0,      1.0,      0.0]
        ]);

        let A_cam = Vec3::new([ r, t, n]).as_vec4();
        let B_cam = Vec3::new([ r, b, n]).as_vec4();      
        let C_cam = Vec3::new([ l, b, n]).as_vec4();     
        let D_cam = Vec3::new([ l, t, n]).as_vec4();     

        // window view
        let A_vec4 = cam_basis_matrix * A_cam;      // direita superior frente
        let B_vec4 = cam_basis_matrix * B_cam;
        let C_vec4 = cam_basis_matrix * C_cam;
        let D_vec4 = cam_basis_matrix * D_cam;

        // z = f
        // x = (l + r) / 2.
        // y = (f * t) / n
        let tfp_cam = Vec3::new([
            (l + r) / 2., 
            (f * t) / n, 
            f
        ]);
        let tfp_vec4: Vec4 = cam_basis_matrix * tfp_cam.as_vec4();
        let top_further_point = tfp_vec4.vec3_over_w() + camera_pos;


        // z = f
        // x = (l + r) / 2.
        // y = (f * b) / n
        let bfp_cam = Vec3::new([
            (l + r) / 2., 
            (f * b) / n, 
            f
        ]);
        let bfp_vec4: Vec4 = cam_basis_matrix * bfp_cam.as_vec4();
        let bottom_further_point = bfp_vec4.vec3_over_w() + camera_pos;

        // z = f
        // x = (f * r) / n
        // y = (b + t) / 2.
        let rfp_cam = Vec3::new([
            (f * r) / n, 
            (b + t) / 2., 
            f
        ]);
        let rfp_vec4: Vec4 = cam_basis_matrix * rfp_cam.as_vec4();
        let right_further_point = rfp_vec4.vec3_over_w() + camera_pos;

        // z = f
        // x = (f * l) / n
        // y = (b + t) / 2.
        let lfp_cam = Vec3::new([
            (f * l) / n, 
            (b + t) / 2., 
            f
        ]);
        let lfp_vec4: Vec4 = cam_basis_matrix * lfp_cam.as_vec4();
        let left_further_point = lfp_vec4.vec3_over_w() + camera_pos;


        // nearest face of the transformed view volume
        let A = A_vec4.as_vec3() / A_vec4.get_w() + camera_pos;      // direita superior frente
        let B = B_vec4.as_vec3() / B_vec4.get_w() + camera_pos;      
        let C = C_vec4.as_vec3() / C_vec4.get_w() + camera_pos;     
        let D = D_vec4.as_vec3() / D_vec4.get_w() + camera_pos;     

        let test_point = (A + bottom_further_point) / 2.0;;
        //let test_point_vec4 = cam_basis_matrix * Vec3::new([(r+l)/2., (t+b)/2., (n+f)/2.]).as_vec4();
        //let test_point = (test_point_vec4).as_vec3() / test_point_vec4.get_w() + camera_pos;


        fn get_plane_eq (A: Vec3, B: Vec3, C: Vec3, test_point: Vec3) -> (Box< dyn FnMut(Vec3) -> f64 >, Vec3)  {
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

            let normal_cpy = normal.clone();

            let func = Box::new( move |point: Vec3| -> f64 {normal.dot(point) + k} );

            return (func, normal_cpy);
        }

        /*
        let mut func_n  = get_plane_eq(A, B, C, test_point);
        let mut func_f  = get_plane_eq(left_further_point, 
                                       right_further_point, 
                                       top_further_point, 
                                       test_point);

        let mut func_r  = get_plane_eq(right_further_point, 
                                        A, B, test_point);
        let mut func_l  = get_plane_eq(left_further_point, 
                                        C, D, test_point);

        let mut func_t  = get_plane_eq(top_further_point, 
                                        D, A, test_point);
        let mut func_b  = get_plane_eq(bottom_further_point, 
                                        C, B, test_point);
        */

        let mut func_planes = [
            ViewPlane::new([A, B, C], test_point),
            ViewPlane::new([left_further_point, 
                            right_further_point, 
                            top_further_point],
                            test_point),

            ViewPlane::new([right_further_point, A, B],
                            test_point),
            ViewPlane::new([left_further_point, C, D],
                            test_point),

            ViewPlane::new([top_further_point, D, A],
                            test_point),
            ViewPlane::new([bottom_further_point, C, B],
                            test_point)
        ];
        //let M_cam_basis = self.camera.get_matrix_basis();
                                                        // array de tuplas (eq do plano e o vetor
                                                        // normal)
        fn clipping(tri: &Triangle, view_planes: &[ViewPlane]) -> Vec<Triangle> 
        {
            let mut vertex_out = false;
            let mut plane_out: usize = 0;

            let mut inside:  Vec<Vec3> = vec![];
            let mut outside: Vec<(usize, Vec3)> = vec![];

            for point in tri.points.iter(){

                for (idx, plane) in view_planes.iter().enumerate() {
                    //ret |= (*func)(*point) <= 0.;

                    if plane.func(*point) <= 0.0 {
                        vertex_out = true;
                        plane_out = idx;
                    }
                }


                if vertex_out == true {
                    outside.push((plane_out, *point));
                } else {
                    inside.push(*point);
                }

                vertex_out = false;

            }

            if outside.len() == 0 {
                return Vec::from([tri.clone()]);
            } else if outside.len() == 3 {
                return Vec::new();
            }

            if outside.len() == 1 {
                return Vec::new();
            } else if outside.len() == 2 {

                let (p_idx, _) = outside[0];
                let plane = &view_planes[p_idx];

                let t_a = plane;

                return Vec::new();
            }

            panic!("quandt de triangulos incorreta");
            
        };

        {

            let w_ = self.width as f64 - 1.0;
            let h_ = self.height as f64 - 1.0;

            let a = Vec2::new(w_, h_);
            let b = Vec2::new(w_, 0.0);
            let c = Vec2::new(0.0, 0.0);
            let d = Vec2::new(0.0, h_);

            self.canva.draw_line(b, c);
            self.canva.draw_line(d, c);
            self.canva.draw_line(b, a);
            self.canva.draw_line(d, a);

            let center = Vec2::new(w_ / 2., h_ / 2.) ;

            let tfp_vec4 = M * top_further_point.as_vec4();
            let tfp = tfp_vec4.vec3_over_w().as_vec2() + Vec2::new(0.0, -3.);

            let bfp_vec4 = M * bottom_further_point.as_vec4();
            let bfp = bfp_vec4.vec3_over_w().as_vec2() + Vec2::new(0.0, 3.);

            let rfp_vec4 = M * right_further_point.as_vec4();
            let rfp = rfp_vec4.vec3_over_w().as_vec2() + Vec2::new(-30.0, 0.);

            let lfp_vec4 = M * left_further_point.as_vec4();
            let lfp = lfp_vec4.vec3_over_w().as_vec2() + Vec2::new(30.0, 0.);

            self.canva.draw_line(tfp, center);
            self.canva.draw_line(bfp, center);
            self.canva.draw_line(rfp, center);
            self.canva.draw_line(lfp, center);

        }

        for obj in self.objects.iter() {
            for tri in obj.triangles.iter() {
                for cliped_tri in clipping(tri, &mut func_planes).iter() {

                    let a_vec4  = M * cliped_tri.points[0].as_vec4();
                    let b_vec4  = M * cliped_tri.points[1].as_vec4();
                    let c_vec4  = M * cliped_tri.points[2].as_vec4();

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

                    self.canva.draw_triangle_with_depth(a / a_w, 
                        b / b_w, 
                        c / c_w, 
                        a_depth, 
                        b_depth, 
                        c_depth);

                }
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
