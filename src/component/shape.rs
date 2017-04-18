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
pub struct ShapedPrimitive {
    shape: Arc<Shape>,
    material: Arc<Material>,
    area_light: Option<Arc<Light>>,
    // TODO: medium:
}

impl ShapedPrimitive {
    /// construction
    #[inline]
    pub fn new(shape: Arc<Shape>, material: Arc<Material>, area_light: Option<Arc<Light>>) -> ShapedPrimitive {
        ShapedPrimitive{
            shape: shape, material: material, area_light: area_light,
        }
    }
}

impl Composable for ShapedPrimitive
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.shape.bbox_parent()
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        let r = self.shape.intersect_ray(ray);
        if let Some((t, mut si)) = r {
            ray.set_max_extend(t);
            si.set_primitive(self);
            // transform si into parent frame
            let tlp = *self.shape.info().parent_local;

            Some(si.apply_transform(&tlp))
        } else {
            None
        }
    }
}

impl Primitive for ShapedPrimitive
{
    fn get_material(&self) -> &Material {
        &*self.material
    }
}
