use super::canvas::{
    Canvas,
    Color,
    VertexAttributes,
};

use super::linalg::{
    Vec2,
    Vec3,
    Vec4,
    Matrix4,
    EPS,
};

use obj;


#[derive(Clone, Copy, Debug)]
struct Triangle {
    vertices:       [Vec3; 3],
    vertices_attr:  [VertexAttributes; 3],
    //normal:         Vec3,
    //color:          Color,
}

impl Triangle {
    pub
    fn new (vertices:      [Vec3; 3],
            vertices_attr: [VertexAttributes; 3], 
            //color: Color
            ) -> Self
    {
        Self {
            vertices:       vertices,
            vertices_attr:  vertices_attr,
            //color:          color,
            //normal:         Vec3::zeros(),
        }
    }

    fn zeroed () -> Self {
        unsafe {std::mem::MaybeUninit::<Triangle>::zeroed().assume_init()}
        /*
        Triangle::new(
            [
                Vec3::zeros(), 
                Vec3::zeros(), 
                Vec3::zeros()
            ],
            [
                VertexAttributes::zeros(), 
                VertexAttributes::zeros(), 
                VertexAttributes::zeros()
            ],
            Color::White,
        )
        */
    }
    
    pub
    fn calc_normal(tri: &Self) -> Vec3 {
        let a = tri.vertices[0];
        let b = tri.vertices[1];
        let c = tri.vertices[2];

        let p = b - a;
        let q = c - b;

        let normal = p.cross(q);

        normal
    }

