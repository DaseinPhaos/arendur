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
pub use filming::SampleInfo;

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
        self.evaluate_path(rd.ray.origin(), rd.ray.direction())
    }

    /// Given a position and an incoming direction in local coordinates,
    /// evaluate the light's radiance along that direction.
    ///
    /// Default implementation yields zero radiance
    #[inline]
    fn evaluate_path(&self, _pos: Point3f, _wi: Vector3f) -> RGBSpectrumf {
        RGBSpectrumf::black()
    }

    /// Given a `pos` in local frame with a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    fn evaluate_sampled(&self, pos: Point3f, sample: Point2f) -> LightSample;

    /// Generate a photon path from the light source based on the sample info
    fn generate_path(&self, samples: SampleInfo) -> PathInfo;

    /// Given position and direction of a photon path, and the light's `normal`
    /// return its pdfs as `(pdfpos, pdfdir)`
    fn pdf(&self, pos: Point3f, dir: Vector3f, normal: Vector3f) -> (Float, Float);
    

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
        // TODO: check floating point error
        let epsilon = Point3f::default_epsilon();
        let epsilon = Vector3f::new(epsilon, epsilon, epsilon);
        let pfrom = self.pfrom + epsilon;
        // let pto = self.pto + (-epsilon);
        let mut ray = RawRay::spawn(pfrom, self.pto);
        if let Some(si) = components.intersect_ray(&mut ray) {
            !relative_eq!(si.basic.pos, self.pto)
        } else {
            false
        }
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

/// Information about a photon path
#[derive(Debug, PartialEq, Copy, Clone)]
#[must_use]
pub struct PathInfo {
    /// originate position and direction of this path
    pub ray: RawRay,
    /// light's normal vector
    pub normal: Vector3f,
    /// pdf wrt the originate position
    pub pdfpos: Float,
    /// pdf wrt the light direction
    pub pdfdir: Float,
    /// radiance
    pub radiance: RGBSpectrumf,
}

impl PathInfo {
    #[inline]
    pub fn apply_transform(&self, t: &Matrix4f) -> Self {
        PathInfo{
            ray: self.ray.apply_transform(t),
            normal: t.transform_norm(self.normal),
            pdfpos: self.pdfpos,
            pdfdir: self.pdfdir,
            radiance: self.radiance,
        }
    }
}

pub mod pointlights;
pub mod distantlight;
pub mod prelude;
