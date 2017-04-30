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
    #[inline]
    fn can_intersect(&self, ray: &RawRay) -> bool {
        let mut ray = ray.clone();
        self.intersect_ray(&mut ray).is_some()
    }

    fn as_light(&self) -> &Light {
        unimplemented!();
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        1.0 as Float
    }
}

// /// An aggregated renderable entity
// pub trait Aggregate: Composable {

// }

/// A renderable primitive
pub trait Primitive: Composable + Light {
    /// return if the primitive can emit lights
    fn is_emissive(&self) -> bool;

    /// return the material associated with this primitive
    fn get_material(&self) -> &Material;
}


pub mod shape;
pub mod transformed;
pub mod bvh;
pub mod naive;
pub mod prelude;
