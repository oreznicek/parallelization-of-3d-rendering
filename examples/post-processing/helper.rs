
fn abs(x: f32) -> f32 {
    if x < 0.0 {
        return x*(-1.0);
    }
    x
}

// This position equals [0, 0] in UV coordinates
const origin: [f32; 2] = [-1.0, 1.0];

// Calculate UV coordinates from position on the screen
pub fn getUVfromPosition(pos: [f32; 2]) -> [f32; 2] {
    let mut uv = [0.0; 2];

    uv[0] = abs(origin[0] - pos[0]) / 2.0;
    uv[1] = abs(origin[1] - pos[1]) / 2.0;

    uv
} 