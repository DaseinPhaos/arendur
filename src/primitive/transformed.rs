use geometry::*;
use super::*;
use std::rc::Rc;
use shape::*;

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