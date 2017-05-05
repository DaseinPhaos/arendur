// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines Lambertian bxdf

use super::*;

/// A lambertian bxdf.
#[derive(Copy, Clone, Debug)]
pub struct LambertianBxdf {
    pub reflectance: RGBSpectrumf,
}

impl LambertianBxdf {
    /// construction
    pub fn new(reflectance: RGBSpectrumf) -> LambertianBxdf {
        LambertianBxdf{
            reflectance: reflectance
        }
    }
}

impl Bxdf for LambertianBxdf {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_REFLECTION | BXDF_DIFFUSE
    }

    #[inline]
    fn evaluate(&self, _wo: Vector3f, _wi: Vector3f) -> RGBSpectrumf {
        self.reflectance * float::frac_1_pi()
    }

    #[inline]
    fn rho_hd(&self, _wo: Vector3f, _samples: &[Point2f]) -> RGBSpectrumf {
        self.reflectance
    }

    #[inline]
    fn rho_hh(&self, _samples0: &[Point2f], _samples1: &[Point2f]) -> RGBSpectrumf {
        self.reflectance
    }
}
