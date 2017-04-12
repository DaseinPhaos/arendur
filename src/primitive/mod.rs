//! A primitive represents a renderable entity in the world.

use geometry::*;

/// A renderable composable entity.
pub trait Composable {
    /// returns bounding box in parent frame.
    fn bbox_parent(&self) -> BBox3f;

    /// test for intersection. Note its different guarantees from `Shape`:
    /// - `ray` is in parent frame,
    /// - if hit, returns surface interaction data, in parent frame.
    /// - if hit, `ray`'s tmax would be updated to the hitting `t`.
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<(SurfaceInteraction, &Primitive)>;

    /// test if an intersection can occur. Might be more efficient
    fn can_intersect(&self, ray: &RawRay) -> bool {
        let mut ray = ray.clone();
        self.intersect_ray(&mut ray).is_some()
    }

    // TODO: Add arealight accessor

    // TODO: Add material accessor

    // TODO: Add bxdf computer
}

/// An aggregated renderable entity
pub trait Aggregate: Composable {

}

/// A renderable primitive entity
pub trait Primitive: Composable {

}


pub mod shape;
pub mod transformed;
// pub mod bvh;
pub mod naive;