    pub
    fn clip_against_planes(&self, view_planes: &[ViewPlane], tri_pool_ret: &mut [Triangle]) -> usize
    {

        let mut tri_pool_size: usize = 0;

        tri_pool_ret[tri_pool_size] = self.clone();
        tri_pool_size += 1;

        assert!(tri_pool_ret.len() >= 12);
        //let mut new_tri_pool: [Triangle; 8] = [Triangle::zeroed(); 8];
        let mut new_tri_pool: [Triangle; 12] = unsafe {std::mem::MaybeUninit::<[Triangle; 12]>::zeroed().assume_init()};
        
        // TODO: otimizar isso aq dsp (tlvz substituir por MaybeUninit zeroed etc tqv)
        //new_tri_pool.copy_from_slice(tri_pool_ret);

        let mut tri_pool_ref: &mut [Triangle] = tri_pool_ret;
        let mut new_pool_ref: &mut [Triangle] = new_tri_pool.as_mut();

        for plane in view_planes.iter() {
            let mut new_tri_pool_size: usize = 0;
            //let mut new_tri_pool:  Vec<Self> = Vec::with_capacity(tri_pool.len() * 2);

            for tri in tri_pool_ref[0..tri_pool_size].iter() {

                let clipped_count = plane.clip(tri, new_pool_ref[new_tri_pool_size..].as_mut());

                new_tri_pool_size += clipped_count;
            }

            /*
            for tri in tri_pool.iter() {
                let mut clipped_triangles = plane.clip(tri);
                new_tri_pool.append(&mut clipped_triangles);
            }
            */




            //tri_pool_ret[..new_tri_pool_size].copy_from_slice(new_tri_pool[..new_tri_pool_size].as_mut());
            //tri_pool_size = new_tri_pool_size;
            
            std::mem::swap(&mut tri_pool_ref, &mut new_pool_ref);
            std::mem::swap(&mut tri_pool_size, &mut new_tri_pool_size);
        }

        return tri_pool_size;

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

trait IndexedTriangleNormal {
    fn calc_normal(&self, vertices: &Vec<Vec3>) -> Vec3;
}

type IndexedTriangle = [usize; 3];

impl IndexedTriangleNormal for IndexedTriangle {

    fn calc_normal(&self, vertices: &Vec<Vec3>) -> Vec3 {
        let a = vertices[self[0]];
        let b = vertices[self[1]];
        let c = vertices[self[2]];

        let p = b - a;
        let q = c - b;

        let normal = p.cross(q);

        normal
    }
}

#[derive(Clone)]
pub
struct IndexedMesh {
    triangles:  Vec<(IndexedTriangle, IndexedTriangle, IndexedTriangle)>,
  //triangles:          Vec<IndexedTriangle>,
  //texture_triangles:  Vec<IndexedTriangle>,
  //normals_triangles:  Vec<IndexedTriangle>,


    texture_idx:       Option<usize>,
}

impl IndexedMesh {
    /*
    pub
    fn new (tri_vec: &Vec<(IndexedTriangle, IndexedTriangle, IndexedTriangle)>, vertex_vec: &Vec<Vec3>, text_idx: usize) -> Self {
        // se algum triangulo nao possuir textura entao -> texture_idx = None
        //
    }
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

    pub
    fn vec3_list_from_indexed(indexed_tri: IndexedTriangle, vert_list: &Vec<Vec3>) -> [Vec3; 3] {
        let a_idx = indexed_tri[0];
        let b_idx = indexed_tri[1];
        let c_idx = indexed_tri[2];

        assert!(vert_list.len() > a_idx);
        assert!(vert_list.len() > b_idx);
        assert!(vert_list.len() > c_idx);

        unsafe {
            let a_vert = *vert_list.get_unchecked(a_idx);
            let b_vert = *vert_list.get_unchecked(b_idx);
            let c_vert = *vert_list.get_unchecked(c_idx);

            [a_vert, 
            b_vert, 
            c_vert]
        }
    }

}

pub
struct TextureMap {
    img: Vec<u8>,
    width: usize,
    height: usize,
    components: usize,

    f_width: f32,
    f_height: f32,
}

impl TextureMap {
    pub
    fn new (img: Vec<u8>, width: usize, height: usize, components: usize) -> Self {
        assert!(img.len() % components == 0);
        assert!(img.len() / components == width * height);

        Self {
            img:        img,
            width:      width,
            height:     height,
            components: components,

            f_width:  ( width -1) as f32,
            f_height: ( height-1) as f32,
        }
    }

    pub
    fn default () -> Self {
        Self ::new(
            Vec::from(Color::White.as_slice()),
            1,  // width
            1,  // height
            3,  // components
        )
    }

    pub
    fn get_rgb_slice(&self, u: f32, v: f32) -> [u8; 3] {
        debug_assert!(0.0 <= u && u <= 1.0);
        debug_assert!(0.0 <= v && v <= 1.0);

        let u_idx =                  (u * self.f_width).floor()  as usize;
        let v_idx = self.height -1 - (v * self.f_height).floor() as usize;
        
        let offset = (v_idx * self.width + u_idx) * self.components;

        debug_assert!(offset     <  self.img.len());
        debug_assert!(offset + 3 <= self.img.len());

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
    fn load_from_file (file_path: &std::path::PathBuf) -> Self {
        use std::fs::File;
        use std::io::Seek;
        use stb::image::stbi_load_from_reader;
        use stb::image::stbi_info_from_reader;
        use stb::image::Channels;

        //let path = Path::new(filename);

        let mut file = File::open(file_path).expect("Unable to open file");
        let pre_info = stbi_info_from_reader(&mut file)
                            .expect("Deu errado ler a textura");

        _ = file.rewind();

        let channels = match dbg!(pre_info.components) {
            3 => Channels::Rgb,
            4 => Channels::RgbAlpha,
            _ => unreachable!(),
        };

        dbg!(channels);
        let (info, img) = stbi_load_from_reader(&mut file, channels)
                            .expect("Deu errado ler a textura");

        Self::new(
            Vec::from(img.as_slice()),
            info.width      as usize,
            info.height     as usize,
            info.components as usize
        )
    }

}

// https://paulbourke.net/dataformats/mtl/

// isso eh um obj mtl
pub
struct Texture {
    pub name: String,
    pub ka: Vec3,
    pub kd: Vec3,
    pub ks: Vec3,

    pub alpha: f32,

    pub map_ka:      TextureMap,
    pub map_kd:      TextureMap,
}

impl Texture {
    fn default() -> Self {
        let map_ka = TextureMap::default();
        let map_kd = TextureMap::default();

        Self {
            name: String::from("default"),
            ka: Vec3::new([0.9, 0.9, 0.9]),
            kd: Vec3::new([0.4, 0.4, 0.4]),
            ks: Vec3::new([0.5, 0.5, 0.5]),

            alpha: 1.0,

            map_ka: map_ka,
            map_kd: map_kd,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VertexVisual {
    light:  Vec3,
    eye:    Vec3,
    halfway:    Vec3,
    depth:  f32,
}

impl VertexVisual {
    fn zeroed () -> Self {
        Self {
            light:  Vec3::zeros(),
            eye:    Vec3::zeros(),
            halfway:    Vec3::zeros(),
            depth:  0.0,
        }
    }
}

// 1. conter no obeto vetor de meshes de triangulos
// 2. cada mesh pode referenciar um meterial especifico
// 3. os dados dos vetores/vertices/texturas serao armazenados pelo struct Object
// 4. as malhas armazenar'ao as infos de materiasi e os indices de vetores/vertices/textura

pub
struct Object {
    vertices:           Vec<Vec3>,

    normals_vertices:   Vec<Vec3>,
    texture_vertices:   Option<Vec<Vec3>>,

    vertices_visual_info: Vec<VertexVisual>,

    opaque_meshes:      Vec<IndexedMesh>,
    transparent_meshes: Vec<IndexedMesh>,

    textures: Vec<Texture>,
}

impl Object {
    pub
    fn new (vertices:           Vec<Vec3>,
            normals_vertices:   Vec<Vec3>,
            texture_vertices:   Option<Vec<Vec3>>,
            meshes:             Vec<IndexedMesh>,
            textures:           Vec<Texture>) -> Self 
    {

        let mut opaque:      Vec<IndexedMesh> = Vec::new();
        let mut transparent: Vec<IndexedMesh> = Vec::new();

        for mesh in meshes.iter() {
            let texture_idx = mesh.texture_idx.unwrap();

            if textures[texture_idx].alpha < 1.0 {
                transparent.push(mesh.clone());
            } else {
                opaque.push(mesh.clone());
            }
        }

        let vert_total = vertices.len();

        Self {
            vertices:           vertices,
            normals_vertices:   normals_vertices,
            texture_vertices:   texture_vertices,

            vertices_visual_info: vec![VertexVisual::zeroed(); vert_total],

            opaque_meshes:      opaque,
            transparent_meshes: transparent,

            textures:           textures,
        }
    }

    pub
    fn load_from_directory(dir: &str) -> Vec<Self> {
        let file_ext = "obj";
        let path = std::path::Path::new(dir);

        if path.is_dir() == false {
            eprintln!("Invalid directory path: {}", path.display());
            return Vec::new();
        }

        // Lê o diretório
        let entries = std::fs::read_dir(path).unwrap();

        let mut obj_vec = Vec::new();

        // Itera sobre as entradas do diretório
        for entry in entries {
            if entry.is_ok() {
                let entry = entry.unwrap();
                let path = entry.path();

                // Verifica se o caminho é um arquivo e tem a extensão específica
                if path.is_file() && path.extension().map_or(false, |e| e == file_ext) {
                    println!("{}", path.display());
                    
                    obj_vec.push(Self::load_from_file(path.to_str().unwrap()))
                }
            }
        }

        obj_vec
    }

    // TODO: utilizar Result no retorno da funcao
    pub
    fn load_from_file(filename: &str) -> Self {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::{Path, PathBuf};

        let path = Path::new(filename);
        let parent_dir = path.parent();

        let add_file_path = |filename: &String| -> PathBuf {
            if parent_dir.is_some() {
                [
                    parent_dir.unwrap().to_str().expect("Failed adding file path."),
                    filename.as_str()
                ].iter().collect()
            } else {
                PathBuf::from(filename.as_str())
            }
        };

        let file = File::open(&path);
        assert!(file.is_ok(), "Unable to open file {}", filename);
        let reader = BufReader::new(file.unwrap());

        let mut obj_data = obj::ObjData::load_buf(reader).unwrap();

        obj_data.material_libs.iter_mut().for_each(
            |mtllib| {
                let mtl_path: PathBuf = add_file_path(&mtllib.filename);
                let fname = mtllib.filename.as_str();
                let file = File::open(&mtl_path);
                assert!(file.is_ok(), "Unable to open file {}", fname);
                _ = mtllib.reload(file.unwrap());
            }
        );



        let mut obj_vertices:    Vec<Vec3> = obj_data.position.iter().map(|e| Vec3::new([e[0], e[1], e[2]]))             .collect::<_>();
        let mut obj_normals:     Vec<Vec3> = obj_data.normal  .iter().map(|e| Vec3::new([e[0], e[1], e[2]]).normalized()).collect::<_>();
        let mut obj_texture_uv:  Vec<Vec3> = obj_data.texture .iter().map(|e| Vec3::new([e[0], e[1], 0.0]))              .collect::<_>();


        // rescaling test
        let mut vertices_sorted = 
        obj_vertices.iter()
                    .map(|e| e.norm())
                    .collect::<Vec<_>>();
        vertices_sorted.as_mut_slice()
                    .sort_by(|a, b| a.partial_cmp(b).unwrap().reverse());
        let vertex_max = vertices_sorted.get(0).unwrap();
        dbg!(vertex_max);
        
        let scale = 100.0;
        let factor = scale / vertex_max;
        obj_vertices.iter_mut().for_each(|e| *e = *e * factor);
                    


        let mut textures: Vec<Texture> = Vec::from([Texture::default()]);

        let mut meshes: Vec<IndexedMesh> = Vec::new();

        //assert!(obj_data.objects.len() == 1);
        for mtl in obj_data.material_libs.iter() {
            for material in mtl.materials.iter() {
                let name = material.name.clone();
                println!("material {:?}", name);

                let ka = material.ka.as_ref().unwrap();
                let kd = material.kd.as_ref().unwrap();
                let ks = material.ks.as_ref().unwrap();
                let alpha = material.d.as_ref().unwrap_or_else(|| &1.0);

                let map_ka = 
                    if let Some(map_ka_filename) = material.map_ka.as_ref() {
                        println!("{}", map_ka_filename);
                        let f_path = add_file_path(map_ka_filename);
                        TextureMap::load_from_file(&f_path)
                    } else {
                        TextureMap::default()
                    };

                let map_kd = 
                    if let Some(map_kd_filename) = material.map_kd.as_ref() {
                        println!("{}", map_kd_filename);
                        let f_path = add_file_path(map_kd_filename);
                        TextureMap::load_from_file(&f_path)
                    } else {
                        TextureMap::default()
                    };


                println!("ambient {:?}", ka);
                println!("difuse {:?}", kd);
                println!("specular {:?}", ks);
                println!("d {}", alpha);

                textures.push(
                    Texture {
                        name: name,

                        ka: Vec3::new(*ka),
                        kd: Vec3::new(*kd),
                        ks: Vec3::new(*ks),

                        alpha: *alpha,

                        map_ka: map_ka,
                        map_kd: map_kd,
                    }
                );
                
                //let _ = material.map_kd.as_ref().unwrap();

            }
        }

        //unreachable!();

        for obj in obj_data.objects.iter() {
            println!("Object {}", obj.name);


            for group in obj.groups.iter() {
                // Group doesnt has faces
                if group.polys.is_empty() {
                    continue;
                }

                let mut group_mesh_triangles: Vec<(IndexedTriangle, Option<IndexedTriangle>, Option<IndexedTriangle>)> = Vec::new();
              //let mut mesh_faces:         Vec<IndexedTriangle> = Vec::new();
              //let mut mesh_texture_faces: Vec<IndexedTriangle> = Vec::new();
              //let mut mesh_normals_faces: Vec<IndexedTriangle> = Vec::new();

                println!("\t Group name     {}", group.name);
                println!("\t Group material {:?}", group.material);

                let material_name = 
                if let Some(material) = &group.material {
                    match material {
                        obj::ObjMaterial::Ref(material_name) => {
                            //println!("\tGroup material (Ref) {:?}", material_name);
                            material_name.clone()
                            
                        },

                        obj::ObjMaterial::Mtl(material_arc) => {
                            //println!("\tGroup material (Arc) {:?}", material_arc.name);
                            material_arc.name.clone()
                        },
                    }
                } else {
                    //unreachable!();

                    String::from("default")
                };

                let mut mesh_missing_texture = false;
                let mut mesh_missing_normals = false;

                for face in group.polys.iter() {
                    let face_vec = &face.0;
                    let mut vertex_index:  Vec<usize>         = Vec::new();
                    let mut texture_index: Vec<Option<usize>> = Vec::new();
                    let mut normals_index: Vec<Option<usize>> = Vec::new();

                    for vertex_tuple in face_vec.iter() {

                        vertex_index.push(vertex_tuple.0);

                        if vertex_tuple.1.is_some() {
                            texture_index.push(vertex_tuple.1);
                        } else {
                            texture_index.push(None);
                        }

                        if vertex_tuple.2.is_some() {
                            normals_index.push(vertex_tuple.2);
                        } else {
                            normals_index.push(None);
                        }
                    }

                    let face_missing_texture = texture_index.contains(&None);
                    let face_missing_normals = normals_index.contains(&None);

                    if face_missing_texture {
                        mesh_missing_texture = true;
                    }

                    if face_missing_normals {
                        mesh_missing_normals = true;
                    }

                    let vertex_idx_a = vertex_index[0];
                    let vertex_idx_b = vertex_index[1];
                    let vertex_idx_c = vertex_index[2];

                    let texture_idx_a = texture_index[0];
                    let texture_idx_b = texture_index[1];
                    let texture_idx_c = texture_index[2];

                    let normals_idx_a = normals_index[0];
                    let normals_idx_b = normals_index[1];
                    let normals_idx_c = normals_index[2];

                    if face_vec.len() >= 3 {



                        group_mesh_triangles.push((
                            // position vertices
                            [
                                vertex_idx_a,
                                vertex_idx_b,
                                vertex_idx_c
                            ],

                            // texture vertices
                            if face_missing_texture == false
                            {
                                Some([
                                    texture_idx_a.unwrap(),
                                    texture_idx_b.unwrap(),
                                    texture_idx_c.unwrap()
                                ])
                            } else { None },

                            // normal vertices
                            if face_missing_normals == false
                            {
                                Some([
                                    normals_idx_a.unwrap(),
                                    normals_idx_b.unwrap(),
                                    normals_idx_c.unwrap()
                                ])
                            } else { None }
                        ));

                    }

                    if face_vec.len() == 4 {

                        let vertex_idx_d  = vertex_index[3];
                        let texture_idx_d = texture_index[3];
                        let normals_idx_d = normals_index[3];

                        group_mesh_triangles.push((
                            [
                                vertex_idx_c,
                                vertex_idx_d,
                                vertex_idx_a
                            ],
                            
                            // texture vertices
                            if face_missing_texture == false
                            {
                                Some([
                                    texture_idx_c.unwrap(),
                                    texture_idx_d.unwrap(),
                                    texture_idx_a.unwrap()
                                ])
                            } else { None },

                            // normal vertices
                            if face_missing_normals == false
                            {
                                Some([
                                    normals_idx_c.unwrap(),
                                    normals_idx_d.unwrap(),
                                    normals_idx_a.unwrap()
                                ])
                            } else { None }
                        ));

                    } else if face_vec.len() > 4 {
                        todo!();
                    }
                }
                
                let mut texture_idx_match: Option<usize> = Some(0);

                if mesh_missing_texture {
                    // add dummy global texture coord
                    // (PT): isso aq serve pra ter oque armazenar no struct VertexAttributes dps,
                    // pra nao precisar ficar usando um monte de Option por ai
                    let new_dummy_texture_indx = obj_texture_uv.len();
                    obj_texture_uv.push(Vec3::zeros());

                    for (_, indexed_uv, _) in group_mesh_triangles.iter_mut() {

                        if indexed_uv.is_none() {
                            *indexed_uv = Some([
                                new_dummy_texture_indx,
                                new_dummy_texture_indx,
                                new_dummy_texture_indx,
                            ]);
                        }
                    }


                }

                // TODO: mudar struct Texture para Material
                {
                    
                    // determinar texture_idx 
                    for (text_idx, texture) in textures.iter().enumerate() {
                        if texture.name == material_name {
                            texture_idx_match = Some(text_idx);
                            break;
                        }
                    }
                }

                if mesh_missing_normals {
                    //println!("{:#?}", group_mesh_triangles);

                    // calc normals
                    let mut gen_normals: Vec<Vec3> = vec![Vec3::zeros(); obj_vertices.len()];
                    for (indexed_tri, _, _) in group_mesh_triangles.iter() {
                        let normal = indexed_tri.calc_normal(&obj_vertices);

                        let a_idx = indexed_tri[0];
                        let b_idx = indexed_tri[1];
                        let c_idx = indexed_tri[2];

                        gen_normals[a_idx] = gen_normals[a_idx] + normal;
                        gen_normals[b_idx] = gen_normals[b_idx] + normal;
                        gen_normals[c_idx] = gen_normals[c_idx] + normal;
                    }

                    /*
                    if gen_normals.contains(&Vec3::zeros()) { 
                        let mut sum = 0;
                        for norm in gen_normals.iter() {
                            if *norm == Vec3::zeros() {
                                sum += 1;
                            }
                        }

                        println!("foun {sum} of {}", gen_normals.len());
                        panic!("The normal vectors can't be calculated properly.");
                    }
                    */

                    for normal in gen_normals.iter_mut() {
                        *normal = normal.normalized();
                    }

                    // TODO: Como os indeces armazenados num IndexedTriangle
                    // sao utilizados para indexar os vectors globais dum Object, uma possivel
                    // solucao seria extender o vector global de vetores normais, obtidos na leitura
                    // do arquivo, utilizando o vector de normais calculados.
                    //
                    // TODO: Criar teste onde eh realizada a leitura de um modelo que possui
                    // vetores normais definidos e faces que nao referenciam os vetores
                    // normais.

                    // build indexed triangles for the normals
                    // (PT): substitui os triangulos de normais faltantes.
                    // poercorre o group_mesh_triangles procurando por faces onde 
                    // faltam normais. 
                    for (indexed_tri, _, indexed_normal) in group_mesh_triangles.iter_mut() {

                        if indexed_normal.is_none() {
                            *indexed_normal = Some([
                                indexed_tri[0] + obj_normals.len(),
                                indexed_tri[1] + obj_normals.len(),
                                indexed_tri[2] + obj_normals.len(),
                            ]);
                        }
                    }

                    obj_normals.extend(gen_normals);

                }

                let mesh_triangles = group_mesh_triangles.iter()
                                                        .map(|(vert, text, norm)| (*vert, text.unwrap(), norm.unwrap()))
                                                        .collect::<_>();

                meshes.push(

                    IndexedMesh {
                        triangles:  mesh_triangles,
                      //triangles:              mesh_faces,
                      //normals_triangles:      mesh_normals_faces,
                      //texture_triangles:      mesh_texture_faces,
                        texture_idx: texture_idx_match,
                    }
                );

            }
        }


        Self::new(
          obj_vertices,
          obj_normals,
          Some(obj_texture_uv),
          meshes,
          //Some(materials),
          textures,             // Option<Vec<TextureMap>>
        )

    }

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

        let horizontal_view_angle: f32 = 135.0;  // degrees
        let h_angle_rad = horizontal_view_angle.to_radians();

        let right = dbg!(n.abs() * (h_angle_rad / 2.0).tan());
        let left = - right;

        let top = ratio.recip() * right;
        let bottom = - top;
        let further = n - 500.;

        assert!(horizontal_view_angle < 180.0);
        assert!(n < 0.0);
        assert!(n > further);
        assert!(right > left);
        assert!(top > bottom);

        println!("CameraWindow dimension ({} x {})", right-left, top-bottom);

        // nearest face of the view volume

        Self {
            position:    pos,
            direction:   dir.normalized(),
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
    fn set_screen_direction(&mut self, dx: f32, dy: f32) {
        self.direction = self.direction 
            + (self.u.normalized() * dx + self.v.normalized() * dy);
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
    fn clip (&self, tri: &Triangle, tri_pool_ret: &mut [Triangle]) -> usize {
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
            //return Vec::from([tri.clone()]);
            tri_pool_ret[0] = tri.clone();
            return 1;
        } else
        if f_a <= 0.0 && 
           f_b <= 0.0 && 
           f_c <= 0.0 
        {
            //return Vec::new();
            return 0;
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


        if tri_pool_ret.is_empty() {
            println!("eita minino");
        }


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
                //tri.color,
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
                //tri.color,
            );

            //return Vec::from([new_triangle_a, new_triangle_b]);
            tri_pool_ret[0] = new_triangle_a;
            tri_pool_ret[1] = new_triangle_b;
            return 2;

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
                //tri.color,
            );

            tri_pool_ret[0] = new_triangle_c;
            return 1;

        }

    }
}

