use crate::renderer::linalg::Vec3;

pub struct MeshInfo {
    pub name: String,
    pub triangle_count: u32,
    pub texture_name: Option<String>,
}

pub trait IndexedTriangleNormal {
    fn calc_normal(&self, vertices: &Vec<Vec3>) -> Vec3;
}

pub type IndexedTriangle = [usize; 3];

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
pub struct IndexedMesh {
    pub name: String,
    pub triangles: Vec<(IndexedTriangle, IndexedTriangle, IndexedTriangle)>,
    pub texture_idx: Option<usize>,
}

impl IndexedMesh {
    pub fn vec3_list_from_indexed(
        indexed_tri: IndexedTriangle,
        vert_list: &Vec<Vec3>,
    ) -> [Vec3; 3] {
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

            [a_vert, b_vert, c_vert]
        }
    }
}
