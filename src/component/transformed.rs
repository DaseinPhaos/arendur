// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Component transformed from another component

use geometry::prelude::*;
use super::*;
use std::rc::Rc;

/// Component transformed from another component
pub struct TransformedComposable<'a> {
    original: Rc<Composable>,
    local_parent: &'a Matrix4f,
    parent_local: &'a Matrix4f,
}

impl<'a> Composable for TransformedComposable<'a>
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.original.bbox_parent().apply_transform(self.local_parent)
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<(SurfaceInteraction, &Primitive)> {
        ray.apply_transform(self.parent_local);
        let mut ret = self.original.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            ret.0.apply_transform(self.local_parent);
        }
        ray.apply_transform(self.local_parent);
        ret
    }
}
