use super::cgmath_prelude::*;
use super::bbox::BBox3;

pub trait TransformExt: Transform<Point3f> {
    fn transform_ray<R>(&self, ray: &R) -> R
        where R: super::Ray {
            ray.apply_transform(self)
    }

    fn transform_bbox(&self, bbox: &BBox3<Float>) -> BBox3<Float> {
        bbox.apply_transform(self)
    }
}