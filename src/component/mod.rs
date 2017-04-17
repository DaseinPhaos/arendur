// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines renderable components in the world.

use geometry::prelude::*;
use lighting::Light;
use material::Material;

/// A renderable composable component.
pub trait Composable: Sync + Send {
    /// returns bounding box in parent frame.
    fn bbox_parent(&self) -> BBox3f;

    /// test for intersection. Note that its guarantees are from `Shape`'s:
    /// - `ray` is specified in parent frame,
    /// - if hit, returns surface interaction data in *parent* frame.
    /// - if hit, `ray`'s `tmax` would be updated to the hitting `t`.
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction>;

    /// test if an intersection can occur. Might be more efficient
    fn can_intersect(&self, ray: &RawRay) -> bool {
        let mut ray = ray.clone();
        self.intersect_ray(&mut ray).is_some()
    }
}

// /// An aggregated renderable entity
// pub trait Aggregate: Composable {

// }

/// A renderable primitive
pub trait Primitive: Composable {
    // TODO: Add arealight accessor
    fn get_area_light(&self) -> Option<&Light> {
        None
    }

    // TODO: Add material accessor
    fn get_material(&self) -> &Material;

    // TODO: Add bxdf computer
}


pub mod shape;
pub mod transformed;
// pub mod bvh;
pub mod naive;
