mod tint;
mod contour;

use std::vec::Vec;
use bytemuck::{Pod, Zeroable};
use tint::Tint;
use contour::Contour;
use bitflags::bitflags;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct UVVertex {
    pub pos: [f32; 2],
    pub uv_coords: [f32; 2]
}

bitflags! {
    pub struct AllowedEffects: u32 {
        const TINT = 1 << 0;
        const CONTOUR = 1 << 1;
    }
}

fn nearest_power_of_two(n: u32) -> (u32, u32) {
    let mut bit = 0;
    let mut power_of_two = 2;

    if n <= power_of_two {
        return (power_of_two, n);
    }

    while n > power_of_two {
        power_of_two *= 2;
        bit += 1;
    }

    (power_of_two, bit)
}

impl AllowedEffects {
    // Based on AllowedEffects count we will generate output textures for each member in post processing chain
    // textures_to_generate_count = AllowedEffects::count() - 1;
    // the last chain member will output the result into given frame buffer
    pub fn count(&self) -> u32 {
        let num = self.bits;
        let (power_of_two, b) = nearest_power_of_two(num);
        println!("{} {}", power_of_two, b);
        let mut bit: i32 = b as i32;
        let mut count = 0;
        let mut temp = 0;

        while bit >= 0 {
            temp = num & (1 << bit);
            if temp > 0 {
                count += 1;
            }
            bit -= 1;
        }

        count
    }

    pub fn highest_bit(&self) -> u32 {
        let (power_of_two, bit) = nearest_power_of_two(self.bits);
        power_of_two
    }
}

pub fn create_output_texture_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
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

pub struct PostProcessing {
    flags: AllowedEffects,
    texture_views: Vec<wgpu::TextureView>,
	tint: Option<Tint>,
	contour: Option<Contour>,
}

impl PostProcessing {
	pub fn init(
		flags: AllowedEffects,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
		input_frame: &wgpu::TextureView,
	) -> PostProcessing {
		let effects_count = flags.count();
		let tint;
        let contour; 
        let mut texture_views = Vec::new();
        let mut texture_index = 0;

        if effects_count == 0 {
            return PostProcessing { flags, texture_views, tint: None, contour: None };
        }

        for i in 0..effects_count-1 {
            texture_views.push(create_output_texture_view(device, config));
        }
        //texture_views.push(final_frame);

        // Tint
		if !(flags & AllowedEffects::TINT).is_empty() {
			tint = Some(Tint::init(device, input_frame, [1.0, 0.0, 0.0, 1.0]));
            texture_index += 1;
		}
		else {
			tint = None;
		}


        // Contour
		if !(flags & AllowedEffects::CONTOUR).is_empty() {
            if texture_index == 0 {
                contour = Some(Contour::init(device, input_frame, config)) 
            }
            else {
                contour = Some(Contour::init(device, &texture_views[texture_index - 1], config)) 
            }
            texture_index += 1;
        }
        else {
            contour = None;
        }

        PostProcessing {
            flags,
            texture_views,
            tint,
            contour,
        }
	}

    pub fn resolve(&self, device: &wgpu::Device, queue: &wgpu::Queue, final_frame: &wgpu::TextureView) {
        let mut texture_index = 0;

        if let Some(tint) = &self.tint {
            if self.flags.highest_bit() == AllowedEffects::TINT.bits {
                tint.resolve(device, queue, final_frame);
            }
            else {
                tint.resolve(device, queue, &self.texture_views[texture_index]);
                texture_index += 1;
            }
        }

        if let Some(contour) = &self.contour {
            if self.flags.highest_bit() == AllowedEffects::CONTOUR.bits {
                contour.resolve(device, queue, final_frame);
            }
            else {
                contour.resolve(device, queue, &self.texture_views[texture_index]);
                texture_index += 1;
            }
        }
    }
}


