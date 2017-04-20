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
}

pub mod sphere;
pub mod triangle;
pub mod prelude;
#[cfg(test)]
mod tests;
