// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! glass material

use std::sync::Arc;
use spectrum::RGBSpectrumf;
use super::*;
use bxdf::prelude::*;

/// A glass material
#[derive(Clone)]
pub struct GlassMaterial {
    pub transmittance: Arc<Texture<Texel=RGBSpectrumf>>,
    pub reflectance: Arc<Texture<Texel=RGBSpectrumf>>,
    pub dissolve: Float,
    pub eta: Float,
    pub bump: Option<Arc<Texture<Texel=Float>>>,
}

impl GlassMaterial {
    pub fn new(
        transmittance: Arc<Texture<Texel=RGBSpectrumf>>,
        reflectance: Arc<Texture<Texel=RGBSpectrumf>>,
        dissolve: Float, eta: Float,
        bump: Option<Arc<Texture<Texel=Float>>>
    ) -> GlassMaterial {
        GlassMaterial{
            transmittance, reflectance, dissolve, eta, bump
        }
    }
}

impl Material for GlassMaterial {
    fn compute_scattering<'a>(
        &self,
        si: &mut SurfaceInteraction,
        dxy: &DxyInfo,
        alloc: &mut Allocator<'a>
    ) -> bsdf::Bsdf<'a> {
        if let Some(ref bump) = self.bump {
            add_bumping(si, dxy, &**bump);
        }
        let reflectance = self.reflectance.evaluate(si, dxy) * self.dissolve;
        let transmittance = self.transmittance.evaluate(si, dxy) * (1. as Float - self.dissolve);
        let mut ret = bsdf::Bsdf::new(si, 1.0 as Float);
        ret.add(alloc.alloc(
            FresnelBxdf::new(
                reflectance, transmittance, 1. as Float, self.eta
            )
        ));
        ret
    }
}
