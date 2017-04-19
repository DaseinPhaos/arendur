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
use std::sync::Arc;

/// Component transformed from another component
pub struct TransformedComposable<T> {
    original: T,
    local_parent: Arc<Matrix4f>,
    parent_local: Arc<Matrix4f>,
}

impl<T: Composable> TransformedComposable<T> {
    pub fn new(original: T, local_parent: Arc<Matrix4f>, parent_local: Arc<Matrix4f>) -> Self
    {
        #[cfg(debug)]
        {
            assert_relative_eq(*local_parent *(*parent_local), Matrix4f::identity());
        }
        TransformedComposable{
            original: original,
            local_parent: local_parent,
            parent_local: parent_local,
        }
    }
}

impl<T: Composable> Composable for TransformedComposable<T>
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.original.bbox_parent().apply_transform(&*self.local_parent)
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.original.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            ret.apply_transform(&*self.local_parent);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }
}

impl<T: Primitive> Primitive for TransformedComposable<T>
{
    #[inline]
    fn get_area_light(&self) -> Option<&Light> {
        self.original.get_area_light()
    }

    #[inline]
    fn get_material(&self) -> &Material {
        self.original.get_material()
    }
}
