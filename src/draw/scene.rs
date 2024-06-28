use crate::draw::canva::{
    Canva,
    Color,
    VertexAttributes,
};

use crate::draw::linalg::{
    Vec2,
    Vec3,
    Vec4,
    Matrix4,
    EPS,
};


#[derive(Clone, Debug)]
struct Triangle {
    vertices:      [Vec3; 3],
    vertices_attr: [VertexAttributes; 3],
    normal: Vec3,
    color: Color,
}

impl Triangle {
    pub
    fn new (vertices:      [Vec3; 3],
            vertices_attr: [VertexAttributes; 3], 
            color: Color) -> Self
    {
        Self {
            vertices: vertices,
            vertices_attr: vertices_attr,
            color: color,
            normal: Vec3::zeros(),
        }
    }
    
    pub
    fn calc_normal(tri: Self) -> Vec3 {
        let a = tri.vertices[0];
        let b = tri.vertices[1];
        let c = tri.vertices[2];

        let p = b - a;
        let q = c - b;

        let normal = p.cross(q);

        normal
    }

    pub
    fn clip_against_planes(&self, view_planes: &[ViewPlane]) -> Vec<Self> 
    {
        let mut tri_pool:  Vec<Self> = Vec::from([self.clone()]);

        for plane in view_planes.iter() {
            let mut new_tri_pool:  Vec<Self> = Vec::with_capacity(tri_pool.len() * 2);

            for tri in tri_pool.iter() {
                let mut clipped_triangles = plane.clip(tri.clone());
                new_tri_pool.append(&mut clipped_triangles);
            }

            tri_pool = new_tri_pool;
        }

        return tri_pool;

    }

    pub
    fn get_center(&self) -> Vec3 {
        let mut sum = Vec3::zeros();
        for vertex in self.vertices.iter() {
            sum = sum + *vertex;
        }

        sum / 3.0
    }


}

type IndexedTriangle = [usize; 3];

pub
struct IndexedMesh {
    triangles:          Vec<IndexedTriangle>,
    vertices:           Vec<Vec3>,

    normals_triangles:  Vec<IndexedTriangle>,
    normals_vertices:   Vec<Vec3>,

    texture_triangles:  Vec<IndexedTriangle>,
    texture_vertices:   Vec<Vec3>,
}

impl IndexedMesh {
    /*
    pub
    fn new (tri_vec: Vec<IndexedTriangle>, vert_vec: Vec<Vec3>) -> Self {
        todo!();
        let normals = vec![Vec3::zeros(); vert_vec.len()];

        let mut ret = Self {
            triangles: tri_vec.clone(),
            vertices: vert_vec,


            normals_triangles: vec![],
            normals_vertices: normals,

            texture_triangles: vec![],
            texture_vertices: vec![],
        };

        for (tri_idx, indexed_tri) in tri_vec.iter().enumerate() {
            let tri_vertices = ret.vertices_from_indexed(tri_idx);

            let tri = Triangle::new(
                [
                    tri_vertices[0],
                    tri_vertices[1],
                    tri_vertices[2],
                ],
                Color::Green,
                "",
            );

            let normal = Triangle::calc_normal(tri);

            for vert_idx in indexed_tri {
                //let v_norm = vert_normals[vert_idx];
                ret.normals_vertices[*vert_idx] = ret.normals_vertices[*vert_idx] + normal;
            }
        }

        for normal in ret.normals_vertices.iter_mut() {
            *normal = normal.normalized();
        }

        return ret;
    }
    */

    //TODO: enxugar essas 3 funcs ".._from_index" aq
    fn vec3_list_from_indexed(indexed_tri: IndexedTriangle, vert_list: &Vec<Vec3>) -> [Vec3; 3] {
        let a_idx = indexed_tri[0];
        let b_idx = indexed_tri[1];
        let c_idx = indexed_tri[2];

        unsafe {
            let a_vert = *vert_list.get_unchecked(a_idx);
            let b_vert = *vert_list.get_unchecked(b_idx);
            let c_vert = *vert_list.get_unchecked(c_idx);

            [a_vert, 
            b_vert, 
            c_vert]
        }
    }

