//! A naively built linear aggregation of some other components

use geometry::*;
use super::{Aggregate, Primitive, Composable};
use std::rc::Rc;

pub struct Naive {
    elements: Vec<Rc<Composable>>,
    bbox: BBox3f,
}

impl Composable for Naive {
    fn bbox_parent(&self) -> BBox3f {
        self.bbox
    }

    fn intersect_ray(&self, ray: &mut RawRay) -> Option<(SurfaceInteraction, &Primitive)> {
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

impl Aggregate for Naive { }