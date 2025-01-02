mod mesh;
mod object;

use mesh::IndexedMesh;
pub use object::Object;
pub use object::ObjectInfo;

use super::canvas::{Canvas, Color, VertexAttributes};

use super::linalg::{Matrix4, Vec2, Vec3, EPS};

#[derive(Clone, Copy, Debug)]
struct Triangle {
    vertices: [Vec3; 3],
    vertices_attr: [VertexAttributes; 3],
}

impl Triangle {
    pub fn new(vertices: [Vec3; 3], vertices_attr: [VertexAttributes; 3]) -> Self {
        Self {
            vertices,
            vertices_attr,
        }
    }

    fn zeroed() -> Self {
        unsafe { std::mem::MaybeUninit::<Triangle>::zeroed().assume_init() }
    }

    pub fn calc_normal(tri: &Self) -> Vec3 {
        let a = tri.vertices[0];
        let b = tri.vertices[1];
        let c = tri.vertices[2];

        let p = b - a;
        let q = c - b;

        let normal = p.cross(q);

        normal
    }

    pub fn clip_against_planes(
        &self,
        view_planes: &([ViewPlane; 2], [ViewPlane; 4]),
        tri_pool_ret: &mut [Triangle],
    ) -> usize {
        let depth_planes = &view_planes.0;
        let lateral_planes = &view_planes.1;

        let mut tri_pool_size: usize = 0;

        // Nessa etapa do pipeline, a verificação de clipagem nos
        // planos laterais (right, left, top, bottom) vai impedir
        // apenas aqueles triângulos que estiverem *completamente*
        // fora do volume de visualização. A clipagem dos triângulos
        // que estão parcialmente fora não é feita aqui pois é resolvido
        // durante a rasterização.
        if lateral_planes[0].at_least_partially_visible(&self)
            && lateral_planes[1].at_least_partially_visible(&self)
            && lateral_planes[2].at_least_partially_visible(&self)
            && lateral_planes[3].at_least_partially_visible(&self)
        {
            tri_pool_ret[0] = self.clone();
            tri_pool_size = 1;
        }

        // TODO: substituir por heapless::Vec
        let mut new_tri_pool: [Triangle; 12] =
            unsafe { std::mem::MaybeUninit::<[Triangle; 12]>::zeroed().assume_init() };

        let mut tri_pool_ref: &mut [Triangle] = tri_pool_ret;
        let mut new_pool_ref: &mut [Triangle] = new_tri_pool.as_mut();

        // the 'far' and 'near' planes will apply the complete clipping method in the triangles.
        for plane in depth_planes.iter() {
            let mut new_tri_pool_size: usize = 0;

            for tri in tri_pool_ref[0..tri_pool_size].iter() {
                let clipped_count = plane.clip(tri, new_pool_ref[new_tri_pool_size..].as_mut());

                new_tri_pool_size += clipped_count;
            }

            std::mem::swap(&mut tri_pool_ref, &mut new_pool_ref);
            std::mem::swap(&mut tri_pool_size, &mut new_tri_pool_size);
        }

        return tri_pool_size;
    }

    pub fn get_center(&self) -> Vec3 {
        let mut sum = Vec3::zeros();
        for vertex in self.vertices.iter() {
            sum = sum + *vertex;
        }

        sum / 3.0
    }
}

pub struct TextureMap {
    img: Vec<u8>,
    width: usize,
    height: usize,
    components: usize,

    f_width: f32,
    f_height: f32,
}

impl TextureMap {
    pub fn new(img: Vec<u8>, width: usize, height: usize, components: usize) -> Self {
        assert!(img.len() % components == 0);
        assert!(img.len() / components == width * height);

        Self {
            img,
            width,
            height,
            components,

            f_width: width as f32,
            f_height: height as f32,
        }
    }

    pub fn default() -> Self {
        Self::new(
            Vec::from(Color::White.as_slice()),
            1, // width
            1, // height
            3, // components
        )
    }

    pub fn get_rgba_slice(&self, u: f32, v: f32) -> [u8; 4] {
        debug_assert!((0.0..=1.0).contains(&u));
        debug_assert!((0.0..=1.0).contains(&v));
        debug_assert!(self.components == 4);

        // TODO: Verify correctness
        let u_idx = (u * self.f_width).floor() as usize;
        let v_idx = self.height - 1 - (v * self.f_height).floor() as usize;

        let offset = (v_idx * self.width + u_idx) * self.components;

        debug_assert!(offset < self.img.len());
        debug_assert!(offset + 4 <= self.img.len());

        self.img[(offset)..(offset) + 4].try_into().unwrap()
    }

    pub fn get_rgb_slice(&self, u: f32, v: f32) -> [u8; 3] {
        debug_assert!((0.0..=1.0).contains(&u));
        debug_assert!((0.0..=1.0).contains(&v));

        // TODO: Verify correctness
        let u_idx = (u * (self.f_width - 1.)).floor() as usize;
        let v_idx = self.height - 1 - (v * (self.f_height - 1.)).floor() as usize;

        let offset = (v_idx * self.width + u_idx) * self.components;

        debug_assert!(offset < self.img.len());
        debug_assert!(offset + 3 <= self.img.len());

        self.img[(offset)..(offset) + 3].try_into().unwrap()
    }

    pub fn new_empty() -> Self {
        Self::new(Vec::new(), 0, 0, 0)
    }