    pub
    fn vertices_from_index (&self, tri_idx: usize) -> [Vec3; 3] {
        debug_assert!(tri_idx < self.triangles.len());
        unsafe {
            Self::vec3_list_from_indexed(
                *self.triangles.get_unchecked(tri_idx), 
                &self.vertices
            )
        }
    }

    pub
    fn normals_from_index (&self, tri_idx: usize) -> [Vec3; 3] {
        debug_assert!(tri_idx < self.normals_triangles.len());
        unsafe {
            Self::vec3_list_from_indexed(
                *self.normals_triangles.get_unchecked(tri_idx), 
                &self.normals_vertices
            )
        }
    }

    pub
    fn textures_from_index (&self, tri_idx: usize) -> [Vec3; 3] {
        debug_assert!(tri_idx < self.texture_triangles.len());
        unsafe {
            Self::vec3_list_from_indexed(
                *self.texture_triangles.get_unchecked(tri_idx), 
                &self.texture_vertices
            )
        }
    }

}

pub
struct Texture {
    img: Vec<u8>,
    width: usize,
    height: usize,
    components: usize,

}

impl Texture {
    pub
    fn new (img: Vec<u8>, width: usize, height: usize, components: usize) -> Self {
        Self {
            img: img,
            width: width,
            height: height,
            components: components,
        }
    }

    pub
    fn get_rgb_slice(&self, u: f32, v: f32) -> [u8; 3] {
        debug_assert!(
            0.0 <= u && u < 1.0 &&
            0.0 <= v && v < 1.0
        );

        let u_idx = (u * (self.width)  as f32).floor() as usize;
        let v_idx = self.height -1 - (v * (self.height) as f32).floor() as usize;
        
        let offset = (v_idx * self.width + u_idx) * self.components;
        self.img[
            (offset) ..
            (offset) + 3
        ].try_into().unwrap()
    }


    pub 
    fn new_empty() -> Self {
        Self::new( Vec::new(), 0, 0, 0) 
    }

    pub
    fn load_from_file (filename: &str) -> Self {
        use std::fs::File;
        use std::path::Path;
        use stb::image::stbi_load_from_reader;
        use stb::image::Channels;

        let path = Path::new(filename);

        let mut file = File::open(&path).expect("Unable to open file");
        let (info, img) = stbi_load_from_reader(&mut file, Channels::Rgb)
                            .expect("Deu errado ler a textura");

        Self::new(
            Vec::from(img.as_slice()),
            info.width      as usize,
            info.height     as usize,
            info.components as usize
        )
    }

}

pub
struct Object {
    mesh: IndexedMesh,
    texture: Texture,
    //triangles: Vec<Triangle>,
}

impl Object {
    pub
    fn new (mesh: IndexedMesh, texture: Texture) -> Self {
        Self {
            mesh: mesh,
            texture: texture,
        }
    }

    /*
    pub
    fn get_center (&self) -> Vec3 {
        let mut sum = Vec3::zeros();

        for tri in self.triangles.iter() {
            for p in tri.points.iter() {
                sum = sum + *p;
            }
        }

        sum / (3. * self.triangles.len() as f32)
    }
    */

