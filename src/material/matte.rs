// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Matte material
use std::sync::Arc;
use spectrum::RGBSpectrumf;
use super::*;
use bxdf::lambertian::LambertianRBxdf;
use bxdf::oren_nayar::OrenNayer;
//use std::ops::Deref;


/// Matte material
#[derive(Clone)]
pub struct MatteMaterial {
    pub kd: Arc<Texture<Texel=RGBSpectrumf>>,
    pub sigma: Arc<Texture<Texel=Float>>,
    pub bump: Option<Arc<Texture<Texel=Float>>>,
}

impl MatteMaterial {
    /// construction
    #[inline]
    pub fn new(kd: Arc<Texture<Texel=RGBSpectrumf>>,
    sigma: Arc<Texture<Texel=Float>>,
    bump: Option<Arc<Texture<Texel=Float>>>) -> Self {
        MatteMaterial{
            kd: kd, sigma: sigma, bump: bump,
        }
    }
}

impl Material for MatteMaterial {
    fn compute_scattering<'a>(
        &self,
        si: &mut SurfaceInteraction,
        dxy: &DxyInfo,
        alloc: &mut Allocator<'a>
    ) -> bsdf::Bsdf<'a>
    {
        if let Some(ref bump) = self.bump {
            add_bumping(si, dxy, &**bump);
        }
        let r = self.kd.evaluate(si, dxy);
        let sig = float::clamp(
            self.sigma.evaluate(si, dxy),
            0.0 as Float,
            90.0 as Float
        );
        let mut ret = bsdf::Bsdf::new(si, 1.0 as Float);
        if !r.is_black() {
            if sig == 0.0 as Float {
                ret.add(alloc.alloc(LambertianRBxdf::new(r)));
            } else {
                ret.add(alloc.alloc(OrenNayer::new(r, sig)));
            }
        }
        ret
    }
}
