use crate::shapes::Vertex;
use std::vec::Vec;
use genmesh::{Position, Polygon};
use genmesh::generators::{SphereUv, SharedVertex, IndexedPolygon};

pub fn generate_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let resolution: usize = 20;
    let sphere = SphereUv::new(resolution, resolution);

    let mut vertices: Vec<Vertex> = Vec::<Vertex>::new();

    for item in sphere.shared_vertex_iter() {
        let v1: Position = item.pos;
        let new_vertex = Vertex {
            _pos: [v1.x, v1.y, v1.z, 1.0],
            _tex_coord: [0.5, 0.5],
        };
        vertices.push(new_vertex);
    }

    let mut indexes: Vec<u16> = Vec::<u16>::new();

    for item in sphere.indexed_polygon_iter() {
        match item {
            Polygon::PolyTri(triangle) => {
                indexes.push(triangle.x as u16);
                indexes.push(triangle.y as u16);
                indexes.push(triangle.z as u16);
            },
            Polygon::PolyQuad(quad) => {
                indexes.push(quad.x as u16);
                indexes.push(quad.y as u16);
                indexes.push(quad.z as u16);

                indexes.push(quad.z as u16);
                indexes.push(quad.w as u16);
                indexes.push(quad.x as u16);
            },
        }
    }

    (vertices, indexes)
}