    pub fn load_from_file(file_path: &std::path::PathBuf) -> Self {
        use stb::image::stbi_info_from_reader;
        use stb::image::stbi_load_from_reader;
        use stb::image::Channels;
        use std::fs::File;
        use std::io::Seek;

        let mut file = File::open(file_path).expect("Unable to open file");
        let pre_info = stbi_info_from_reader(&mut file).expect("Deu errado ler a textura");

        _ = file.rewind();

        let channels = match dbg!(pre_info.components) {
            3 => Channels::Rgb,
            4 => Channels::RgbAlpha,
            _ => unreachable!(),
        };

        dbg!(channels);
        let (info, img) =
            stbi_load_from_reader(&mut file, channels).expect("Deu errado ler a textura");

        Self::new(
            Vec::from(img.as_slice()),
            info.width as usize,
            info.height as usize,
            info.components as usize,
        )
    }
}

// https://paulbourke.net/dataformats/mtl/
pub struct Texture {
    pub name: String,
    pub ka: Vec3,
    pub kd: Vec3,
    pub ks: Vec3,

    pub alpha: f32,

    pub map_ka: TextureMap,
    pub map_kd: TextureMap,
}

impl Texture {
    pub fn with_diffuse_map(diff_map: TextureMap) -> Self {
        let map_ka = TextureMap::default();
        let map_kd = diff_map;

        Self {
            name: String::from("default"),
            ka: Vec3::new([1.0, 1.0, 1.0]),
            kd: Vec3::new([1.0, 1.0, 1.0]),
            ks: Vec3::new([1.0, 1.0, 1.0]),

            alpha: 1.0,

            map_ka,
            map_kd,
        }
    }

