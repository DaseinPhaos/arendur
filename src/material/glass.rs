// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! glass material

use std::sync::Arc;
use spectrum::prelude::*;
use super::*;
use bxdf::prelude::*;
use bxdf::microfacet::roughness_to_alpha;

/// A glass material
#[derive(Clone)]
pub struct GlassMaterial {
    pub diffuse: Arc<Texture<Texel=RGBSpectrumf>>,
    pub specular: Arc<Texture<Texel=RGBSpectrumf>>,
    pub roughness: Arc<Texture<Texel=Float>>,
    pub eta: Float,
    pub bump: Option<Arc<Texture<Texel=Float>>>,
}

impl GlassMaterial {
    pub fn new(
        diffuse: Arc<Texture<Texel=RGBSpectrumf>>,
        specular: Arc<Texture<Texel=RGBSpectrumf>>,
        roughness: Arc<Texture<Texel=Float>>,
        eta: Float,
        bump: Option<Arc<Texture<Texel=Float>>>
    ) -> GlassMaterial {
        GlassMaterial{
            diffuse, specular, roughness, eta, bump
        }
    }
}

impl Material for GlassMaterial {
    fn compute_scattering<'a>(
        &self,
        si: &mut SurfaceInteraction,
        dxy: &DxyInfo,
        alloc: &'a Allocator
    ) -> bsdf::Bsdf<'a> {
        if let Some(ref bump) = self.bump {
            add_bumping(si, dxy, &**bump);
        }
        let specular = self.specular.evaluate(si, dxy);
        let diffuse = self.diffuse.evaluate(si, dxy);
        let roughness = self.roughness.evaluate(si, dxy);
        let alpha = roughness_to_alpha(roughness);
        let mut ret = bsdf::Bsdf::new(si, 1.0 as Float);
        if !specular.is_black() {
            ret.add(alloc.alloc(FresnelBxdf::new(
                specular, specular, 1. as Float, self.eta
            )));
        }
        if !diffuse.is_black() {
            // diffuse reflection
            ret.add(alloc.alloc(TorranceSparrowRBxdf::new(
                diffuse,
                Trowbridge{
                    ax: alpha, ay: alpha
                },
                Dielectric::new(1. as Float, self.eta)
            )));
            // diffuse transmission
            ret.add(alloc.alloc(TorranceSparrowTBxdf::new(
                diffuse, 
                Trowbridge{
                    ax: alpha, ay: alpha
                },
                1. as Float, self.eta
            )));
        }
        ret
    }
}
