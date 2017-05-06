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

    /// evaluate the function given two normalized directions.
    ///
    /// As specular surfaces are `totally` specular, this method
    /// always returns zero factoring 
    #[inline]
    fn evaluate(&self, _wo: Vector3f, _wi: Vector3f) -> RGBSpectrumf {
        RGBSpectrumf::black()
    }

    /// Given an outgoing direction `wo`, and a uniform sample
    /// `u` from $[0,1)^2$, sample an incoming direction `wi`,
    /// and returns it with function value evaluated as `f(wo, wi)`,
    /// as well as the pdf associated with the incoming direction.
    ///
    /// Specular surfaces choose the incoming direction wrt `wo` only,
    /// with pdf always equals to one.
    /// Evaluation behavior are described by the fresnel factor
    #[inline]
    fn evaluate_sampled(&self, wo: Vector3f, _sample: Point2f) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        let r = Vector3f::new(-wo.x, -wo.y, wo.z);
        let cos = normal::cos_theta(r);
        let s = self.fresnel.evaluate(cos) * self.reflectance / cos.abs();
        (s, r, 1.0 as Float, self.kind())
    }
}

/// A specular transmission bxdf
#[derive(Copy, Clone, Debug)]
pub struct SpecularTBxdf {
    pub transmittance: RGBSpectrumf,
    pub fresnel: Dielectric,
}

impl SpecularTBxdf {
    /// construction
    pub fn new(transmittance: RGBSpectrumf, eta_a: Float, eta_b: Float) -> SpecularTBxdf {
        SpecularTBxdf{
            transmittance: transmittance,
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
    fn evaluate_sampled(&self, wo: Vector3f, _sample: Point2f) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        let r = Vector3f::new(-wo.x, -wo.y, wo.z);
        let cos = normal::cos_theta(r);
        let t = RGBSpectrumf::grey_scale(1.0 as Float) - self.fresnel.evaluate(cos);
        // TODO: Double check
        (t*self.transmittance/cos.abs(), r, 1.0 as Float, self.kind())
    }

    #[inline]
    fn rho_hh(&self, _samples0: &[Point2f], _samples1: &[Point2f]) -> RGBSpectrumf {
        unimplemented!();
    }
}

// TODO: generalize a fresnell specular bxdf
