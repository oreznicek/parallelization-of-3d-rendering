use crate::shapes::{Vertex, PI};
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
            _color: crate::create_color([1, 41, 95, 255]),
        };
        vertices.push(new_vertex);
        //println!("{:?}", item);
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

// Own generator
pub fn map(value: f32, range1: [f32; 2], range2: [f32; 2]) -> f32 {
    // Check values passed to the function
    if range1[1] <= range1[0] || range2[1] <= range2[0] {
        println!("Wrong values were passed in the function.");
        return 0.0;
    }
    else if value > range1[1] || value < range1[0] {
        println!("Value passed to the function is not inside the given range.");
        return 0.0;
    }

    // Calculate how big are those ranges
    let size1 = range1[1] - range1[0];
    let size2 = range2[1] - range2[0];

    // How far is value from the start of its range
    let distance = value - range1[0];

    // Calculate percent and the resulting value
    let percentage = distance / size1;
    let result_value = (size2 * percentage) + range2[0];

    result_value
}

// Get position of two dimensional value in one dimensional array
fn pos_in_one_dim_vec(x: u16, y: u16, columns: u16) -> u16 {
    y*columns+x
}

pub fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let r: f32 = 1.0;
    let resolution: u16 = 20;

    let mut vertices: Vec<Vertex> = Vec::<Vertex>::new();

    for i in 0..resolution {
        let lon = map(i as f32, [0.0, resolution as f32], [-PI, PI]);
        for j in 0..resolution {
            let lat = map(j as f32, [0.0, resolution as f32], [-PI/2.0, PI/2.0]);
            let x = r * lon.sin() * lat.cos();
            let y = r * lon.sin() * lat.sin();
            let z = r * lon.cos();

            vertices.push(Vertex {
                _pos: [x, y, z, 1.0],
                _color: crate::create_color([255, 255, 255, 255]),
            });
        }
    }

    let mut indexes: Vec<u16> = Vec::<u16>::new();

    for y in 0..resolution {
        for x in 0..resolution {
            //let position: [f32; 4] = vertices[(y*resolution+x) as usize]._pos;
            // first triangle
            indexes.push(pos_in_one_dim_vec(x, y+1, resolution));
            indexes.push(pos_in_one_dim_vec(x+1, y+1, resolution));
            indexes.push(pos_in_one_dim_vec(x+1, y, resolution));

            // second triangle
            indexes.push(pos_in_one_dim_vec(x+1, y, resolution));
            indexes.push(pos_in_one_dim_vec(x, y, resolution));
            indexes.push(pos_in_one_dim_vec(x, y+1, resolution));
        }
    }
    //println!("{:?}", indexes);

    (vertices, indexes)
}