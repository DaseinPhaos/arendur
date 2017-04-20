// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines a scaled bxdf
use super::*;

/// A scaled bxdf. Values of the inner bxdf would be scaled by
/// the scaling factor `scale` and be returned
pub struct ScaledBxdf<T> {
    inner: T,
    /// the scaling factor
    pub scale: RGBSpectrumf,
}

impl<T: Bxdf> Bxdf for ScaledBxdf<T> {
    #[inline]
    fn kind(&self) -> BxdfType {
        self.inner.kind()
    }

    #[inline]
    fn evaluate(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf {
        self.inner.evaluate(wo, wi) * self.scale
    }

    #[inline]
    fn evaluate_sampled(&self, wo: Vector3f, sample: Point2f) -> (RGBSpectrumf, Vector3f, Float) {
        let (s, v, f) = self.inner.evaluate_sampled(wo, sample);
        (s * self.scale, v, f)
    }

    #[inline]
    fn rho_hd(&self, wo: Vector3f, samples: &[Point2f]) -> RGBSpectrumf {
        self.inner.rho_hd(wo, samples) * self.scale
    }

    #[inline]
    fn rho_hh(&self, samples0: &[Point2f], samples1: &[Point2f]) -> RGBSpectrumf {
        self.inner.rho_hh(samples0, samples1) * self.scale
    }
}