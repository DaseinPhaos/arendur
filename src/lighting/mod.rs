// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The Lighting Interface.

use geometry::prelude::*;
use spectrum::*;
use component::Composable;
use renderer::scene::Scene;

/// A Light
pub trait Light: Sync+ Send {
    /// return the flags of the light
    fn flags(&self) -> LightFlag;

    /// test if the light has delta distribution
    #[inline]
    fn is_delta(&self) -> bool {
        self.flags().is_delta()
    }

    /// Given a position and an incoming direction in local coordinates,
    /// evaluate the light's radiance along that direction. This method
    /// takes an `RayDifferential` because some light implementations
    /// might found thouse differentials helpful.
    ///
    /// Default implementation yields zero radiance
    #[inline]
    fn evaluate_ray(&self, rd: &RayDifferential) -> RGBSpectrumf {
        self.evaluate(rd.ray.origin(), rd.ray.direction())
    }

    /// Given a position and an incoming direction in local coordinates,
    /// evaluate the light's radiance along that direction.
    ///
    /// Default implementation yields zero radiance
    #[inline]
    fn evaluate(&self, _pos: Point3f, _wi: Vector3f) -> RGBSpectrumf {
        RGBSpectrumf::black()
    }

    /// Given a `pos` in local frame with a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    fn evaluate_sampled(&self, pos: Point3f, sample: Point2f) -> LightSample;

    /// returns an estimation of total power of this light
    fn power(&self) -> RGBSpectrumf;

    /// preporcess with scene components, if necessary.
    /// renderers should respect this requirement.
    ///
    /// Default implementation is noop.
    #[inline]
    fn preprocess(&mut self, _s: &Scene) { }
}

// /// An area light
// pub trait AreaLight: Light {
//     /// evaluate 
//     fn evalute()
// }

bitflags! {
    pub flags LightFlag: u32 {
        const LIGHT_DPOS = 0x1,
        const LIGHT_DDIR = 0x2,
        const LIGHT_AREA = 0x4,
        const LIGHT_INFINITE = 0x8,
    }
}

impl LightFlag {
    /// test if the light is delta light
    pub fn is_delta(self) -> bool {
        (self & LIGHT_DPOS == LIGHT_DPOS) ||
        (self & LIGHT_DDIR == LIGHT_DDIR)
    }
}

/// Results of a light's sampling evaluation
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct LightSample {
    /// outgoing radiance
    pub radiance: RGBSpectrumf,
    /// pdf for this sample
    pub pdf: Float,
    /// outgoing point, parent frame
    pub pfrom: Point3f,
    /// receiving point, parent frame
    pub pto: Point3f,
}

#[must_use]
impl LightSample {
    /// get light direction vector `wi`
    #[inline]
    pub fn wi(&self) -> Vector3f {
        (self.pfrom - self.pto).normalize()
    }

    /// test if this light would be occulued by any components
    /// in `Composable`, assuming they are in the same world frame
    #[inline]
    pub fn occluded<C: Composable + ?Sized>(&self, components: &C) -> bool {
        let ray = RawRay::spawn(self.pfrom, self.pto);
        components.can_intersect(&ray)
    }

    #[inline]
    pub fn apply_transform<T>(&self, t: &T) -> LightSample
        where T: TransformExt
    {
        LightSample {
            radiance: self.radiance,
            pdf: self.pdf,
            pfrom: t.transform_point(self.pfrom),
            pto: t.transform_point(self.pto),
        }
    }

    #[inline]
    pub fn no_effect(&self) -> bool {
        self.pdf == 0.0 as Float || self.radiance == RGBSpectrumf::black()
    }
}

pub mod pointlights;
pub mod distantlight;
pub mod arealights;
pub mod prelude;
