// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines the general bxdf (bidirectional distribution function) interface

use geometry::prelude::*;
use spectrum::{Spectrum, RGBSpectrumf};
use sample;
use std::cmp;

/// A bidirectional distribution function
pub trait Bxdf {
    /// returns the type of the bxdf
    fn kind(&self) -> BxdfType;

    /// check if the type matches
    #[inline]
    fn is(&self, t: BxdfType) -> bool {
        (self.kind().bits & t.bits) == self.kind().bits
    }

    /// evaluate the function given two normalized directions
    fn evaluate(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf;

    /// Given an outgoing direction `wo`, and a uniform sample
    /// `u` from $[0,1)^2$, sample an incoming direction `wi`,
    /// and returns it with function value evaluated as `f(wo, wi)`,
    /// as well as the pdf associated with the incoming direction,
    /// as well as the type of the scattering event.
    ///
    /// The default implementation samples the incoming direction
    /// with a cos-weighted distribution above the hemisphere,
    /// then returns the corresponding evaluation and pdf with invocations
    /// to `evaluate` and `pdf`. Bxdfs having better distribution descriptions
    /// should overwrite the behavior when needed.
    #[inline]
    fn evaluate_sampled(&self, wo: Vector3f, u: Point2f) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        let mut wi = sample::sample_cosw_hemisphere(u);
        if wo.z < 0.0 as Float {wi.z = -wi.z;}
        let pdf = self.pdf(wo, wi);
        let spectrum = self.evaluate(wo, wi);
        (spectrum, wi, pdf, self.kind())
    }

    /// evaluate the function given two normalized directions,
    /// the particle being traced is camera-ray importance,
    /// rather than light radiance.
    ///
    /// default implementation assumes bxdf have symmetrical scattering 
    /// properties and just forwards the call to `self.evaluate`
    #[inline]
    fn evaluate_importance(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf {
        self.evaluate(wo, wi)
    }

    /// Given an outgoing direction `wo`, and a uniform sample
    /// `u` from $[0,1)^2$, sample an incoming direction `wi`,
    /// and returns it with function value evaluated as `f(wo, wi)`,
    /// as well as the pdf associated with the incoming direction，
    /// as well as the type of the scattering event.
    /// The particles being traced is camera-ray importance, rather tan light radiance
    ///
    /// default implementation assumes bxdf have symmetrical scattering 
    /// properties and just forwards the call to `self.evaluate_sampled`
    #[inline]
    fn evaluate_importance_sampled(&self, wo: Vector3f, u: Point2f) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        self.evaluate_sampled(wo, u)
    }

    /// evalute pdf given the incoming and outgoing direction
    #[inline]
    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        if wo.z * wi.z > 0.0 as Float {
            normal::cos_theta(wi).abs() * float::frac_1_pi()
        } else {
            0.0 as Float
        }
    }

    /// hemispherical-directional reflectance
    // TODO: explain more
    fn rho_hd(&self, wo: Vector3f, samples: &[Point2f]) -> RGBSpectrumf {
        let mut ret = RGBSpectrumf::black();
        for sample in samples {
            let (spec, wi, pdf, _) = self.evaluate_sampled(wo, *sample);
            if pdf > 0.0 as Float {
                ret += spec * normal::cos_theta(wi).abs() / pdf;
            }
        }
        ret/(samples.len() as Float)
    }

    /// hemispherical-hemispherical reflactance
    // TODO: explain more
    fn rho_hh(&self, samples0: &[Point2f], samples1: &[Point2f]) -> RGBSpectrumf {
        let mut ret = RGBSpectrumf::black();
        let nsamples = cmp::min(samples0.len(), samples1.len());
        for i in 0..nsamples {
            let pdfo = sample::pdf_uniform_hemisphere();
            let wo = unsafe {
                sample::sample_uniform_hemisphere(*samples0.get_unchecked(i))
            };
            let (spec, wi, pdfi, _) = unsafe {
                self.evaluate_sampled(wo, *samples1.get_unchecked(i))
            };
            if pdfi > 0.0 as Float {
                ret += spec * (normal::cos_theta(wi)*normal::cos_theta(wo)).abs() / (pdfi * pdfo);
            }
        }
        ret / (nsamples as Float)
    }
}

bitflags! {
    pub flags BxdfType: u32 {
        const BXDF_REFLECTION = 0x01,
        const BXDF_TRANSMISSION = 0x02,
        const BXDF_DIFFUSE = 0x04,
        const BXDF_GLOSSY = 0x08,
        const BXDF_SPECULAR = 0x10,
        const BXDF_ALL = BXDF_REFLECTION.bits
                  | BXDF_TRANSMISSION.bits
                  | BXDF_DIFFUSE.bits
                  | BXDF_GLOSSY.bits
                  | BXDF_SPECULAR.bits,
    }
}

pub mod scaled;
pub mod fresnel;
pub mod specular;
pub mod lambertian;
pub mod oren_nayar;
pub mod prelude;
pub mod microfacet;
