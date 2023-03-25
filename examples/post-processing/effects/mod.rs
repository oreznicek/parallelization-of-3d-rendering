mod tint;
mod contour;

use std::vec::Vec;
use bytemuck::{Pod, Zeroable};
use tint::Tint;
use contour::Contour;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct UVVertex {
    pub pos: [f32; 2],
    pub uv_coords: [f32; 2]
}

// Defines the type of the post-processing effect and encapsulates some additional parameters
#[derive(Clone, Copy, Debug)]
pub enum EffectType {
    Tint(f32, f32, f32, f32), // RGBA
    Contour
}

pub trait Effect {
    // Initializes the resources for the effect
    fn init(
        device: &wgpu::Device,
        input_view: &wgpu::TextureView,
        effect_type: EffectType,
    ) -> Self where Self: Sized;
    // Resolves the input frame and returns the result into output_view
    fn resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
    );
}

pub fn create_output_texture_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {

    let output_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: output_texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    });
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    output_view
}

// Creates specified number of texture views
pub fn create_output_texture_views(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    count: usize,
) -> Vec<wgpu::TextureView> {
    let mut output_views = Vec::new();

    for _i in 0..count {
        output_views.push(create_output_texture_view(device, config));
    }

    output_views
}

pub struct PostProcessing {
    effects: Vec<Box<dyn Effect>>, // Post-processing chain
    texture_views: Vec<wgpu::TextureView>, // Swap chain
}

impl PostProcessing {
	pub fn init(
		chain: &Vec<EffectType>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
		input_frame: &wgpu::TextureView,
	) -> PostProcessing {
		let effects_count = chain.len();
        let mut effects: Vec<Box<dyn Effect>> = Vec::new();
        let mut texture_views = Vec::new();

        if effects_count == 0 {
            return PostProcessing { effects, texture_views };
        }

        // 0 effects -> no need for output textures
        // 1 effect -> no need for output textures
        // 2 effects -> 1 texture
        // more effects -> swap chain (2 textures)
        match effects_count {
            2 => texture_views.push(create_output_texture_view(device, config)),
            _ => texture_views.extend(create_output_texture_views(device, config, 2))
        }

        let mut in_texture_id = -1;
        let mut in_texture: &wgpu::TextureView;

        for i in 0..chain.len() {
            // Limit swap chain to two buffers
            if in_texture_id > 1 {
                in_texture_id = 0;
            }

            if in_texture_id < 0 {
                in_texture = input_frame;
            }
            else {
                in_texture = &texture_views[in_texture_id as usize];
            }

            match chain[i] {
                EffectType::Tint(_, _, _, _) => {
                    effects.push(
                        Box::new(Tint::init(device, in_texture, chain[i]))
                    );
                },
                EffectType::Contour => {
                    effects.push(
                        Box::new(Contour::init(device, in_texture, chain[i]))
                    );
                }
            }

            in_texture_id += 1;
        }

        PostProcessing {
            effects,
            texture_views,
        }
	}

    pub fn resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_frame: &wgpu::TextureView, // frame buffer
    ) {
        let effects_count = self.effects.len();
        let mut out_texture_id = 0;

        for i in 0..effects_count {
            // Limit swap chain to two buffers
            if out_texture_id > 1 {
                out_texture_id = 0;
            }

            if i == effects_count-1 {
                (*self.effects[i]).resolve(device, queue, output_frame);
            }
            else {
                (*self.effects[i]).resolve(device, queue, &self.texture_views[out_texture_id]);
            }

            out_texture_id += 1;
        }
    }
}


