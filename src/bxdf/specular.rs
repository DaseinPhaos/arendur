// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Specular bxdfs

use super::*;
use super::fresnel::*;
use spectrum::Spectrum;

/// A specular reflectional bxdf
#[derive(Clone, Copy, Debug)]
pub struct SpecularRBxdf<F> {
    pub reflectance: RGBSpectrumf,
    pub fresnel: F,
}

impl<F: Fresnel> SpecularRBxdf<F> {
    /// construction
    #[inline]
    pub fn new(reflectance: RGBSpectrumf, fresnel: F) -> SpecularRBxdf<F> {
        SpecularRBxdf{
            reflectance: reflectance, fresnel: fresnel
        }
    }
}

impl<T: Fresnel> Bxdf for SpecularRBxdf<T> {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_REFLECTION | BXDF_SPECULAR
    }

    #[inline]
    fn evaluate(&self, _wo: Vector3f, _wi: Vector3f) -> RGBSpectrumf {
        RGBSpectrumf::black()
    }

    #[inline]
    fn evaluate_sampled(&self, wo: Vector3f, _sample: Point2f) -> (RGBSpectrumf, Vector3f, Float) {
        let r = Vector3f::new(-wo.x, -wo.y, wo.z);
        let cos = normal::cos_theta(r);
        let s = self.fresnel.evaluate(cos) * self.reflectance / cos.abs();
        (s, r, 1.0 as Float)
    }
}

/// A specular transmission bxdf
#[derive(Copy, Clone, Debug)]
pub struct SpecularTBxdf {
    pub transmitance: RGBSpectrumf,
    pub fresnel: Dielectric,
}

impl SpecularTBxdf {
    /// construction
    pub fn new(transmitance: RGBSpectrumf, eta_a: Float, eta_b: Float) -> SpecularTBxdf {
        SpecularTBxdf{
            transmitance: transmitance,
            fresnel: Dielectric::new(eta_a, eta_b),
        }
    }
}

impl Bxdf for SpecularTBxdf {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_TRANSMISSION | BXDF_SPECULAR
    }

    #[inline]
    fn evaluate(&self, _wo: Vector3f, _wi: Vector3f) -> RGBSpectrumf {
        RGBSpectrumf::black()
    }

    #[inline]
    fn evaluate_sampled(&self, wo: Vector3f, _sample: Point2f) -> (RGBSpectrumf, Vector3f, Float) {
        let r = Vector3f::new(-wo.x, -wo.y, wo.z);
        let cos = normal::cos_theta(r);
        let t = RGBSpectrumf::grey_scale(1.0 as Float) - self.fresnel.evaluate(cos);
        // TODO: Double check
        (t*self.transmitance/cos.abs(), r, 1.0 as Float)
    }

    #[inline]
    fn rho_hh(&self, _samples0: &[Point2f], _samples1: &[Point2f]) -> RGBSpectrumf {
        unimplemented!();
    }
}

// TODO: generalize a fresnell specular bxdf
