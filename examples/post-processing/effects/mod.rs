mod tint;
mod contour;

use tint::Tint;
use contour::Contour;

use std::vec::Vec;
use std::iter::IntoIterator;
use std::ops::Index;
use crate::helper::{create_output_texture_view, create_output_texture_views};

// Defines the type of the post-processing effect and encapsulates some additional parameters
#[derive(Clone, Copy, Debug)]
pub enum EffectType {
    Tint(f32, f32, f32, f32), // RGBA
    Contour
}

// Represents the post-processing effect
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

// Wrapper for Vec<EffectType>
// Specifies the order in which the effects will be executed
pub struct PostProcessingChain {
    inner: Vec<EffectType>
}

impl PostProcessingChain {
    pub fn new() -> Self {
        Self {
            inner: Vec::new()
        }
    }

    pub fn add_effect(&mut self, effect_type: EffectType) {
        self.inner.push(effect_type);
    }

    pub fn effects_count(&self) -> usize {
        self.inner.len()
    }
}

// Make the struct iterable
impl IntoIterator for PostProcessingChain {
    type Item = EffectType;
    type IntoIter = <Vec<EffectType> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

// Make the struct indexable
impl Index<usize> for PostProcessingChain {
    type Output = EffectType;

    fn index(&self, index: usize) -> &EffectType {
        &self.inner[index]
    }
}

// Provides access to post-processing effects and handles the swap chain mechanism
pub struct PostProcessing {
    effects: Vec<Box<dyn Effect>>, // Effect instances (in order of post-processing chain)
    texture_views: Vec<wgpu::TextureView>, // Swap chain
}

impl PostProcessing {
	pub fn init(
		chain: &PostProcessingChain,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
		input_frame: &wgpu::TextureView,
	) -> PostProcessing {
		let effects_count = chain.effects_count();
        let mut effects: Vec<Box<dyn Effect>> = Vec::new();
        let mut texture_views = Vec::new();

        if effects_count == 0 {
            return PostProcessing { effects, texture_views };
        }

        // zero effects -> no need for output textures
        // one effect -> no need for output textures
        // two effects -> 1 texture
        // more effects -> swap chain (2 textures)
        match effects_count {
            2 => texture_views.push(create_output_texture_view(device, config)),
            _ => texture_views.extend(create_output_texture_views(device, config, 2))
        }

        let mut in_texture_id = -1;
        let mut in_texture: &wgpu::TextureView;

        for i in 0..effects_count {
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


