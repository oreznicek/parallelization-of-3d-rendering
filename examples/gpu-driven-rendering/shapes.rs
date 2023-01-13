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

pub const TEXTURE_TYPE_VARIANTS: usize = 3;

#[derive(Clone, Copy, Debug)]
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
    pub m_type: MeshType,
    pub texture: TextureType,
    pub transform_m: Vec<glam::Mat4>
}

#[repr(C)]
pub struct Object {
    pub transform_m: glam::Mat4,
    pub texture_type: TextureType,
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

pub fn get_objects_from_batches(batches: &Vec<&Batch>) -> Vec<Object> {
    let mut objects = Vec::new();
    for b in batches {
        for m in &b.transform_m {
            objects.push(
                Object {
                    texture_type: b.texture,
                    transform_m: *m
                }
            );
        }
    }
    objects
}

pub fn merge_matrices(objects: &Vec<Object>) -> Vec<f32> {
    let mut matrices: Vec<f32> = Vec::<f32>::new();
    for o in objects {
        matrices.extend(
            &o.transform_m.to_cols_array_2d().concat()
        );
    }
    matrices
}