    // TODO: impl std::default::Default trait
    pub fn default() -> Self {
        let map_ka = TextureMap::default();
        let map_kd = TextureMap::default();

        Self {
            name: String::from("default"),
            ka: Vec3::new([0.9, 0.9, 0.9]),
            kd: Vec3::new([0.4, 0.4, 0.4]),
            ks: Vec3::new([0.5, 0.5, 0.5]),

            alpha: 1.0,

            map_ka,
            map_kd,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VertexVisual {
    light: Vec3,
    eye: Vec3,
    halfway: Vec3,
    depth: f32,
}

impl VertexVisual {
    fn zeroed() -> Self {
        Self {
            light: Vec3::zeros(),
            eye: Vec3::zeros(),
            halfway: Vec3::zeros(),
            depth: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
struct CameraWindow {
    top: f32,
    bottom: f32,
    right: f32,
    left: f32,
}

pub struct Camera {
    position: Vec3,     // Lookfrom
    direction: Vec3,    // (lookat - lookfrom)
    up_direction: Vec3, // Vup

    window_view: CameraWindow,
    min_view_dist: f32, // Distância da origem até o near plane
    max_view_dist: f32, // Distância da origem até o far plane

    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, ratio: f32) -> Self {
        /*
        let f: f32 = n - 100.0;       // furtherest

        let r: f32 = 10.0;      // right-most
        let l: f32 = -10.0;     // left-most

        let t: f32 = 10.0;      // top-most
        let b: f32 = -10.0;     // bottom-most
        */

        let near: f32 = -10.0; // Distância da origem até o near plane
        let far = near - 500.; // Distância da origem até o far plane
        let fov_x: f32 = 135.0;
        let fov_x_rad = fov_x.to_radians();

        // A origem das coordenadas é no centro:
        //  ___________________
        // |       top         |
        // |                   |
        // |left  center  right| H
        // |                   |
        // |      bottom       |
        //  ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾
        //           W
        let right = dbg!(near.abs() * (fov_x_rad / 2.0).tan()); // right = W/2
        let left = -right; // left = -W/2
        let top = ratio.recip() * right; // top = H/2
        let bottom = -top; // bottom = -H/2

        assert!(fov_x < 180.0);
        assert!(near < 0.0);
        assert!(near > far);
        assert!(right > left);
        assert!(top > bottom);

        println!(
            "CameraWindow dimension ({} x {})",
            right - left,
            top - bottom
        );

        Self {
            position: pos, // Lookfrom
            direction: dir.normalized(),
            up_direction: Vec3::new([0., 1., 0.]), // Vup
            window_view: CameraWindow {
                top,
                bottom,
                right,
                left,
            },

            min_view_dist: near,
            max_view_dist: far,

            u: Vec3::zeros(),
            v: Vec3::zeros(),
            w: Vec3::zeros(),
        }
    }

    pub fn get_pos(&self) -> Vec3 {
        self.position
    }
    pub fn get_direction(&self) -> Vec3 {
        self.direction
    }

    fn get_window(&self) -> CameraWindow {
        self.window_view.clone()
    }

    pub fn get_min_view_dist(&self) -> f32 {
        self.min_view_dist
    }
    pub fn get_max_view_dist(&self) -> f32 {
        self.max_view_dist
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn move_up(&mut self, dist: f32) {
        self.position = self.position + self.up_direction * dist;
    }

    pub fn move_down(&mut self, dist: f32) {
        self.position = self.position + self.up_direction * (-dist);
    }

    pub fn move_left(&mut self, dist: f32) {
        self.position = self.position + self.u * (-dist);
    }

    pub fn move_right(&mut self, dist: f32) {
        self.position = self.position + self.u * dist;
    }

    pub fn move_foward(&mut self, dist: f32) {
        let foward = self.up_direction.cross(self.u).normalized();
        self.position = self.position + foward * dist;
    }

    pub fn move_backward(&mut self, dist: f32) {
        let backward = self.u.cross(self.up_direction).normalized();
        self.position = self.position + backward * dist;
    }

    pub fn rotate_origin(&mut self, theta: f32) {
        let new_pos = Matrix4::rotate_y(theta.to_radians()) * self.position.as_vec4();
        let new_dir = Matrix4::rotate_y(theta.to_radians()) * self.direction.as_vec4();

        self.position = new_pos.as_vec3();
        self.direction = new_dir.as_vec3();
    }

    pub fn get_basis_matrix(&self) -> Matrix4 {
        let u = self.u;
        let v = self.v;
        let w = self.w;

        assert!(u.norm() > 0. && v.norm() > 0. && w.norm() > 0.);

        Matrix4::new([
            [u.x(), v.x(), w.x(), 0.0],
            [u.y(), v.y(), w.y(), 0.0],
            [u.z(), v.z(), w.z(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn offset_screen_direction(&mut self, dx: f32, dy: f32) {
        self.direction = (self.direction + self.u * dx + self.v * dy).normalized();
    }

    pub fn rotate_by_offset(&mut self, dx: f32, dy: f32) {
        let offset = self.u * dx + self.v * dy;
    }

    pub fn update_basis(&mut self) {
        let g = self.direction;

        let w = (g / g.norm()) * (-1.0);

        let t_x_w = self.up_direction.cross(w);
        let u = t_x_w / t_x_w.norm();

        let v = w.cross(u);

        self.u = u.normalized();
        self.v = v.normalized();
        self.w = w.normalized();
    }

    pub fn gen_matrix(&mut self) -> Matrix4 {
        // A matriz View, que transforma pontos do espaço do mundo
        // para o espaço da câmera, é obtida da seguinte forma:
        // Primeiro, encontramos a matriz de transformação que translada
        // a câmera para a origem do mundo utilizando a posição da câmera.
        let pos = self.position;
        let matrix_pos = Matrix4::new([
            [1.0, 0.0, 0.0, -pos.x()],
            [0.0, 1.0, 0.0, -pos.y()],
            [0.0, 0.0, 1.0, -pos.z()],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        // Com as origens alinhadas, encontramos (u,v,w), os três vetores
        // unitários que formam a base da câmera e que não estão alinhados
        // com as coordenadas de mundo (são o x,y,z da câmera).
        self.update_basis();

        // A matriz de rotação que alinha os eixos da câmera com o mundo
        // é obtida pela transposta da matriz da base da câmera, que aplica
        // uma mudança de base do mundo para a câmera.
        let matrix_basis_transp = self.get_basis_matrix().transposed();

        // Multiplicando a translação com a rotação, obtemos a matriz View.
        let matrix_cam = matrix_basis_transp * matrix_pos;
        matrix_cam
    }

    fn gen_view_planes(&mut self) -> ([ViewPlane; 2], [ViewPlane; 4]) {
        self.update_basis();

        // Matriz para aplicar a rotação que leva coordenadas da câmera para o mundo.
        let matrix_basis = self.get_basis_matrix();

        let camera_pos = self.get_pos();
        let camera_window = self.get_window();

        let n = self.get_min_view_dist();
        let f = self.get_max_view_dist();
        let r = camera_window.right;
        let l = camera_window.left;
        let t = camera_window.top;
        let b = camera_window.bottom;

        // Os 4 vértices do near plane em coordenadas de câmera
        let upper_right_near_cam = Vec3::new([r, t, n]).as_vec4();
        let upper_left_near_cam = Vec3::new([l, t, n]).as_vec4();
        let lower_right_near_cam = Vec3::new([r, b, n]).as_vec4();
        let lower_left_near_cam = Vec3::new([l, b, n]).as_vec4();

        // Os 4 vértices do near plane em coordenadas de mundo com offset da posicao da camera
        let upper_right_near_world =
            (matrix_basis * upper_right_near_cam).vec3_over_w() + camera_pos;
        let upper_left_near_world = (matrix_basis * upper_left_near_cam).vec3_over_w() + camera_pos;
        let lower_right_near_world =
            (matrix_basis * lower_right_near_cam).vec3_over_w() + camera_pos;
        let lower_left_near_world = (matrix_basis * lower_left_near_cam).vec3_over_w() + camera_pos;

        let x_center = (l + r) / 2.; // = 0
        let y_center = (b + t) / 2.; // = 0

        // Os 4 vértices do far plane em coordenadas de câmera
        let upper_center_far_cam = Vec3::new([x_center, (f * t) / n, f]).as_vec4();
        let lower_center_far_cam = Vec3::new([x_center, (f * b) / n, f]).as_vec4();
        let right_center_far_cam = Vec3::new([(f * r) / n, y_center, f]).as_vec4();
        let left_center_far_cam = Vec3::new([(f * l) / n, y_center, f]).as_vec4();

        // Os 4 vértices do far plane em coordenadas de mundo com offset da posicao da camera
        let upper_center_far_world =
            (matrix_basis * upper_center_far_cam).vec3_over_w() + camera_pos;
        let lower_center_far_world =
            (matrix_basis * lower_center_far_cam).vec3_over_w() + camera_pos;
        let right_center_far_world =
            (matrix_basis * right_center_far_cam).vec3_over_w() + camera_pos;
        let left_center_far_world = (matrix_basis * left_center_far_cam).vec3_over_w() + camera_pos;

        // Um ponto arbitrário dentro do frustrum para testes.
        let visible_point = (upper_right_near_world + lower_center_far_world) / 2.0;

        let depth_planes = [
            ViewPlane::new(
                [
                    upper_right_near_world,
                    lower_right_near_world,
                    lower_left_near_world,
                ],
                visible_point,
                "near",
            ),
            ViewPlane::new(
                [
                    left_center_far_world,
                    right_center_far_world,
                    upper_center_far_world,
                ],
                visible_point,
                "far",
            ),
        ];

        let lateral_planes = [
            ViewPlane::new(
                [
                    right_center_far_world,
                    upper_right_near_world,
                    lower_right_near_world,
                ],
                visible_point,
                "right",
            ),
            ViewPlane::new(
                [
                    left_center_far_world,
                    lower_left_near_world,
                    upper_left_near_world,
                ],
                visible_point,
                "left",
            ),
            ViewPlane::new(
                [
                    upper_center_far_world,
                    upper_left_near_world,
                    upper_right_near_world,
                ],
                visible_point,
                "top",
            ),
            ViewPlane::new(
                [
                    lower_center_far_world,
                    lower_left_near_world,
                    lower_right_near_world,
                ],
                visible_point,
                "bottom",
            ),
        ];

        (depth_planes, lateral_planes)
    }
}

struct ViewPlane {
    normal: Vec3,
    k: f32,
    label: String,
}

impl ViewPlane {
    pub fn new(points: [Vec3; 3], visible_point: Vec3, label: &str) -> Self {
        let a_point = points[0];
        let b_point = points[1];
        let c_point = points[2];

        let p_vec = b_point - a_point;
        let q_vec = c_point - b_point;

        let mut normal = p_vec.cross(q_vec);
        let mut k = -normal.dot(a_point);

        let mut test_value = normal.dot(visible_point) + k;

        // a condicao de validez eh que a origem gere um valor positivo,
        // ou seja, ela esta dentro do volume de visao

        if test_value < 0.0 {
            normal = q_vec.cross(p_vec);
            k = -normal.dot(a_point);

            test_value = normal.dot(visible_point) + k;
            assert!(test_value > 0.0, "N'ao deu pra criar o plano");
        }

        Self {
            normal,
            k,
            label: label.to_string(),
        }
    }

    pub fn func(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.k
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }
    pub fn at_least_partially_visible(&self, tri: &Triangle) -> bool {
        let a_vertex = tri.vertices[0];
        let b_vertex = tri.vertices[1];
        let c_vertex = tri.vertices[2];

        let f_a = self.func(a_vertex);
        let f_b = self.func(b_vertex);
        let f_c = self.func(c_vertex);

        if f_a > 0.0 && f_b > 0.0 && f_c > 0.0 {
            // completly visible
            return true;
        } else if f_a <= 0.0 && f_b <= 0.0 && f_c <= 0.0 {
            // not visible
            return false;
        } else {
            // partially visible
            return true;
        }
    }

    pub fn clip(&self, tri: &Triangle, tri_pool_ret: &mut [Triangle]) -> usize {
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

        if f_a > 0.0 && f_b > 0.0 && f_c > 0.0 {
            // Todos os vértices dentro do volume,
            // não é necessário clipar.
            tri_pool_ret[0] = tri.clone();
            return 1;
        } else if f_a <= 0.0 && f_b <= 0.0 && f_c <= 0.0 {
            // Todos os vértices fora do volume,
            // não será exibido.
            return 0;
        }
        // Se prosseguiu, verificar a existência de
        // um vértice para fora do plano enquanto
        // os outros dois vértices estáo dentro.

        if f_a * f_c >= 0.0 {
            swap(&mut f_b, &mut f_c);
            swap(&mut b_vertex, &mut c_vertex);

            swap(&mut b_attr, &mut c_attr);

            swap(&mut f_a, &mut f_b);
            swap(&mut a_vertex, &mut b_vertex);

            swap(&mut a_attr, &mut b_attr);
        } else if f_b * f_c >= 0.0 {
            swap(&mut f_a, &mut f_c);
            swap(&mut a_vertex, &mut c_vertex);

            swap(&mut a_attr, &mut c_attr);

            swap(&mut f_a, &mut f_b);
            swap(&mut a_vertex, &mut b_vertex);

            swap(&mut a_attr, &mut b_attr);
        }

        // Calcular os pontos de interseção entre o plano e o triângulo
        // para a criação dos novos triângulos da clipagem
        let t_a = self.func(a_vertex) / self.normal().dot(a_vertex - c_vertex) - EPS;
        let new_vertex_a = a_vertex + (c_vertex - a_vertex) * t_a;
        let new_a_attr = a_attr + (c_attr - a_attr) * t_a;
        let t_b = self.func(b_vertex) / self.normal().dot(b_vertex - c_vertex) - EPS;
        let new_vertex_b = b_vertex + (c_vertex - b_vertex) * t_b;
        let new_b_attr = b_attr + (c_attr - b_attr) * t_b;

        // Apenas um vértice do triângulo para fora do plano
        if f_c <= 0.0 {
            let new_triangle_a = Triangle::new(
                [a_vertex, new_vertex_a, new_vertex_b],
                [a_attr, new_a_attr, new_b_attr],
            );

            let new_triangle_b = Triangle::new(
                [a_vertex, b_vertex, new_vertex_b],
                [a_attr, b_attr, new_b_attr],
            );

            tri_pool_ret[0] = new_triangle_a;
            tri_pool_ret[1] = new_triangle_b;
            return 2;
        } else {
            let new_triangle_c = Triangle::new(
                [c_vertex, new_vertex_a, new_vertex_b],
                [c_attr, new_a_attr, new_b_attr],
            );

            tri_pool_ret[0] = new_triangle_c;
            return 1;
        }
    }
}

pub struct Scene {
    width: usize,  // Comprimento da Janela
    height: usize, // Altura da Janela
    pub camera: Camera,
    objects: Vec<Object>,
    next_object_id: u32,

    light_source: Vec3,
}

impl Scene {
    pub fn new(width: usize, height: usize) -> Self {
        //let camera_pos = Vec3::new([0., 2., 4.]);
        //let camera_pos = Vec3::new([0., 300., 630.0]); // Aviao ae
        let camera_pos = Vec3::new([0., 0., 150.0]);
        let camera_dir = camera_pos * -1.0;

        let light_source = Vec3::new([0., 300., 300.]);

        let ratio = (width as f32) / (height as f32);
        let camera = Camera::new(camera_pos, camera_dir, ratio);

        // alguns modelos classicos
        // https://casual-effects.com/data/

        //let obj = Object::load_from_file("models/lemur/lemur.obj");
        //let obj_vec = Object::load_from_directory("models/dungeon_set/");

        Self {
            width,
            height,
            camera,
            objects: vec![],
            next_object_id: 0,

            light_source,
        }
    }

    pub fn add_obj(&mut self, obj: Object) -> ObjectInfo {
        let info = ObjectInfo {
            id: self.next_object_id,
            name: obj.name.clone(),
            mesh_info_list: obj.mesh_info_list(),
        };

        self.objects.push(obj);
        self.objects.last_mut().unwrap().id = Some(self.next_object_id);

        self.next_object_id += 1;

        info
    }

    pub fn move_camera_direction(&mut self, dx: i32, dy: i32) {
        assert!(dx < self.width as _);
        assert!(dy < self.height as _);

        self.camera.offset_screen_direction(
            dx as f32 / self.width as f32,
            dy as f32 / self.height as f32,
        );

        self.camera.update_basis();
    }

    pub fn rotate_camera_by_offset(&mut self, dx: i32, dy: i32) {}

    fn gen_transformation_matrix(&mut self) -> Matrix4 {
        let n_x: f32 = self.width as _; // Qnt. de pixeis do comprimento da janela
        let n_y: f32 = self.height as _; // Qnt. de pixeis da altura da janela

        let n = self.camera.get_min_view_dist();
        let f = self.camera.get_max_view_dist();

        let camera_window = self.camera.get_window();
        let r = camera_window.right;
        let l = camera_window.left;

        let t = camera_window.top;
        let b = camera_window.bottom;

        // A matriz View, que transforma pontos do espaço do mundo
        // para o espaço da câmera, é obtida da seguinte forma:
        // Primeiro, encontramos a matriz de transformação que translada
        // a câmera para a origem do mundo utilizando a posição da câmera.
        // Com as origens alinhadas, encontramos (u,v,w), os três vetores unitários
        // que formam a base da câmera e que não estão alinhados com as coordenadas
        // de mundo (são o x,y,z da câmera).
        // A matriz de rotação que alinha os eixos da câmera com o mundo é obtida
        // pela transposta da matriz da base da câmera, que aplica uma mudança de base
        // do mundo para a câmera.
        // Multiplicando a translação com a rotação, obtemos a matriz View.
        let matrix_cam = self.camera.gen_matrix();

        // A matriz de perspectiva mapeia o volume de visão perspectiva,
        // que é o frustum, para o volume de visão ortográfica, que é
        // uma caixa alinhada aos eixos a partir do plano near até o far.
        // Ela mantém os pontos no plano z = n inalterados e mapeia
        // o grande retângulo em z = f, na parte de trás do volume de perspectiva,
        // para o pequeno retângulo em z = f, na parte de trás do volume ortográfico.
        let persp = Matrix4::new([
            [n, 0.0, 0.0, 0.0],
            [0.0, n, 0.0, 0.0],
            [0.0, 0.0, (n + f), -(n * f)],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        // Note que é necessário fazer a desomogeneização após as transformações.

        // Do volume de visualização ortográfico para o volume de visualização
        // canônico (o cubo [-1,1]):
        // Fazemos a redimensionalização do volume ortográfico para o canônico.
        // Para isso, basta apenas alterar os limites dos volumes da matriz
        // dessa operação, com os limites do volume de origem sendo o retângulo
        // que se estende do near até o far plane, e os limites do volume de destino
        // sendo o cubo canônico.
        // Esse processo é similar a redimensionalização de janela da matriz ViewPort.
        let matrix_orth = Matrix4::new([
            [2.0 / (r - l), 0.0, 0.0, -(r + l) / (r - l)],
            [0.0, 2.0 / (t - b), 0.0, -(t + b) / (t - b)],
            [0.0, 0.0, 2.0 / (n - f), -(n + f) / (n - f)],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        // Do espaço canônico [-1,1] para o espaço de tela (janela):
        //        ___________________(1,1)         ___________________(width,height)
        //       |                   |            |                   |
        //       |        ^          |            |                   |
        //       |        │          |     =>     |                   |
        //       |        └───>      |            |                   |
        //       |                   |            ^                   |
        //       |                   |            │                   |
        // (-1,-1)‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾         (0,0)───>───────────────
        // Primeiro, aplicar a translação T(1,1), que posicionará o espaço canônico
        // na origem, obtendo o espaço dentro dos pontos extremos (0,0) e (2,2).
        // Em seguida, aplicar a escala S(width/2,height/2), que redimensionará
        // o espaço para o mesmo tamanho da janela.
        // O último passo seria posicionar o espaço na origem da janela,
        // mas como ele já está em (0,0), a translação T(0,0) não é necessária.
        // M_vp = S(width/2,height/2) * T(1,1)
        let matrix_viewport = Matrix4::new([
            [n_x / 2.0, 0.0, 0.0, (n_x - 1.0) / 2.0],
            [0.0, n_y / 2.0, 0.0, (n_y - 1.0) / 2.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let matrix_transf = matrix_viewport * matrix_orth * persp * matrix_cam;

        matrix_transf
    }

    pub fn render(&mut self, canvas: &mut Canvas) {
        canvas.clear();

        let matrix_transf = self.gen_transformation_matrix();

        let camera_pos = self.camera.get_pos();

        let func_planes = self.camera.gen_view_planes();

        for obj in self.objects.iter_mut() {
            let obj_vertices = &obj.vertices;
            let obj_normals = &obj.normals_vertices;
            let obj_texture_uv = obj.texture_vertices.as_ref().unwrap();

            // Calcular VertexAttributes aq para remover cálculos redundantes

            for (vertex, visual_info) in
                obj.vertices.iter().zip(obj.vertices_visual_info.iter_mut())
            {
                let eye_dir = *vertex - camera_pos;

                visual_info.light = (*vertex - self.light_source).normalized();
                visual_info.eye = eye_dir.normalized();
                visual_info.depth = eye_dir.norm() as _;
                visual_info.halfway = (visual_info.light + visual_info.eye).normalized();
            }

            canvas.enable_depth_update();
            for obj_mesh in obj.opaque_meshes.iter() {
                let mesh_texture_idx = obj_mesh.texture_idx.unwrap();
                // TODO: ta meio feio isso aq, tem que embelezar.
                // criar um iterador no futuro tlvz
                for (vertex_tri_idx, texture_tri_idx, normal_tri_idx) in obj_mesh.triangles.iter() {
                    let indexed_tri_vertex = vertex_tri_idx.clone();
                    let indexed_tri_normal = normal_tri_idx.clone();
                    let indexed_tri_texture = texture_tri_idx.clone();

                    let tri_vertices =
                        IndexedMesh::vec3_list_from_indexed(indexed_tri_vertex, obj_vertices);
                    let tri_normals =
                        IndexedMesh::vec3_list_from_indexed(indexed_tri_normal, obj_normals);
                    let tri_textures =
                        IndexedMesh::vec3_list_from_indexed(indexed_tri_texture, obj_texture_uv);

                    let a_vertex_idx = indexed_tri_vertex[0];
                    let b_vertex_idx = indexed_tri_vertex[1];
                    let c_vertex_idx = indexed_tri_vertex[2];

                    let a_vertex = tri_vertices[0];
                    let b_vertex = tri_vertices[1];
                    let c_vertex = tri_vertices[2];

                    let a_normal = tri_normals[0];
                    let b_normal = tri_normals[1];
                    let c_normal = tri_normals[2];

                    let a_texture_coord = tri_textures[0];
                    let b_texture_coord = tri_textures[1];
                    let c_texture_coord = tri_textures[2];

                    let a_light = obj.vertices_visual_info[a_vertex_idx].light;
                    let b_light = obj.vertices_visual_info[b_vertex_idx].light;
                    let c_light = obj.vertices_visual_info[c_vertex_idx].light;

                    //let a_eye = obj.vertices_visual_info[a_vertex_idx].eye;
                    //let b_eye = obj.vertices_visual_info[b_vertex_idx].eye;
                    //let c_eye = obj.vertices_visual_info[c_vertex_idx].eye;

                    let a_halfway = obj.vertices_visual_info[a_vertex_idx].halfway;
                    let b_halfway = obj.vertices_visual_info[b_vertex_idx].halfway;
                    let c_halfway = obj.vertices_visual_info[c_vertex_idx].halfway;

                    let a_depth = obj.vertices_visual_info[a_vertex_idx].depth;
                    let b_depth = obj.vertices_visual_info[b_vertex_idx].depth;
                    let c_depth = obj.vertices_visual_info[c_vertex_idx].depth;

                    let a_attr = VertexAttributes::new(
                        Vec2::new(0., 0.),
                        //Color::Green,
                        a_depth,
                        a_normal,
                        a_light,
                        //a_eye,
                        a_halfway,
                        a_texture_coord,
                    );

                    let b_attr = VertexAttributes::new(
                        Vec2::new(0., 0.),
                        //Color::Green,
                        b_depth,
                        b_normal,
                        b_light,
                        //b_eye,
                        b_halfway,
                        b_texture_coord,
                    );

                    let c_attr = VertexAttributes::new(
                        Vec2::new(0., 0.),
                        //Color::Green,
                        c_depth,
                        c_normal,
                        c_light,
                        //c_eye,
                        c_halfway,
                        c_texture_coord,
                    );

                    let original_tri = Triangle::new(
                        [a_vertex, b_vertex, c_vertex],
                        [a_attr, b_attr, c_attr],
                        //Color::Green,
                    );

                    let tri_normal = Triangle::calc_normal(&original_tri);
                    let tri_eye = camera_pos - original_tri.get_center();

                    // Back-face culling
                    if tri_eye.dot(tri_normal) <= 0.0 {
                        // Renderizamos modelos poligonais fechados em que faces que
                        // não estão viradas para a câmera são sobrepostas por faces
                        // que estão viradas para a câmera. Portanto, se o ângulo entre
                        // o vetor que sai do triângulo em direção à câmera e a normal
                        // do triângulo for maior do que 90 graus, o triângulo não é renderizado.
                        continue;
                    }

                    // TODO: substituir por heapless::Vec
                    // 12 é o número máximo possível de triângulos gerados após clipagem entre os 6 planos.
                    let mut clipped_triangles: [Triangle; 12] = [Triangle::zeroed(); 12];
                    let clipped_count = original_tri
                        .clip_against_planes(&func_planes, clipped_triangles.as_mut_slice());
                    for clipped_tri in clipped_triangles[..clipped_count].iter_mut() {
                        // TODO: (performance) mover para esse laço a criação dos VertexAttr's.
                        // 1) O caso onde o triangulo nao eh clipado -> VisualInfo já está
                        // pré-calculado.
                        // 2) O caso onde o triangulo nao eh totalmente clipado -> Recalcular
                        // VisualInfo.
                        // Justificativa: A maioria dos triangulos do mundo ou será totalmente
                        // clippada ou nao será clippada. Por isso, teoricamente, vale a pena
                        // recalcular os a VisualInfo triangulos parcialmente clippados, para
                        // evitar as cópias dessas structs na funcao de clipping

                        // Aplica todas as transformações que levam o ponto de coordenadas
                        // de mundo (clipped_tri.vertices) para coordenadas de janela.
                        let a_vec4 = matrix_transf * clipped_tri.vertices[0].as_vec4();
                        let b_vec4 = matrix_transf * clipped_tri.vertices[1].as_vec4();
                        let c_vec4 = matrix_transf * clipped_tri.vertices[2].as_vec4();

                        let a_w = a_vec4.get_w();
                        let b_w = b_vec4.get_w();
                        let c_w = c_vec4.get_w();

                        // Aplica a desomogeneização.
                        let a_coord = a_vec4.as_vec2() / a_w;
                        let b_coord = b_vec4.as_vec2() / b_w;
                        let c_coord = c_vec4.as_vec2() / c_w;

                        let clip_tri_vert_attr = &mut clipped_tri.vertices_attr;
                        clip_tri_vert_attr[0].screen_coord = a_coord;
                        clip_tri_vert_attr[1].screen_coord = b_coord;
                        clip_tri_vert_attr[2].screen_coord = c_coord;
                        // vis'ao ortogonal
                        /*
                        let camera_dir = self.camera.get_direction().normalized();
                        let a_depth: f32 = camera_dir.dot(camera_pos - tri.points[0]).abs() as _;
                        let b_depth: f32 = camera_dir.dot(camera_pos - tri.points[1]).abs() as _;
                        let c_depth: f32 = camera_dir.dot(camera_pos - tri.points[2]).abs() as _;
                        */

                        let mesh_texture = match obj.textures.get(mesh_texture_idx) {
                            Some(texture) => texture,
                            None => &Texture::default(),
                        };

                        canvas.draw_triangle_with_attributes(
                            &clip_tri_vert_attr[0],
                            &clip_tri_vert_attr[1],
                            &clip_tri_vert_attr[2],
                            mesh_texture,
                            None,
                        );
                    }
                }
            }

            canvas.disable_depth_update();

            for obj_mesh in obj.transparent_meshes.iter_mut() {
                let mesh_texture_idx = obj_mesh.texture_idx.unwrap();

                // sort the triangles of the transparent meshes (painter algorithm)
                // TODO: essa ordenação precisa ser aplicada a todos os triangulos
                // de todos os objetos transparentes de maneira absoluta.
                // Talvez seja possivel apenas manter essa ordenação atual, relativa
                // a cada mesh, e complementar com a ordenação prévia das meshes
                // transparentes pela sua posição.

                obj_mesh.triangles.as_mut_slice().sort_by(|a, b| {
                    let (a_vert_tri, _, _) = a;
                    let (b_vert_tri, _, _) = b;

                    let a_verts = IndexedMesh::vec3_list_from_indexed(*a_vert_tri, obj_vertices);

                    let b_verts = IndexedMesh::vec3_list_from_indexed(*b_vert_tri, obj_vertices);

                    let a_center = (a_verts[0] + a_verts[1] + a_verts[2]) / 3.0;
                    let b_center = (b_verts[0] + b_verts[1] + b_verts[2]) / 3.0;

                    let a_depth = a_center.dist(camera_pos);
                    let b_depth = b_center.dist(camera_pos);

                    a_depth.total_cmp(&b_depth).reverse()
                });

                for (vertex_tri_idx, texture_tri_idx, normal_tri_idx) in obj_mesh.triangles.iter() {
                    let indexed_tri_vertex = vertex_tri_idx.clone();
                    let indexed_tri_normal = normal_tri_idx.clone();
                    let indexed_tri_texture = texture_tri_idx.clone();

                    let tri_vertices =
                        IndexedMesh::vec3_list_from_indexed(indexed_tri_vertex, obj_vertices);
                    let tri_normals =
                        IndexedMesh::vec3_list_from_indexed(indexed_tri_normal, obj_normals);
                    let tri_textures =
                        IndexedMesh::vec3_list_from_indexed(indexed_tri_texture, obj_texture_uv);

                    let a_vertex_idx = indexed_tri_vertex[0];
                    let b_vertex_idx = indexed_tri_vertex[1];
                    let c_vertex_idx = indexed_tri_vertex[2];

                    let a_vertex = tri_vertices[0];
                    let b_vertex = tri_vertices[1];
                    let c_vertex = tri_vertices[2];

                    let a_normal = tri_normals[0];
                    let b_normal = tri_normals[1];
                    let c_normal = tri_normals[2];

                    let a_texture_coord = tri_textures[0];
                    let b_texture_coord = tri_textures[1];
                    let c_texture_coord = tri_textures[2];

                    let a_light = obj.vertices_visual_info[a_vertex_idx].light;
                    let b_light = obj.vertices_visual_info[b_vertex_idx].light;
                    let c_light = obj.vertices_visual_info[c_vertex_idx].light;

                    //let a_eye = obj.vertices_visual_info[a_vertex_idx].eye;
                    //let b_eye = obj.vertices_visual_info[b_vertex_idx].eye;
                    //let c_eye = obj.vertices_visual_info[c_vertex_idx].eye;

                    let a_halfway = obj.vertices_visual_info[a_vertex_idx].halfway;
                    let b_halfway = obj.vertices_visual_info[b_vertex_idx].halfway;
                    let c_halfway = obj.vertices_visual_info[c_vertex_idx].halfway;

                    let a_depth = obj.vertices_visual_info[a_vertex_idx].depth;
                    let b_depth = obj.vertices_visual_info[b_vertex_idx].depth;
                    let c_depth = obj.vertices_visual_info[c_vertex_idx].depth;

                    let a_attr = VertexAttributes::new(
                        Vec2::new(0., 0.),
                        //Color::Green,
                        a_depth,
                        a_normal,
                        a_light,
                        //a_eye,
                        a_halfway,
                        a_texture_coord,
                    );

                    let b_attr = VertexAttributes::new(
                        Vec2::new(0., 0.),
                        //Color::Green,
                        b_depth,
                        b_normal,
                        b_light,
                        //b_eye,
                        b_halfway,
                        b_texture_coord,
                    );

                    let c_attr = VertexAttributes::new(
                        Vec2::new(0., 0.),
                        //Color::Green,
                        c_depth,
                        c_normal,
                        c_light,
                        //c_eye,
                        c_halfway,
                        c_texture_coord,
                    );

                    let original_tri = Triangle::new(
                        [a_vertex, b_vertex, c_vertex],
                        [a_attr, b_attr, c_attr],
                        //Color::Green,
                    );

                    // TODO: substituir por heapless::Vec
                    let mut clipped_triangles: [Triangle; 12] = [Triangle::zeroed(); 12];
                    let clipped_count = original_tri
                        .clip_against_planes(&func_planes, clipped_triangles.as_mut_slice());
                    for clipped_tri in clipped_triangles[..clipped_count].iter_mut() {
                        // Aplica todas as transformações que levam o ponto de coordenadas
                        // de mundo (clipped_tri.vertices) para coordenadas de janela.
                        let a_vec4 = matrix_transf * clipped_tri.vertices[0].as_vec4();
                        let b_vec4 = matrix_transf * clipped_tri.vertices[1].as_vec4();
                        let c_vec4 = matrix_transf * clipped_tri.vertices[2].as_vec4();

                        let a_w = a_vec4.get_w();
                        let b_w = b_vec4.get_w();
                        let c_w = c_vec4.get_w();

                        // Aplica a desomogeneização.
                        let a_coord = a_vec4.as_vec2() / a_w;
                        let b_coord = b_vec4.as_vec2() / b_w;
                        let c_coord = c_vec4.as_vec2() / c_w;

                        let clip_tri_vert_attr = &mut clipped_tri.vertices_attr;
                        clip_tri_vert_attr[0].screen_coord = a_coord;
                        clip_tri_vert_attr[1].screen_coord = b_coord;
                        clip_tri_vert_attr[2].screen_coord = c_coord;

                        /*
                             // vis'ao ortogonal
                             let camera_dir = self.camera.get_direction().normalized();
                             let a_depth: f32 = camera_dir.dot(camera_pos - tri.points[0]).abs() as _;
                             let b_depth: f32 = camera_dir.dot(camera_pos - tri.points[1]).abs() as _;
                             let c_depth: f32 = camera_dir.dot(camera_pos - tri.points[2]).abs() as _;
                        */

                        let mesh_texture = match obj.textures.get(mesh_texture_idx) {
                            Some(texture) => texture,
                            None => &Texture::default(),
                        };

                        canvas.draw_triangle_with_attributes(
                            &clip_tri_vert_attr[0],
                            &clip_tri_vert_attr[1],
                            &clip_tri_vert_attr[2],
                            mesh_texture,
                            None,
                        );
                    }
                }
            }
        }
    }

    pub fn draw_indexed_mesh(&mut self, mesh: &IndexedMesh) {}
}
