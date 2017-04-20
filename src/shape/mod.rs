// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines interface `Shape`, representing some geometric entity
//! which resides in certain coordinate frames.

use geometry::prelude::*;

/// A shape
pub trait Shape: Sync + Send
{
    // /// returns basic info of this shape
    // fn info(&self) -> &ShapeInfo;
    // fn orientation_reversed(&self) -> bool;

    // fn reverse_orientation(&mut self, reverse: bool);
    
    /// returns bounding box of the shape in its local frame
    fn bbox_local(&self) -> BBox3f;

    /// Tests for intersection.
    // /// - `ray` is in parent frame
    /// - if hit, return `t` as the parametric distance along the ray
    ///   to the hitting point., and a `surface_interaction` for hitting
    ///   information at the surface, in local frame.
    fn intersect_ray(&self, ray: &RawRay) -> Option<(Float, SurfaceInteraction)>;

    /// Tests if the interaction can occur. Implementation maybe faster
    /// than `self.intersect_ray`
    fn can_intersect(&self, ray: &RawRay) -> bool {
        self.intersect_ray(ray).is_some()
    }

    /// Return an estimation of the surface area of the shape, in local space
    fn surface_area(&self) -> Float;

    /// Sample the shape, return a point and normal of the sampled point
    fn sample(&self, sample: Point2f) -> (Point3f, Vector3f);

    /// pdf of a sampled interaction on the surface, defaults to `1/area`
    #[inline]
    fn pdf(&self, _p: Point3f, _n: Vector3f) -> Float {
        1. as Float / self.surface_area()
    }

    /// Sample the shape wrt some reference point and an associated
    /// incoming ray. defaults to ignore the references
    fn sample_wrt(&self, _posref: Point3f, _wi: Vector3f, sample: Point2f) -> (Point3f, Vector3f) {
        self.sample(sample)
    }

    /// Pdf wrt some reference point and an associated incoming ray
    fn pdf_wrt(&self, pos_ref: Point3f, wi: Vector3f) -> Float {
        let ray = RawRay::from_od(pos_ref, wi);
        if let Some((_t, si)) = self.intersect_ray(&ray) {
            (si.basic.pos - pos_ref).magnitude2() /
            (wi.dot(si.basic.norm).abs()*self.surface_area())
        } else {
            0. as Float
        }
    }
}

pub mod sphere;
pub mod triangle;
pub mod prelude;
#[cfg(test)]
mod tests;