    pub
    fn load_from_file(filename: &str) -> Self {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::Path;

        println!("Reading Wavefront .obj file: {}", filename);
        let path = Path::new(filename);

        let file = File::open(&path).expect("Unable to open file");
        let reader = BufReader::new(file);

        let mut vertices: Vec<Vec3> = Vec::new();
        let mut normals:  Vec<Vec3> = Vec::new();
        let mut texture:  Vec<Vec3> = Vec::new();

        let mut faces:         Vec<IndexedTriangle> = Vec::new();
        let mut texture_faces: Vec<IndexedTriangle> = Vec::new();
        let mut normals_faces: Vec<IndexedTriangle> = Vec::new();

        for line in reader.lines() {
            //println!("{}", line.as_ref().unwrap().clone());
            let line = line.expect("Unable to read line");
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {

                "v" => {
                    //println!("{:?}", parts);
                    // Vertex
                    let x: f32 = parts[1].parse().expect("Invalid vertex x coordinate");
                    let y: f32 = parts[3].parse().expect("Invalid vertex y coordinate");
                    let z: f32 = parts[2].parse().expect("Invalid vertex z coordinate");
                    vertices.push(Vec3::new([x, y, z]));
                },

                "vn" => {
                    // Normal
                    let x: f32 = parts[1].parse().expect("Invalid normal x coordinate");
                    let y: f32 = parts[3].parse().expect("Invalid normal y coordinate");
                    let z: f32 = parts[2].parse().expect("Invalid normal z coordinate");
                    normals.push(Vec3::new([x, y, z]));
                },

                "vt" => {
                    //println!("{:?}", parts);
                    // Vertex
                    let u: f32 = parts[1].parse().expect("Invalid vertex u coordinate");
                    let v: f32 = parts[2].parse().expect("Invalid vertex v coordinate");
                    let w: f32 = parts[3].parse().expect("Invalid vertex w coordinate");
                    texture.push(Vec3::new([u, v, w]));
                },

                "f" => {
                    let mut vertex_idx_list:  Vec<usize> = vec![];
                    let mut texture_idx_list: Vec<usize> = vec![];
                    let mut normals_idx_list: Vec<usize> = vec![];

                    for part in &parts[1..] {

                        let indices: Vec<&str> = part.split('/').collect();
                        let vertex_idx: usize = indices[0].parse().expect("Deu ruimm");
                        //let associeted_vertex: Vec3 = vertices[vertex_idx - 1];

                        vertex_idx_list.push(vertex_idx - 1);
                        //vertex_list.push(associeted_vertex);

                        // indice da textura
                        if indices.len() >= 2 && indices[1].is_empty() == false {
                            let texture_index: usize = indices[1].parse().expect("Invalid texture index");
                            texture_idx_list.push(texture_index - 1);
                        }

                        // indice do vetor normal
                        if indices.len() == 3 && indices[2].is_empty() == false {
                            let normal_index: usize = indices[2].parse().expect("Invalid normal index");
                            normals_idx_list.push(normal_index - 1);
                        }


                    }


                    // TODO: adaptar isso aq para faces com um numero variado de vertices 
                    let vertex_idx_a = vertex_idx_list[0];
                    let vertex_idx_b = vertex_idx_list[1];
                    let vertex_idx_c = vertex_idx_list[2];
                    let vertex_idx_d = vertex_idx_list[3];

                    faces.push([
                        vertex_idx_a,
                        vertex_idx_b,
                        vertex_idx_c
                    ]);
                    faces.push([
                        vertex_idx_c,
                        vertex_idx_d,
                        vertex_idx_a
                    ]);

                    let texture_idx_a = texture_idx_list[0];
                    let texture_idx_b = texture_idx_list[1];
                    let texture_idx_c = texture_idx_list[2];
                    let texture_idx_d = texture_idx_list[3];

                    texture_faces.push([
                        texture_idx_a,
                        texture_idx_b,
                        texture_idx_c
                    ]);
                    texture_faces.push([
                        texture_idx_c,
                        texture_idx_d,
                        texture_idx_a
                    ]);

                    let normals_idx_a = normals_idx_list[0];
                    let normals_idx_b = normals_idx_list[1];
                    let normals_idx_c = normals_idx_list[2];
                    let normals_idx_d = normals_idx_list[3];

                    normals_faces.push([
                        normals_idx_a,
                        normals_idx_b,
                        normals_idx_c
                    ]);
                    normals_faces.push([
                        normals_idx_c,
                        normals_idx_d,
                        normals_idx_a
                    ]);

                },

                _ => {println!("Can't interpret this line: {}", line);},
            }
        }

        println!("Triangles count: {}",  faces.len());

        //let mesh = IndexedMesh::new(faces, vertices);
        let mesh = IndexedMesh {
            triangles: faces,
            vertices: vertices,

            normals_triangles: normals_faces,
            normals_vertices: normals,


            texture_triangles: texture_faces,
            texture_vertices: texture,
        };

        Self::new(
            mesh,
            Texture::load_from_file("airplane.jpg")
        )
    }

