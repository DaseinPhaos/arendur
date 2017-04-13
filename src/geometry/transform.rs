//! Transformation interface

use super::foundamental::*;
use super::bbox::BBox3;
use super::interaction::{InteractInfo, DerivativeInfo2D, SurfaceInteraction};
use super::{Ray, RayDifferential};

/// An object that can transform geometry entities.
pub trait TransformExt: Transform3<Float> + Copy {
    #[inline]
    fn transform_ray<R>(&self, ray: &R) -> R
        where R: Ray {
            ray.apply_transform(self)
    }

    #[inline]
    fn transform_ray_differential(&self, raydif: &RayDifferential) -> RayDifferential {
        raydif.apply_transform(self)
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
    fn transform_derivative_info(&self, info: &DerivativeInfo2D) -> DerivativeInfo2D {
        info.apply_transform(self)
    }

    #[inline]
    fn transform_surface_interaction<'a>(&self, si: &SurfaceInteraction<'a>) -> SurfaceInteraction<'a> {
        si.apply_transform(self)
    }

    #[inline]
    fn transform_norm(&self, norm: Vector3f) -> Vector3f
    {
        let m = <Self as Into<Matrix4<_>>>::into(*self);
        let inverse_transpose = m.inverse_transform().expect("Invalid inversion").transpose();
        inverse_transpose.transform_vector(norm).normalize()
    }
}

impl<T> TransformExt for T where T: Transform3<Float> + Copy {}