pub
struct Scene {
    canvas: Canvas,
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
        //let camera_pos = Vec3::new([0., 300., 630.0]); // Aviao ae
        let camera_pos = Vec3::new([0., 100., 140.0]);
        let camera_dir = Vec3::new([0., -0.3, -1.0]);

        let light_source = Vec3::new([0., 300., 300.]);

        let ratio = (width as f32) / (height as f32);
        let camera = Camera::new(camera_pos, camera_dir, ratio);


        let mut canvas = Canvas::new(width, height);
        canvas.init_depth(100000.0);

        // alguns modelos classicos
        // https://casual-effects.com/data/

        //let obj = Object::inv_piramid(Vec3::zeros());
        //let obj = Object::load_from_file("Glass Bowl with Cloth Towel.obj");
        //let obj = Object::load_from_file("models/donut/donut.obj");
        //let obj = Object::load_from_file("models/soldier1/soldier1.obj");
        //let obj = Object::load_from_file("models/g_soldier1/soldier1.obj");
        //let obj = Object::load_from_file("models/lemur/lemur.obj");
        //let obj = Object::load_from_file("models/airplane/11804_Airplane_v2_l2.obj");
        let obj = Object::load_from_file("models/CornellBox/CornellBox-Original.obj");

        //let obj_vec = Object::load_from_directory("models/dungeon_set/");

