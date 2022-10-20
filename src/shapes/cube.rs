use crate::shapes::{Vertex, vertex};

// Create cube vertices
pub fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // front
        vertex([-1,  1, -1], [255, 0, 0, 255]),
        vertex([ 1,  1, -1], [255, 0, 0, 255]),
        vertex([-1, -1, -1], [255, 0, 0, 255]),
        vertex([ 1, -1, -1], [255, 0, 0, 255]),
        // back
        vertex([-1,  1,  1], [0, 0, 255, 255]),
        vertex([ 1,  1,  1], [0, 0, 255, 255]),
        vertex([-1, -1,  1], [0, 0, 255, 255]),
        vertex([ 1, -1,  1], [0, 0, 255, 255]),
    ];

    //2, 3, 1, 1, 0, 2, // front
    //6, 4, 5, 5, 7, 6, // back
    //0, 1, 5, 5, 4, 0, // top
    //2, 6, 7, 7, 3, 2, // bottom
    //3, 7, 5, 5, 1, 3, // right
    //2, 0, 4, 4, 6, 2, // left
    let index_data: &[u16] = &[
        2, 0, 1, 1, 3, 2, // bottom
        6, 7, 5, 5, 4, 6, // top
        0, 4, 5, 5, 1, 0, // back
        2, 3, 7, 7, 6, 2, // front
        3, 1, 5, 5, 7, 3, // right
        2, 6, 4, 4, 0, 2, // left
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}