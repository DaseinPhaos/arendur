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

pub use self::sphere::Sphere;
pub use self::triangle::{TriangleInstance, TriangleMesh};

/// Basic information about a shape
/// Guarantees: 
/// - `local_parent.inverse() == parent_local`.
/// - `(local_parent.det() < 0) == swap_handedness`.
#[derive(Copy, Clone)]
pub struct ShapeInfo<'a> {
    /// transform from local coordinate frame into parent
    pub local_parent: &'a Matrix4f,
    /// transform from parent coordinate frame into local
    pub parent_local: &'a Matrix4f,
    /// indicates if the shape normal's orientation should be reversed
    pub reverse_orientation: bool,
    /// indicates if transforms swap handedness
    pub swap_handedness: bool,
}

impl<'a> ShapeInfo<'a> {
    /// Construct a new shape. Users should always use this method
    /// so that gurantees are met.
    pub fn new(local_parent: &'a Matrix4f, parent_local: &'a Matrix4f, reverse_orientation: bool) -> ShapeInfo<'a> {
        #[cfg(debug)]
        {
            let b = relative_eq!(local_parent, parent_local.inverse());
            debug_assert!(b, "invalid inpu matrix");
        }
        let swap_handedness = if local_parent.determinant() > (0.0 as Float) {
            true
        } else {
            false
        };
        ShapeInfo{
            local_parent: local_parent,
            parent_local: parent_local,
            reverse_orientation: reverse_orientation,
            swap_handedness: swap_handedness,
        }
    }
}


/// A shape
pub trait Shape
{
    /// returns basic info of this shape
    fn info(&self) -> ShapeInfo;
    
    /// returns bounding box of the shape in its local frame
    fn bbox_local(&self) -> BBox3f;

    /// returns bounding box of the shape in the parent frame
    fn bbox_parent(&self) -> BBox3f {
        let local_parent = self.info().local_parent;
        let bbox_local = self.bbox_local();
        local_parent.transform_bbox(&bbox_local)
    }

    /// Tests for intersection.
    /// - `ray` is in parent frame
    /// - if hit, return `t` as the parametric distance along the ray
    ///   to the hitting point., and a `surface_interaction` for hitting
    ///   information at the surface, in local frame.
    fn intersect_ray<R: Ray>(&self, ray: &R) -> Option<(Float, SurfaceInteraction)>;

    /// Tests if the interaction can occur. Implementation maybe faster
    /// than `self.intersect_ray`
    fn can_intersect<R: Ray>(&self, ray: &R) -> bool {
        self.intersect_ray(ray).is_some()
    }

    /// Return an estimation of the surface area of the shape, in local space
    fn surface_area(&self) -> Float;
}

pub mod sphere;
pub mod triangle;
