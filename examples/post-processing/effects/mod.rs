
// This effect will change the tone of the whole scene
// based on the input color
struct Tint {
    pipeline: wgpu::RenderPipeline,
}

impl Tint {
    // New will create pipeline for this specific post-processing effect
    // With all the buffer bindings needed
    fn new(device: &wgpu::Device, tint_color: [f32; 4]) -> Self {
        let 
    }

    // Resolve frame will take the output frame of previous rendering as an input
    // It will apply the effect to the scene
    fn resolve_frame() {
        
    }
}