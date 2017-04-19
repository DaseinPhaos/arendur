// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Primitive component made up by a `Shape`.

use geometry::prelude::*;
use super::*;
use std::sync::Arc;
use shape::*;

/// Represents a primitive made up by a single `Shape`
pub struct ShapedPrimitive<S, M> {
    shape: S,
    material: M,
    area_light: Option<Arc<Light>>,
    // TODO: medium:
}

impl<S, M> ShapedPrimitive<S, M>
    where S: Shape, M: Material
{
    /// construction
    #[inline]
    pub fn new(shape: S, material: M, area_light: Option<Arc<Light>>) -> ShapedPrimitive<S, M> {
        ShapedPrimitive{
            shape: shape, material: material, area_light: area_light,
        }
    }
}

impl<S, M> Composable for ShapedPrimitive<S, M>
    where S: Shape, M: Material
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.shape.bbox_local()
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        let r = self.shape.intersect_ray(ray);
        if let Some((t, mut si)) = r {
            ray.set_max_extend(t);
            si.set_primitive(self);
            Some(si)
        } else {
            None
        }
    }
}

impl<S, M> Primitive for ShapedPrimitive<S, M>
    where S: Shape, M: Material
{
    #[inline]
    fn get_material(&self) -> &Material {
        &self.material
    }

    #[inline]
    fn get_area_light(&self) -> Option<&Light> {
        if let Some(ref al) = self.area_light {
            Some(al.as_ref())
        } else {
            None
        }
    }
}