        /*
    pub
    fn inv_piramid (bot: Vec3) -> Self {
        let height: f32 = 3.0;
        let side = 3.0_f32;
        let l = (side / 2.0) * (2.0 / f32::sqrt(3.0));

        let c_x = 0.0_f32;
        let c_z = l;

        let a_x = side / 2.0;
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
            Color::White,
            "white basee"
            );

        let f_a = Triangle::new([
                bot + vert_a,
                bot + vert_b,
                bot + vert_o
            ],
            Color::Blue,
            "Blue f_A"
            );

        let f_b = Triangle::new([
                bot + vert_c,
                bot + vert_b,
                bot + vert_o
            ],
            Color::Red,
            "Red f_B"
            );

        let f_c = Triangle::new([
                bot + vert_c,
                bot + vert_a,
                bot + vert_o
            ],
            Color::Green,
            "Green f_C"
            );

        Self {
            triangles: vec![f_a, f_b, f_c, base],
            //triangles: vec![base, f_c, f_b, f_a],
        }
    }
        */

}

#[derive(Clone, Debug)]
struct CameraWindow {
    top:    f32,
    bottom: f32,
    right:  f32,
    left:   f32,
}

struct Camera {
    position:  Vec3,
    direction: Vec3,

    window_view: CameraWindow,
    min_view_dist: f32,
    max_view_dist: f32,

    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    pub
    fn new (pos:      Vec3, 
            dir:      Vec3,
            ratio:    f32) -> Self 
    {

        let n: f32 = -10.0;      // nearest
        /*
        let f: f32 = n - 100.0;       // furtherest

        let r: f32 = 10.0;      // right-most
        let l: f32 = -10.0;     // left-most

        let t: f32 = 10.0;      // top-most
        let b: f32 = -10.0;     // bottom-most
        */

        let horizontal_view_angle: f32 = 50.0;  // degrees
        let h_angle_rad = horizontal_view_angle.to_radians();

        let right = n.abs() * h_angle_rad.tan();
        let left = - right;

        let top = ratio.recip() * right;
        let bottom = - top;
        let further = n - 5000.;

        assert!(n < 0.0);
        assert!(n > further);
        assert!(right > left);
        assert!(top > bottom);

        // nearest face of the view volume

        Self {
            position:    pos,
            direction:   dir,
            window_view: CameraWindow {
                top:    top,
                bottom: bottom,
                right:  right,
                left:   left,
            },

            min_view_dist: n,
            max_view_dist: further,

            u: Vec3::zeros(),
            v: Vec3::zeros(),
            w: Vec3::zeros(),
        }
    }

    pub fn get_pos       (&self) -> Vec3 {self.position}
    pub fn get_direction (&self) -> Vec3 {self.direction}

    pub fn get_window (&self) -> CameraWindow {self.window_view.clone()}

    pub fn get_min_view_dist (&self) -> f32 {self.min_view_dist}
    pub fn get_max_view_dist (&self) -> f32 {self.max_view_dist}

