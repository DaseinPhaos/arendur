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
use std::sync::Arc;

pub use self::sphere::Sphere;
pub use self::triangle::{TriangleInstance, TriangleMesh};

// /// Basic information about a shape
// /// Guarantees: 
// /// - `local_parent.inverse() == parent_local`.
// /// - `(local_parent.det() < 0) == swap_handedness`.
// #[derive(Clone)]
// pub struct ShapeInfo {
//     /// transform from local coordinate frame into parent
//     pub local_parent: Arc<Matrix4f>,
//     /// transform from parent coordinate frame into local
//     pub parent_local: Arc<Matrix4f>,
//     /// indicates if the shape normal's orientation should be reversed
//     pub reverse_orientation: bool,
//     /// indicates if transforms swap handedness
//     pub swap_handedness: bool,
// }

// impl ShapeInfo {
//     /// Construct a new shape. Users should always use this method
//     /// so that gurantees are met.
//     pub fn new(local_parent: Arc<Matrix4f>, parent_local: Arc<Matrix4f>, reverse_orientation: bool) -> ShapeInfo {
//         #[cfg(debug)]
//         {
//             let b = relative_eq!(local_parent, parent_local.inverse());
//             debug_assert!(b, "invalid inpu matrix");
//         }
//         let swap_handedness = if local_parent.determinant() > (0.0 as Float) {
//             true
//         } else {
//             false
//         };
//         ShapeInfo{
//             local_parent: local_parent,
//             parent_local: parent_local,
//             reverse_orientation: reverse_orientation,
//             swap_handedness: swap_handedness,
//         }
//     }
// }


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
#[cfg(test)]
mod tests;