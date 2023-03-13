mod sphere;
mod cube;
mod cylinder;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeshType {
    Cube,
    Cylinder,
    Sphere
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureType {
    Blue,
    Red,
    Yellow
}

pub struct Mesh {
    pub m_type: MeshType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>
}

impl Mesh {
    pub fn generate_vertices(&mut self) {
        let result;
        match self.m_type {
            MeshType::Cube => result = cube::create_vertices(),
            MeshType::Cylinder => result = cylinder::generate_vertices(),
            MeshType::Sphere => result = sphere::generate_vertices()
        }
        self.vertices = result.0;
        self.indices = result.1;
    }
}

// Batch is a pair of mesh and texture.
// The number of objects we want to draw from this batch
// is defined with number of transform matrices in transform_m vector.
#[repr(C)]
#[derive(Debug)]
pub struct Batch {
    pub transform_m: Vec<glam::Mat4>,
    pub m_type: MeshType,
    pub t_type: TextureType,
}

// Represents an object from the scene
#[repr(C)]
pub struct Object {
    pub transform_m: glam::Mat4,
    pub m_type: MeshType,
    pub t_type: TextureType,
}

fn vertex(pos: [i8; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: tc,
    }
}

pub fn merge_index_vertex_data(meshes: &Vec<&Mesh>) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::<Vertex>::new();
    let mut indices: Vec<u16> = Vec::<u16>::new();

    // Vertex count of meshes before
    // We need to add this number to indices of the next mesh
    let mut vertex_count = 0;

    for m in meshes {
        vertices.extend(&m.vertices);

        for i in 0..m.indices.len() {
            indices.push(m.indices[i] + vertex_count as u16);
        }
        vertex_count += m.vertices.len();
    }

    (vertices, indices)
}

pub fn get_batches_from_objects(objects: &Vec<Object>) -> Vec<Batch> {
    let mut batches = Vec::<Batch>::new();

    for o in objects {
        let batch = batches.iter_mut().find(|b| b.m_type == o.m_type && b.t_type == o.t_type);

        match batch {
            Some(x) => x.transform_m.push(o.transform_m),
            None => batches.push(
                Batch {
                    transform_m: vec![o.transform_m],
                    m_type: o.m_type,
                    t_type: o.t_type,
                } 
            )
        }
    }

    batches    
}

pub fn merge_matrices(batches: &Vec<Batch>) -> Vec<f32> {
    let mut matrices = Vec::<f32>::new();

    for b in batches {
        for m in &b.transform_m {
            matrices.extend(m.to_cols_array());
        }
    }

    matrices
}

pub fn merge_objects(batches: &Vec<Batch>) -> Vec<u32> {
    let mut objects = Vec::<u32>::new();
    let mut transform_id = 0;

    for b in batches {
        for _m in 0..b.transform_m.len() {
            objects.push(transform_id);
            transform_id += 1;

            let texture_id: u32 = match b.t_type {
                TextureType::Blue => 0,
                TextureType::Red => 1,
                TextureType::Yellow => 2,
            };
            objects.push(texture_id); // Add objects texture_id
        }
    }

    objects
}