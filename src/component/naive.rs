// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A naively implemented linear aggregation of some other components

use geometry::prelude::*;
use super::Composable;
use std::sync::Arc;

pub struct Naive {
    elements: Vec<Arc<Composable>>,
    bbox: BBox3f,
}

impl Naive {
    pub fn new(elements: Vec<Arc<Composable>>) -> Naive {
        // assert!(elements.len() > 0);
        let mut bbox = elements[0].bbox_parent();
        for element in &elements {
            bbox = bbox.union(&element.bbox_parent());
        }
        Naive{
            elements: elements,
            bbox: bbox,
        }
    }

    pub fn from_one(element: Arc<Composable>) -> Naive {
        // assert!(elements.len() > 0);
        let bbox = element.bbox_parent();
        Naive{
            elements: vec![element],
            bbox: bbox,
        }
    }

    pub fn append(&mut self, element: Arc<Composable>) {
        let bbox = element.bbox_parent();
        self.bbox = self.bbox.union(&bbox);
        self.elements.push(element);
    }
}

impl Composable for Naive {
    fn bbox_parent(&self) -> BBox3f {
        self.bbox
    }

    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        let mut min_ray = ray.clone();
        let mut final_ret = None;
        for element in &self.elements {
            let mut ray = ray.clone();
            let ret = element.intersect_ray(&mut ray);
            if min_ray.max_extend() > ray.max_extend() { 
                min_ray = ray;
                final_ret = ret;
            }
        }
        ray.set_max_extend(min_ray.max_extend());
        final_ret
    }
}
