// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A plastic material

use std::sync::Arc;
use spectrum::RGBSpectrumf;
use super::*;
use bxdf::prelude::*;
use bxdf::microfacet::roughness_to_alpha;

/// A plastic material
#[derive(Clone)]
pub struct PlasticMaterial {
    pub diffuse: Arc<Texture<Texel=RGBSpectrumf>>,
    pub specular: Arc<Texture<Texel=RGBSpectrumf>>,
    pub roughness: Arc<Texture<Texel=Float>>,
    pub bump: Option<Arc<Texture<Texel=Float>>>,
}

impl PlasticMaterial {
    pub fn new(
        diffuse: Arc<Texture<Texel=RGBSpectrumf>>,
        specular: Arc<Texture<Texel=RGBSpectrumf>>,
        roughness: Arc<Texture<Texel=Float>>,
        bump: Option<Arc<Texture<Texel=Float>>>
    ) -> PlasticMaterial {
        PlasticMaterial{
            diffuse, specular, roughness, bump
        }
    }
}

impl Material for PlasticMaterial {
    fn compute_scattering<'a>(
        &self,
        si: &mut SurfaceInteraction,
        dxy: &DxyInfo,
        alloc: &mut Allocator<'a>
    ) -> bsdf::Bsdf<'a> {
        if let Some(ref bump) = self.bump {
            add_bumping(si, dxy, &**bump);
        }
        let diffuse = self.diffuse.evaluate(si, dxy);
        let specular = self.specular.evaluate(si, dxy);
        let roughness = self.roughness.evaluate(si, dxy);
        let alpha = roughness_to_alpha(roughness);
        let mut ret = bsdf::Bsdf::new(si, 1.0 as Float);
        ret.add(alloc.alloc(
            AshikhminShirleyBxdf::new(
                diffuse, specular,
                Beckmann{
                    ax: alpha, ay: alpha
                }
            )
        ));
        ret
    }
}
