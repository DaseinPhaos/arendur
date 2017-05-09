// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines Lambertian bxdf

use super::*;

/// A lambertian reflection bxdf.
#[derive(Copy, Clone, Debug)]
pub struct LambertianRBxdf {
    pub reflectance: RGBSpectrumf,
}

impl LambertianRBxdf {
    /// construction
    pub fn new(reflectance: RGBSpectrumf) -> LambertianRBxdf {
        LambertianRBxdf{
            reflectance: reflectance
        }
    }
}

impl Bxdf for LambertianRBxdf {
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

/// A lambertian transmission bxdf
#[derive(Copy, Clone, Debug)]
pub struct LambertianTBxdf {
    pub transmittance: RGBSpectrumf,
}

impl LambertianTBxdf {
    /// construction
    pub fn new(transmittance: RGBSpectrumf) -> LambertianTBxdf {
        LambertianTBxdf{
            transmittance
        }
    }
}

impl Bxdf for LambertianTBxdf {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_TRANSMISSION | BXDF_DIFFUSE
    }

    #[inline]
    fn evaluate(&self, _wo: Vector3f, _wi: Vector3f) -> RGBSpectrumf {
        self.transmittance * float::frac_1_pi()
    }

    #[inline]
    fn rho_hd(&self, _wo: Vector3f, _samples: &[Point2f]) -> RGBSpectrumf {
        self.transmittance
    }

    #[inline]
    fn rho_hh(&self, _samples0: &[Point2f], _samples1: &[Point2f]) -> RGBSpectrumf {
        self.transmittance
    }

    #[inline]
    fn evaluate_sampled(&self, wo: Vector3f, u: Point2f
    ) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        let mut wi = sample::sample_cosw_hemisphere(u);
        if wo.z > 0.0 as Float {wi.z = -wi.z;}
        let pdf = self.pdf(wo, wi);
        let spectrum = self.evaluate(wo, wi);
        (spectrum, wi, pdf, self.kind())
    }

    #[inline]
    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        if wo.z * wi.z >= 0. as Float {
            0. as Float
        } else {
            normal::cos_theta(wi).abs() * float::frac_1_pi()
        }
    }
}
