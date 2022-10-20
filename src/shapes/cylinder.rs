use crate::shapes::{Vertex};
use std::vec::Vec;
use genmesh::{Position, Polygon};
use genmesh::generators::{Cylinder, IndexedPolygon, SharedVertex};

pub fn generate_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let resolution: usize = 20;
    let cylinder = Cylinder::new(resolution);

    let mut vertices: Vec<Vertex> = Vec::<Vertex>::new();

    for vertex in cylinder.shared_vertex_iter() {
        let v1: Position = vertex.pos;
        let new_vertex = Vertex {
            _pos: [v1.x, v1.y, v1.z, 1.0],
            _color: crate::create_color([1, 41, 95, 255]),
        };
        vertices.push(new_vertex);
    }

    let mut indices: Vec<u16> = Vec::<u16>::new();

    for polygon in cylinder.indexed_polygon_iter() {
        match polygon {
            Polygon::PolyTri(triangle) => {
                indices.push(triangle.x as u16);
                indices.push(triangle.y as u16);
                indices.push(triangle.z as u16);
            },
            Polygon::PolyQuad(quad) => {
                indices.push(quad.x as u16);
                indices.push(quad.y as u16);
                indices.push(quad.z as u16);

                indices.push(quad.z as u16);
                indices.push(quad.w as u16);
                indices.push(quad.x as u16);
            },
        }
    }

    (vertices, indices)
}