        Self {
            canvas:   canvas,
            width:   width,
            height:  height,
            camera:  camera,
            //objects: obj_vec,
            objects: vec![obj],

            light_source: light_source,
        }
    }

    pub
    fn move_camera_directon(&mut self, dx: f32, dy: f32) {
        assert!(dx < self.width  as f32);
        assert!(dy < self.height as f32);

        println!("{} {}", dx, dy);
        self.camera.set_screen_direction(dx, dy);
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

        self.light_source = new_light.as_vec3();
    }

    pub
    fn camera_right(&mut self) {
        let theta: f32 = 4.0;
        self.camera.rotate_origin(theta);

        //let cam_pos = self.camera.get_pos();
        //self.camera.set_pos(cam_pos + Vec3::new([-0.05, 0., 0.]));

        let new_light = Matrix4::rotate_y(theta.to_radians()) * 
                        self.light_source.as_vec4();

        self.light_source = new_light.as_vec3();
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

        self.canvas.clear();

        let matrix_transf = self.gen_transformation_matrix();

        let camera_pos = self.camera.get_pos();

        let func_planes = self.camera.gen_view_planes();

        for obj in self.objects.iter_mut() {
            let obj_vertices    = &obj.vertices;
            let obj_normals     = &obj.normals_vertices;
            let obj_texture_uv  = obj.texture_vertices.as_ref().unwrap();

            // Calcular VertexAttributes aq para remover cálculos redundantes
        
            for (vertex, visual_info) in obj.vertices.iter()
                .zip(obj.vertices_visual_info.iter_mut()) {

                    let eye_dir = *vertex - camera_pos;

                    visual_info.light   = (*vertex - self.light_source).normalized();
                    visual_info.eye     = eye_dir.normalized();
                    visual_info.depth   = eye_dir.norm() as _;
                    visual_info.halfway = (visual_info.light + visual_info.eye).normalized();

            }


            self.canvas.enable_depth_update();
            for obj_mesh in obj.opaque_meshes.iter() {
                let mesh_texture_idx = obj_mesh.texture_idx.unwrap();
                // TODO: ta meio feio isso aq, tem que embelezar.
                // criar um iterador no futuro tlvz
                for (vertex_tri_idx, texture_tri_idx, normal_tri_idx) in obj_mesh.triangles.iter() {
                    let indexed_tri_vertex  = vertex_tri_idx.clone();
                    let indexed_tri_normal  = normal_tri_idx.clone();
                    let indexed_tri_texture = texture_tri_idx.clone();



                    let tri_vertices = IndexedMesh::vec3_list_from_indexed(
                        indexed_tri_vertex,
                        obj_vertices,
                    );
                    let tri_normals  = IndexedMesh::vec3_list_from_indexed(
                        indexed_tri_normal,
                        obj_normals,
                    );
                    let tri_textures = IndexedMesh::vec3_list_from_indexed(
                        indexed_tri_texture,
                        obj_texture_uv,
                    );

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
                        //Color::Green,
                    );

                    let tri_normal = Triangle::calc_normal(&original_tri);
                    let tri_eye = camera_pos - original_tri.get_center();

                    // TODO: calculo da normal potencialmente errado
                    if tri_eye.dot(tri_normal) <= 0.0 {
                        continue;
                    }

                    let mut clipped_triangles: [Triangle; 12]  = [Triangle::zeroed(); 12]; 
                    let clipped_count = original_tri.clip_against_planes(&func_planes, clipped_triangles.as_mut_slice());
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

                        let a_vec4  = matrix_transf * clipped_tri.vertices[0].as_vec4();
                        let b_vec4  = matrix_transf * clipped_tri.vertices[1].as_vec4();
                        let c_vec4  = matrix_transf * clipped_tri.vertices[2].as_vec4();

                        let a_w = a_vec4.get_w();
                        let b_w = b_vec4.get_w();
                        let c_w = c_vec4.get_w();

                        let a_coord  = a_vec4.as_vec2() / a_w;
                        let b_coord  = b_vec4.as_vec2() / b_w;
                        let c_coord  = c_vec4.as_vec2() / c_w;

                        let mut clip_tri_vert_attr = &mut clipped_tri.vertices_attr;
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
                            Some(texture) => &texture,
                            None          => &Texture::default(),
                        };

                        self.canvas.draw_triangle_with_attributes(
                            &clip_tri_vert_attr[0],
                            &clip_tri_vert_attr[1],
                            &clip_tri_vert_attr[2],

                            mesh_texture
                        );
                    }
                }
            }

            self.canvas.disable_depth_update();


            for obj_mesh in obj.transparent_meshes.iter_mut() {
                let mesh_texture_idx = obj_mesh.texture_idx.unwrap();


                // sort aq (painter algorithm)
                // TODO: essa ordenação precisa ser aplicada a todos os triangulos
                // de todos os objetos transparentes de maneira absoluta.
                // Talvez seja possivel apenas manter essa ordenação atual, relativa
                // a cada mesh, e complementar com a ordenação prévia das meshes 
                // transparentes pela sua posição.

                obj_mesh.triangles.as_mut_slice().sort_by(|a, b| {
                    let (a_vert_tri, _, _) = a;
                    let (b_vert_tri, _, _) = b;

                    let a_verts = IndexedMesh::vec3_list_from_indexed(
                        *a_vert_tri,
                        obj_vertices,
                    );

                    let b_verts = IndexedMesh::vec3_list_from_indexed(
                        *b_vert_tri,
                        obj_vertices,
                    );

                    let a_center = (a_verts[0] + a_verts[1] + a_verts[2]) / 3.0;
                    let b_center = (b_verts[0] + b_verts[1] + b_verts[2]) / 3.0;

                    let a_depth = a_center.dist(camera_pos);
                    let b_depth = b_center.dist(camera_pos);

                    // ordem não decrescente
                    a_depth.total_cmp(&b_depth).reverse()
                    /*
                    if a_depth < b_depth {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
                    */
                });
                
                for (vertex_tri_idx, texture_tri_idx, normal_tri_idx) in obj_mesh.triangles.iter() {
                    let indexed_tri_vertex  = vertex_tri_idx.clone();
                    let indexed_tri_normal  = normal_tri_idx.clone();
                    let indexed_tri_texture = texture_tri_idx.clone();



                    let tri_vertices = IndexedMesh::vec3_list_from_indexed(
                        indexed_tri_vertex,
                        obj_vertices,
                    );
                    let tri_normals  = IndexedMesh::vec3_list_from_indexed(
                        indexed_tri_normal,
                        obj_normals,
                    );
                    let tri_textures = IndexedMesh::vec3_list_from_indexed(
                        indexed_tri_texture,
                        obj_texture_uv,
                    );

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
                        //Color::Green,
                    );

                    let mut clipped_triangles: [Triangle; 12]  = [Triangle::zeroed(); 12]; 
                    let clipped_count = original_tri.clip_against_planes(&func_planes, clipped_triangles.as_mut_slice());
                    for clipped_tri in clipped_triangles[..clipped_count].iter_mut() {

                        let a_vec4  = matrix_transf * clipped_tri.vertices[0].as_vec4();
                        let b_vec4  = matrix_transf * clipped_tri.vertices[1].as_vec4();
                        let c_vec4  = matrix_transf * clipped_tri.vertices[2].as_vec4();

                        let a_w = a_vec4.get_w();
                        let b_w = b_vec4.get_w();
                        let c_w = c_vec4.get_w();

                        let a_coord  = a_vec4.as_vec2() / a_w;
                        let b_coord  = b_vec4.as_vec2() / b_w;
                        let c_coord  = c_vec4.as_vec2() / c_w;

                        let mut clip_tri_vert_attr = &mut clipped_tri.vertices_attr;
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
                            Some(texture) => &texture,
                            None          => &Texture::default(),
                        };

                        self.canvas.draw_triangle_with_attributes(
                            &clip_tri_vert_attr[0],
                            &clip_tri_vert_attr[1],
                            &clip_tri_vert_attr[2],

                            mesh_texture
                        );
                    }
                }
            }
        }
    }

    pub
    fn draw_indexed_mesh (&mut self, mesh: &IndexedMesh) {
    }

    pub
    fn frame_as_bytes_slice(&self) -> &[u8] {
        self.canvas.as_bytes_slice()
    }

}