    pub
    fn set_pos(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub
    fn rotate_origin(&mut self, theta: f32) {

        let new_pos = Matrix4::rotate_y(theta.to_radians()) * 
                        self.position.as_vec4();
        let new_dir = Matrix4::rotate_y(theta.to_radians()) * 
                        self.direction.as_vec4();

        self.position = new_pos.as_vec3();
        self.direction = new_dir.as_vec3();

    }

    pub
    fn get_basis_matrix(&self) -> Matrix4 {
        let u = self.u;
        let v = self.v;
        let w = self.w;

        assert!(
            u.norm() > 0. &&
            v.norm() > 0. &&
            w.norm() > 0.,
            "Base da camera ainda n'ao foi calculada."
        );

        Matrix4::new([
            [u.x(), v.x(),  w.x(), 0.0],
            [u.y(), v.y(),  w.y(), 0.0],
            [u.z(), v.z(),  w.z(), 0.0],
            [0.0,     0.0,    0.0, 1.0]
        ])
    }

    pub
    fn update_basis(&mut self) {
        let pos = self.position;
        let g   = self.direction;
        let top_dir = Vec3::new([0.,  1.,  0.]);

        let w = (g / g.norm()) * (-1.0);

        let t_x_w = top_dir.cross(w);
        let u = t_x_w / t_x_w.norm();

        let v = w.cross(u);

        self.u = u;
        self.v = v;
        self.w = w;
    }

    pub
    fn gen_matrix(&mut self) -> Matrix4 {
        self.update_basis();

        let matrix_basis_transp = self.get_basis_matrix().transposed();

        let pos = self.position;
        let matrix_pos = Matrix4::new([
            [1.0,   0.0,    0.0,   -pos.x()],
            [0.0,   1.0,    0.0,   -pos.y()],
            [0.0,   0.0,    1.0,   -pos.z()],
            [0.0,   0.0,    0.0,        1.0]
        ]);

        let matrix_cam = matrix_basis_transp * matrix_pos;

        matrix_cam
    }

    fn gen_view_planes(&mut self) -> [ViewPlane; 6] {
        self.update_basis();
        let matrix_basis =  self.get_basis_matrix();
        let camera_pos = self.get_pos();

        let n = self.get_min_view_dist();
        let f = self.get_max_view_dist();

        let camera_window = self.get_window();
        let r = camera_window.right;
        let l = camera_window.left;

        let t = camera_window.top;
        let b = camera_window.bottom;

        let a_cam = Vec3::new([ r, t, n]).as_vec4();
        let b_cam = Vec3::new([ r, b, n]).as_vec4();      
        let c_cam = Vec3::new([ l, b, n]).as_vec4();     
        let d_cam = Vec3::new([ l, t, n]).as_vec4();     

        // window view
        let a_vec4 = matrix_basis * a_cam;      // direita superior frente
        let b_vec4 = matrix_basis * b_cam;
        let c_vec4 = matrix_basis * c_cam;
        let d_vec4 = matrix_basis * d_cam;

        // z = f
        // x = (l + r) / 2.
        // y = (f * t) / n
        let tfp_cam = Vec3::new([
            (l + r) / 2., 
            (f * t) / n, 
            f
        ]);
        let tfp_vec4: Vec4 = matrix_basis * tfp_cam.as_vec4();
        let top_further_point = tfp_vec4.vec3_over_w() + camera_pos;


        // TODO: aplicar essa logica aq p o restante
        let x_center = (l + r) / 2.;
        // z = f
        // x = (l + r) / 2.
        // y = (f * b) / n
        let bfp_cam = Vec3::new([
            x_center, 
            x_center + (f * (b-x_center)) / n, 
            f
        ]);
        let bfp_vec4: Vec4 = matrix_basis * bfp_cam.as_vec4();
        let bottom_further_point = bfp_vec4.vec3_over_w() + camera_pos;

        // z = f
        // x = (f * r) / n
        // y = (b + t) / 2.
        let rfp_cam = Vec3::new([
            (f * r) / n, 
            (b + t) / 2., 
            f
        ]);
        let rfp_vec4: Vec4 = matrix_basis * rfp_cam.as_vec4();
        let right_further_point = rfp_vec4.vec3_over_w() + camera_pos;

        // z = f
        // x = (f * l) / n
        // y = (b + t) / 2.
        let lfp_cam = Vec3::new([
            (f * l) / n, 
            (b + t) / 2., 
            f
        ]);
        let lfp_vec4: Vec4 = matrix_basis * lfp_cam.as_vec4();
        let left_further_point = lfp_vec4.vec3_over_w() + camera_pos;


        let a_point = a_vec4.as_vec3() / a_vec4.get_w() + camera_pos;
        let b_point = b_vec4.as_vec3() / b_vec4.get_w() + camera_pos;
        let c_point = c_vec4.as_vec3() / c_vec4.get_w() + camera_pos;
        let d_point = d_vec4.as_vec3() / d_vec4.get_w() + camera_pos;

        let visible_point = (a_point + bottom_further_point) / 2.0;

        let mut func_planes = [
            ViewPlane::new(
                [
                    a_point,
                    b_point,
                    c_point
                ], 
                visible_point, 
                "perto"
            ),
            ViewPlane::new(
                [
                    left_further_point, 
                    right_further_point, 
                    top_further_point
                ],
                visible_point, 
                "longe"
            ),

            ViewPlane::new(
                [
                    right_further_point,
                    a_point,
                    b_point
                ],
                visible_point, 
                "direita"
            ),
            ViewPlane::new(
                [
                    left_further_point,
                    c_point,
                    d_point
                ],
                visible_point,
                "esquerda"
            ),

            ViewPlane::new(
                [
                    top_further_point,
                    d_point,
                    a_point
                ],
                visible_point,
                "topo"
            ),
            ViewPlane::new(
                [
                    bottom_further_point,
                    c_point,
                    b_point
                ],
                visible_point, 
                "piso"
            )
        ];

            func_planes
    }
}

struct ViewPlane {
    //points: [Vec3; 3],
    normal: Vec3,
    k: f32,
    //label: String,
}

impl ViewPlane {
    pub
    fn new (points: [Vec3; 3], visible_point: Vec3, _label: &str) -> Self {
        let a_point = points[0];
        let b_point = points[1];
        let c_point = points[2];

        let p_vec = b_point - a_point;
        let q_vec = c_point - b_point;

        let mut normal = p_vec.cross(q_vec);
        let mut k = - normal.dot(a_point);

        let mut test_value = normal.dot(visible_point) + k;

        // a condicao de validez eh que a origem gere um valor positivo,
        // ou seja, ela esta dentro do volume de visao

        if test_value < 0.0 {
            normal =   q_vec.cross(p_vec);
            k      = - normal.dot(a_point);

            test_value = normal.dot(visible_point) + k;
            assert!(test_value > 0.0, "N'ao deu pra criar o plano");
        }

        Self {
            //points: points,
            normal: normal,
            k: k,
            //label: String::from(label),
        }
    }

