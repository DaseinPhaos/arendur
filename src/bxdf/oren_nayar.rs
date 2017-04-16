// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Microfacet described as per Oren-Nayer

use super::*;

/// An Oren-Nayer brdf as per https://en.wikipedia.org/wiki/Oren%E2%80%93Nayar_reflectance_model
#[derive(Copy, Clone, Debug)]
pub struct OrenNayer {
    reflectance: RGBSpectrumf,
    coef_a: Float,
    coef_b: Float,
}

impl OrenNayer {
    /// construction, sigma in radians
    pub fn new(reflectance: RGBSpectrumf, sigma: Float) -> OrenNayer {
        let sigma2 = sigma * sigma;
        let coef_a = 1.0 as Float - (sigma2 / (2.0 as Float * (sigma2 + 0.33 as Float)));
        let coef_b = (0.45 as Float * sigma2) / (sigma2 + 0.09 as Float);
        OrenNayer {
            reflectance: reflectance,
            coef_a: coef_a,
            coef_b: coef_b,
        }
    }
}

impl Bxdf for OrenNayer {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_REFLECTION | BXDF_DIFFUSE
    }

    fn evaluate(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf {
        let sin_theta_i = normal::sin_theta(wi);
        let sin_theta_o = normal::sin_theta(wo);
        let mut max_cos = 0.0 as Float;
        if sin_theta_i > 1e-4 as Float || sin_theta_o > 1e-4 as Float {
            let sin_phi_i = normal::sin_phi(wi);
            let sin_phi_o = normal::sin_phi(wo);
            let cos_phi_i = normal::cos_phi(wi);
            let cos_phi_o = normal::cos_phi(wo);
            max_cos = max_cos.max(cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o);
        }
        let ci = normal::cos_theta(wi).abs();
        let co = normal::cos_theta(wo).abs();
        let (sin_a, tan_b) = if ci > co {
            (sin_theta_o, sin_theta_i/ci)
        } else {
            (sin_theta_i, sin_theta_o/co)
        };
        self.reflectance * float::frac_1_pi() * (self.coef_a + self.coef_b * max_cos * sin_a * tan_b)
    }

    #[inline]
    fn rho_hh(&self, _samples0: &[Point2f], _samples1: &[Point2f]) -> RGBSpectrumf {
        unimplemented!();
    }
}
