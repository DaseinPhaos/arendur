// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Transformation interface

use super::foundamental::*;
use super::bbox::BBox3;
use super::interaction::{InteractInfo, DuvInfo, SurfaceInteraction};
use super::{Ray, RayDifferential};

/// An object that can transform geometry entities.
pub trait TransformExt: Transform3<Float> + Copy {
    #[inline]
    fn transform_ray<R>(&self, ray: &R) -> R
        where R: Ray
    {
        let m : Matrix4f = (*self).into();
        ray.apply_transform(&m)
    }

    #[inline]
    fn transform_ray_differential(&self, raydif: &RayDifferential) -> RayDifferential {
        let m : Matrix4f = (*self).into();
        raydif.apply_transform(&m)
    }

    #[inline]
    fn transform_bbox(&self, bbox: &BBox3<Float>) -> BBox3<Float> {
        bbox.apply_transform(self)
    }

    #[inline]
    fn transform_interact_info(&self, info: &InteractInfo) -> InteractInfo {
        info.apply_transform(self)
    }

    #[inline]
    fn transform_derivative_info(&self, info: &DuvInfo) -> DuvInfo {
        info.apply_transform(self)
    }

    #[inline]
    fn transform_surface_interaction<'b>(&self, si: &SurfaceInteraction<'b>) -> SurfaceInteraction<'b> {
        si.apply_transform(self)
    }

    #[inline]
    fn transform_norm(&self, norm: Vector3f) -> Vector3f
    {
        let m = <Self as Into<Matrix4<_>>>::into(*self);
        let inverse_transpose = m.invert().expect("Invalid inversion").transpose();
        inverse_transpose.transform_vector(norm).normalize()
    }
}

impl<T> TransformExt for T where T: Transform3<Float> + Copy {}