    pub
    fn func(&self, point: Vec3) -> f32 {
        // essa constante subtraindo serve para que os triangulos sejam
        // clipados um pouquinho antes do plano real, de modo que
        // n'ao exista chance de calcular coordenadas invalidas
        // apos as transformacoes devido 'a imprecisao do float
        
        // TODO: encontrar relação de proporcionalidade entre essa
        // constante e as dimensões do cenário renderizado.

        let delta = 500.0;

        self.normal.dot(point) + self.k - delta
    }

    pub
    fn normal (&self) -> Vec3 {
        self.normal
    }

    pub
    fn clip (&self, tri: Triangle) -> Vec<Triangle> {
        use std::mem::swap;

        let mut a_vertex = tri.vertices[0];
        let mut b_vertex = tri.vertices[1];
        let mut c_vertex = tri.vertices[2];

        let mut a_attr = tri.vertices_attr[0];
        let mut b_attr = tri.vertices_attr[1];
        let mut c_attr = tri.vertices_attr[2];

        let mut f_a = self.func(a_vertex);
        let mut f_b = self.func(b_vertex);
        let mut f_c = self.func(c_vertex);

        if f_a > 0.0 && 
           f_b > 0.0 && 
           f_c > 0.0 
        {
            return Vec::from([tri]);
        } else
        if f_a <= 0.0 && 
           f_b <= 0.0 && 
           f_c <= 0.0 
        {
            return Vec::new();
        }


        if f_a * f_c >= 0.0 {
            swap(&mut f_b,      &mut f_c);
            swap(&mut b_vertex, &mut c_vertex);

            swap(&mut b_attr,   &mut c_attr);


            swap(&mut f_a,      &mut f_b);
            swap(&mut a_vertex, &mut b_vertex);

            swap(&mut a_attr,   &mut b_attr);

        } else if f_b * f_c >= 0.0 {
            swap(&mut f_a,      &mut f_c);
            swap(&mut a_vertex, &mut c_vertex);

            swap(&mut a_attr,   &mut c_attr);


            swap(&mut f_a,      &mut f_b);
            swap(&mut a_vertex, &mut b_vertex);

            swap(&mut a_attr,   &mut b_attr);
        }

        //  resolvendo para t onde p pertence ao plano
        //  p = in + t * (out - in)

        let t_a = self.func(a_vertex) /
            self.normal().dot(a_vertex - c_vertex) - EPS;
        let new_vertex_a = a_vertex + (c_vertex - a_vertex) * t_a;
        let new_a_attr   = a_attr   + (c_attr   - a_attr)   * t_a;


        let t_b = self.func(b_vertex) /
            self.normal().dot(b_vertex - c_vertex) - EPS;
        let new_vertex_b = b_vertex + (c_vertex - b_vertex) * t_b;
        let new_b_attr   = b_attr   + (c_attr   - b_attr)   * t_b;


        if f_c <= 0.0 {
            let new_triangle_a = Triangle::new(
                [
                    a_vertex, 
                    new_vertex_a,
                    new_vertex_b
                ],
                [
                    a_attr, 
                    new_a_attr,
                    new_b_attr
                ],
                tri.color,
            );

            let new_triangle_b = Triangle::new(
                [
                    a_vertex, 
                    b_vertex,
                    new_vertex_b
                ],
                [
                    a_attr, 
                    b_attr,
                    new_b_attr
                ],
                tri.color,
            );

            return Vec::from([new_triangle_a, new_triangle_b]);
        } else {
            let new_triangle_c = Triangle::new(
                [
                    c_vertex, 
                    new_vertex_a,
                    new_vertex_b
                ],
                [
                    c_attr, 
                    new_a_attr,
                    new_b_attr
                ],
                tri.color,
            );

            return Vec::from([new_triangle_c]);

        }

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

    light_source: Vec3,
}

impl Scene {

