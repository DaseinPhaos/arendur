// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Distant light

use super::*;
use sample;

/// Distant light
pub struct DistantLight {
    pub intensity: RGBSpectrumf,
    dir: Vector3f,
    world_center: Point3f,
    world_radius: Float,
}

impl DistantLight {
    /// construction
    #[inline]
    pub fn new(intensity: RGBSpectrumf, dir: Vector3f) -> DistantLight {
        let dir = dir.normalize();
        DistantLight{
            intensity: intensity,
            dir: dir,
            world_center: Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float),
            world_radius: float::infinity(),
        }
    }

    /// set direction
    #[inline]
    pub fn set_direction(&mut self, towards: Vector3f) {
        self.dir = towards.normalize();
    }

    /// set world bounds according to components
    #[inline]
    pub fn set_world_bounds<C>(&mut self, components: &C)
        where C: Composable
    {
        let (world_center, world_radius) = components.bbox_parent().bsphere();
        self.world_radius = world_radius;
        self.world_center = world_center;
    }
}

impl Light for DistantLight {
    #[inline]
    fn flags(&self) -> LightFlag {
        LIGHT_INFINITE
    }

    #[inline]
    fn is_delta(&self) -> bool {
        false
    }

    /// Given a position `pos` in local frame and a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    ///
    /// Distant lights assume a uniform direction across the scene,
    /// so the returned light sample always points to that direction.
    #[inline]
    fn evaluate_sampled(&self, pos: Point3f, _sample: Point2f) -> LightSample {
        let radiance = self.intensity;
        let pfrom = pos + (-2.0 as Float * self.world_radius) * self.dir;
        let pto = pos;
        let pdf = 1.0 as Float;
        LightSample{
            radiance: radiance,
            pfrom: pfrom,
            pto: pto,
            pdf: pdf,
        }
    }

    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        let (u, v) = normal::get_basis_from(self.dir);

        let pdisk = sample::sample_concentric_disk(samples.pfilm);
        /// extend accordingly
        let pdisk = self.world_center + self.world_radius*(pdisk.x * u + pdisk.y * v);
        let pos = pdisk + self.dir * (-self.world_radius);
        
        PathInfo{
            ray: RawRay::from_od(pos, self.dir),
            normal: self.dir,
            pdfpos: 1. as Float / (self.world_radius * self.world_radius * float::pi()),
            pdfdir: 1. as Float,
            radiance: self.intensity,
        }
    }

    #[inline]
    fn pdf(&self, _pos: Point3f, _dir: Vector3f, _normal: Vector3f) -> (Float, Float) {
        (
            1. as Float / (self.world_radius * self.world_radius * float::pi()), 
            0. as Float
        )
    }

    #[inline]
    fn power(&self) -> RGBSpectrumf {
        self.intensity * (self.world_radius * self.world_radius * float::pi())
    }
}
