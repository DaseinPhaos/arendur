// Copyright 2017 Dasein Phaos aka. Luxko aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Primitive component made up by a `Shape`.

use geometry::prelude::*;
use super::*;
use std::rc::Rc;
use shape::*;

/// Represents a primitive made up by a single `Shape`
pub struct ShapedPrimitive<T> {
    shape: Rc<T>,
    // TODO: material:
    // TODO: area_light:
    // TODO: medium:
}

impl<T> Composable for ShapedPrimitive<T>
    where T: Shape
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.shape.bbox_parent()
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<(SurfaceInteraction, &Primitive)> {
        let r = self.shape.intersect_ray(ray);
        if let Some((t, si)) = r {
            ray.set_max_extend(t);
            // transform si into parent frame
            let tlp = self.shape.info().parent_local;
            Some((si.apply_transform(tlp), self))
        } else {
            None
        }
    }
}

impl<T> Primitive for ShapedPrimitive<T>
    where T: Shape
{

}