    pub
    fn new (width: usize, height: usize) -> Self {
        //let camera_pos = Vec3::new([0., 2., 4.]);
        let camera_pos = Vec3::new([0., 300., 630.0]);
        let camera_dir = Vec3::new([0., -0.3, -1.0]);

        let light_source = Vec3::new([0., 300., 5.]);

        let ratio = (width as f32) / (height as f32);
        let camera = Camera::new(camera_pos, camera_dir, ratio);


        let mut canva = Canva::new(width, height);
        canva.enable_depth(100000.0);
        //let obj = Object::inv_piramid(Vec3::zeros());
        let obj = Object::load_from_file("airplane.obj");

        Self {
            canva:   canva,
            width:   width,
            height:  height,
            camera:  camera,
            objects: vec![obj],

            light_source: light_source,
        }
    }

    pub
    fn camera_up(&mut self) {
        let cam_pos = self.camera.get_pos();
        self.camera.set_pos(cam_pos + Vec3::new([0., 0.5, 0.]));
    }

    pub
    fn camera_down(&mut self) {
        let cam_pos = self.camera.get_pos();
        self.camera.set_pos(cam_pos + Vec3::new([0., -0.5, 0.]));
    }

    pub
    fn camera_left(&mut self) {
        let theta: f32 = -4.0;
        self.camera.rotate_origin(theta);

        //let cam_pos = self.camera.get_pos();
        //self.camera.set_pos(cam_pos + Vec3::new([0.05, 0., 0.]));

        let new_light = Matrix4::rotate_y(theta.to_radians()) * 
                        self.light_source.as_vec4();

        //self.light_source = new_light.as_vec3();
    }

    pub
    fn camera_right(&mut self) {
        let theta: f32 = 4.0;
        self.camera.rotate_origin(theta);

        //let cam_pos = self.camera.get_pos();
        //self.camera.set_pos(cam_pos + Vec3::new([-0.05, 0., 0.]));

        let new_light = Matrix4::rotate_y(theta.to_radians()) * 
                        self.light_source.as_vec4();

        //self.light_source = new_light.as_vec3();
    }

    fn gen_transformation_matrix(&mut self) -> Matrix4 {
        let n_x: f32 = self.width as _;
        let n_y: f32 = self.height as _;

        let n = self.camera.get_min_view_dist();
        let f = self.camera.get_max_view_dist();

        let camera_window = self.camera.get_window();
        let r = camera_window.right;
        let l = camera_window.left;

        let t = camera_window.top;
        let b = camera_window.bottom;

        let matrix_viewport = Matrix4::new([
            [n_x / 2.0,        0.0,  0.0,  (n_x-1.0) / 2.0],
            [      0.0,  n_y / 2.0,  0.0,  (n_y-1.0) / 2.0],
            [      0.0,        0.0,  1.0,              0.0],
            [      0.0,        0.0,  0.0,              1.0]
        ]);

        let matrix_orth = Matrix4::new([
            [2.0 / (r-l),          0.0,          0.0,  -(r+l) / (r-l)],
            [        0.0,  2.0 / (t-b),          0.0,  -(t+b) / (t-b)],
            [        0.0,          0.0,  2.0 / (n-f),  -(n+f) / (n-f)],
            [        0.0,          0.0,          0.0,             1.0]
        ]);

        let matrix_cam = self.camera.gen_matrix();

        let persp = Matrix4::new([
            [   n,  0.0,       0.0,      0.0],
            [  0.0,   n,       0.0,      0.0],
            [  0.0,  0.0,  (n + f), -(n * f)],
            [  0.0,  0.0,      1.0,      0.0]
        ]);

        //let M = M_viewport * M_orth * M_cam;
        let matrix_transf = matrix_viewport * matrix_orth * persp * matrix_cam;

        matrix_transf
    }

