use crate::renderer::linalg::Vec3;
use crate::renderer::scene::mesh::IndexedMesh;
use crate::renderer::scene::mesh::IndexedTriangle;
use crate::renderer::scene::mesh::IndexedTriangleNormal;
use crate::renderer::scene::mesh::MeshInfo;
use crate::renderer::scene::Texture;
use crate::renderer::scene::TextureMap;
use crate::renderer::scene::VertexVisual;

use obj;

pub struct ObjectInfo {
    pub id: u32,
    pub name: String,
    pub mesh_info_list: Vec<MeshInfo>,
}

pub struct Object {
    pub id: Option<u32>,
    pub name: String,
    pub vertices: Vec<Vec3>,
    pub normals_vertices: Vec<Vec3>,
    pub texture_vertices: Option<Vec<Vec3>>,

    pub vertices_visual_info: Vec<VertexVisual>,

    pub opaque_meshes: Vec<IndexedMesh>,
    pub transparent_meshes: Vec<IndexedMesh>,

    pub textures: Vec<Texture>,
}

impl Object {
    pub fn new(
        name: String,
        vertices: Vec<Vec3>,
        normals_vertices: Vec<Vec3>,
        texture_vertices: Option<Vec<Vec3>>,
        meshes: Vec<IndexedMesh>,
        textures: Vec<Texture>,
    ) -> Self {
        let mut opaque: Vec<IndexedMesh> = Vec::new();
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
            id: None,
            name,
            vertices,
            normals_vertices,
            texture_vertices,

            vertices_visual_info: vec![VertexVisual::zeroed(); vert_total],

            opaque_meshes: opaque,
            transparent_meshes: transparent,

            textures,
        }
    }

    pub fn load_from_directory(dir: &str) -> Vec<Self> {
        let file_ext = "obj";
        let path = std::path::Path::new(dir);

        if !path.is_dir() {
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
    pub fn load_from_file(filename: &str) -> Self {
        use std::fs::File;
        use std::io::BufReader;
        use std::path::{Path, PathBuf};

        let path = Path::new(filename);
        let parent_dir = path.parent();

        let add_file_path = |filename: &String| -> PathBuf {
            if let Some(dir_path) = parent_dir {
                [
                    dir_path.to_str().expect("Failed adding file path."),
                    filename.as_str(),
                ]
                .iter()
                .collect()
            } else {
                PathBuf::from(filename.as_str())
            }
        };

        let file = File::open(path);
        assert!(file.is_ok(), "Unable to open file {}", filename);
        let reader = BufReader::new(file.unwrap());

        let mut obj_data = obj::ObjData::load_buf(reader).unwrap();

        obj_data.material_libs.iter_mut().for_each(|mtllib| {
            let mtl_path: PathBuf = add_file_path(&mtllib.filename);
            let fname = mtllib.filename.as_str();
            let file = File::open(mtl_path);
            assert!(file.is_ok(), "Unable to open file {}", fname);
            _ = mtllib.reload(file.unwrap());
        });

        let mut obj_vertices: Vec<Vec3> = obj_data
            .position
            .iter()
            .map(|e| Vec3::new([e[0], e[1], e[2]]))
            .collect::<_>();
        let mut obj_normals: Vec<Vec3> = obj_data
            .normal
            .iter()
            .map(|e| Vec3::new([e[0], e[1], e[2]]).normalized())
            .collect::<_>();
        let mut obj_texture_uv: Vec<Vec3> = obj_data
            .texture
            .iter()
            .map(|e| Vec3::new([e[0], e[1], 0.0]))
            .collect::<_>();

        // TODO: keep this ??
        // rescaling test
        if true {
            let mut vertices_sorted = obj_vertices.iter().map(|e| e.norm()).collect::<Vec<_>>();
            vertices_sorted
                .as_mut_slice()
                .sort_by(|a, b| a.partial_cmp(b).unwrap().reverse());
            let vertex_max = vertices_sorted.first().unwrap();
            dbg!(vertex_max);

            let scale = 100.0;
            let factor = scale / vertex_max;
            obj_vertices.iter_mut().for_each(|e| *e = *e * factor);
        }

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
                let alpha = material.d.as_ref().unwrap_or(&1.0);

                let map_ka = if let Some(map_ka_filename) = material.map_ka.as_ref() {
                    println!("{}", map_ka_filename);
                    let f_path = add_file_path(map_ka_filename);
                    TextureMap::load_from_file(&f_path)
                } else {
                    TextureMap::default()
                };

                let map_kd = if let Some(map_kd_filename) = material.map_kd.as_ref() {
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

                textures.push(Texture {
                    name,

                    ka: Vec3::new(*ka),
                    kd: Vec3::new(*kd),
                    ks: Vec3::new(*ks),

                    alpha: *alpha,

                    map_ka,
                    map_kd,
                });
            }
        }

        // FIXME: we need to support loading separately more
        // than one object existing in the same .obj file.
        // Currently we give to the Object the name of the file.
        if obj_data.objects.len() > 1 {
            println!("WARNING: more than one Object exists in this file.");
        }

        for obj in obj_data.objects.iter() {
            println!("Object {}", obj.name);

            for group in obj.groups.iter() {
                // Group doesnt have faces
                if group.polys.is_empty() {
                    continue;
                }

                let mut group_mesh_triangles: Vec<(
                    IndexedTriangle,
                    Option<IndexedTriangle>,
                    Option<IndexedTriangle>,
                )> = Vec::new();

                println!("\t Group name     {}", group.name);
                println!("\t Group material {:?}", group.material);

                let material_name = if let Some(material) = &group.material {
                    match material {
                        obj::ObjMaterial::Ref(material_name) => material_name.clone(),

                        obj::ObjMaterial::Mtl(material_arc) => material_arc.name.clone(),
                    }
                } else {
                    String::from("default")
                };

                let mut mesh_missing_texture = false;
                let mut mesh_missing_normals = false;

                for face in group.polys.iter() {
                    let face_vec = &face.0;
                    let mut vertex_index: Vec<usize> = Vec::new();
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
                            [vertex_idx_a, vertex_idx_b, vertex_idx_c],
                            // texture vertices
                            if face_missing_texture == false {
                                Some([
                                    texture_idx_a.unwrap(),
                                    texture_idx_b.unwrap(),
                                    texture_idx_c.unwrap(),
                                ])
                            } else {
                                None
                            },
                            // normal vertices
                            if face_missing_normals == false {
                                Some([
                                    normals_idx_a.unwrap(),
                                    normals_idx_b.unwrap(),
                                    normals_idx_c.unwrap(),
                                ])
                            } else {
                                None
                            },
                        ));
                    }

                    if face_vec.len() == 4 {
                        let vertex_idx_d = vertex_index[3];
                        let texture_idx_d = texture_index[3];
                        let normals_idx_d = normals_index[3];

                        group_mesh_triangles.push((
                            [vertex_idx_c, vertex_idx_d, vertex_idx_a],
                            // texture vertices
                            if face_missing_texture == false {
                                Some([
                                    texture_idx_c.unwrap(),
                                    texture_idx_d.unwrap(),
                                    texture_idx_a.unwrap(),
                                ])
                            } else {
                                None
                            },
                            // normal vertices
                            if face_missing_normals == false {
                                Some([
                                    normals_idx_c.unwrap(),
                                    normals_idx_d.unwrap(),
                                    normals_idx_a.unwrap(),
                                ])
                            } else {
                                None
                            },
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

                // determinar texture_idx
                for (text_idx, texture) in textures.iter().enumerate() {
                    if texture.name == material_name {
                        texture_idx_match = Some(text_idx);
                        break;
                    }
                }

                if mesh_missing_normals {
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

                    for normal in gen_normals.iter_mut() {
                        *normal = normal.normalized();
                    }

                    // TODO: Criar teste onde eh realizada a leitura de um modelo que possui
                    // vetores normais definidos e faces que nao referenciam os vetores
                    // normais.

                    // build indexed triangles for the normals
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

                let mesh_triangles = group_mesh_triangles
                    .iter()
                    .map(|(vert, text, norm)| (*vert, text.unwrap(), norm.unwrap()))
                    .collect::<_>();

                meshes.push(IndexedMesh {
                    name: group.name.clone(),
                    triangles: mesh_triangles,
                    texture_idx: texture_idx_match,
                });
            }
        }

        let obj_name = path.file_name().unwrap().to_str().unwrap().to_string();

        Self::new(
            obj_name,
            obj_vertices,
            obj_normals,
            Some(obj_texture_uv),
            meshes,
            textures,
        )
    }

    pub fn mesh_info_list(&self) -> Vec<MeshInfo> {
        let mut ret: Vec<MeshInfo> = vec![];

        for mesh_list in [&self.opaque_meshes, &self.transparent_meshes] {
            for mesh in mesh_list {
                let texture_name = if mesh.texture_idx.is_some() {
                    let idx = mesh.texture_idx.unwrap();
                    Some(self.textures[idx].name.clone())
                } else {
                    None
                };

                ret.push(MeshInfo {
                    name: mesh.name.clone(),
                    triangle_count: mesh.triangles.len() as _,
                    texture_name,
                });
            }
        }
        ret
    }
}
