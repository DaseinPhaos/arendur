// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Distant light

use super::*;

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
        LIGHIT_INFINITE
    }

    #[inline]
    fn is_delta(&self) -> bool {
        false
    }

    #[inline]
    fn evaluate_sampled(&self, posw: Point3f, _sample: Point2f) -> LightSample {
        let radiance = self.intensity;
        let pfrom = posw + (-2.0 as Float * self.world_radius) * self.dir;
        let pto = posw;
        let pdf = 1.0 as Float;
        LightSample{
            radiance: radiance,
            pfrom: pfrom,
            pto: pto,
            pdf: pdf,
        }
    }

    #[inline]
    fn power(&self) -> RGBSpectrumf {
        self.intensity * (self.world_radius * self.world_radius * float::pi())
    }
}