    pub
    fn render (&mut self) {

        self.canva.clear();

        let matrix_transf = self.gen_transformation_matrix();

        let camera_pos = self.camera.get_pos();

        let func_planes = self.camera.gen_view_planes();

        for obj in self.objects.iter() {
            // TODO: ta meio feio isso aq, tem que embelezar
            for (tri_idx, _) in obj.mesh.triangles.iter().enumerate() {
                let tri_vertices = obj.mesh.vertices_from_index(tri_idx);
                let tri_normals  = obj.mesh.normals_from_index(tri_idx);
                let tri_textures = obj.mesh.textures_from_index(tri_idx);

                let a_vertex = tri_vertices[0];
                let b_vertex = tri_vertices[1];
                let c_vertex = tri_vertices[2];

                let a_normal = tri_normals[0].normalized();
                let b_normal = tri_normals[1].normalized();
                let c_normal = tri_normals[2].normalized();

                let a_texture_coord = tri_textures[0];
                let b_texture_coord = tri_textures[1];
                let c_texture_coord = tri_textures[2];

                let a_light = (a_vertex - self.light_source).normalized();
                let b_light = (b_vertex - self.light_source).normalized();
                let c_light = (c_vertex - self.light_source).normalized();

                let a_eye = (a_vertex - camera_pos).normalized();
                let b_eye = (b_vertex - camera_pos).normalized();
                let c_eye = (c_vertex - camera_pos).normalized();


                let a_depth: f32 = (camera_pos - a_vertex).norm() as _;
                let b_depth: f32 = (camera_pos - b_vertex).norm() as _;
                let c_depth: f32 = (camera_pos - c_vertex).norm() as _;

                let a_attr = VertexAttributes::new(
                    Vec2::new(0., 0.),
                    Color::Green,
                    a_depth,
                    a_normal,
                    a_light,
                    a_eye,
                    a_texture_coord,
                );

                let b_attr = VertexAttributes::new(
                    Vec2::new(0., 0.),
                    Color::Green,
                    b_depth,
                    b_normal,
                    b_light,
                    b_eye,
                    b_texture_coord,
                );


                let c_attr = VertexAttributes::new(
                    Vec2::new(0., 0.),
                    Color::Green,
                    c_depth,
                    c_normal,
                    c_light,
                    c_eye,
                    c_texture_coord,
                );

                let original_tri = Triangle::new(
                    [
                        a_vertex,
                        b_vertex,
                        c_vertex,
                    ],
                    [
                        a_attr,
                        b_attr,
                        c_attr,
                    ],
                    Color::Green,
                );

                let mut clipped_triangles = original_tri.clip_against_planes(&func_planes);
                for clipped_tri in clipped_triangles.iter_mut() {

                    let a_vec4  = matrix_transf * clipped_tri.vertices[0].as_vec4();
                    let b_vec4  = matrix_transf * clipped_tri.vertices[1].as_vec4();
                    let c_vec4  = matrix_transf * clipped_tri.vertices[2].as_vec4();

                    let a_w = a_vec4.get_w();
                    let b_w = b_vec4.get_w();
                    let c_w = c_vec4.get_w();

                    let a_coord  = a_vec4.as_vec2() / a_w;
                    let b_coord  = b_vec4.as_vec2() / b_w;
                    let c_coord  = c_vec4.as_vec2() / c_w;

                    clipped_tri.vertices_attr[0].screen_coord = a_coord;
                    clipped_tri.vertices_attr[1].screen_coord = b_coord;
                    clipped_tri.vertices_attr[2].screen_coord = c_coord;
                    // vis'ao ortogonal
                    /*
                    let camera_dir = self.camera.get_direction().normalized();
                    let a_depth: f32 = camera_dir.dot(camera_pos - tri.points[0]).abs() as _;
                    let b_depth: f32 = camera_dir.dot(camera_pos - tri.points[1]).abs() as _;
                    let c_depth: f32 = camera_dir.dot(camera_pos - tri.points[2]).abs() as _;
                    */

                    self.canva.draw_triangle_with_attributes(
                        clipped_tri.vertices_attr[0],
                        clipped_tri.vertices_attr[1],
                        clipped_tri.vertices_attr[2],

                        &obj.texture
                    );

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
