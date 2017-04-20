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
use texturing::Texture;
use spectrum::*;
use lighting::{LightFlag, LightSample, LIGHT_AREA};

/// Represents a primitive made up by a single `Shape`
#[derive(Clone)]
pub struct ShapedPrimitive<S, M> {
    pub shape: S,
    pub material: M,
    pub lighting_profile: Option<Arc<Texture<Texel=RGBSpectrumf>>>,
    // TODO: medium:
}

impl<S, M> ShapedPrimitive<S, M>
    where S: Shape, M: Material
{
    /// construction
    #[inline]
    pub fn new(
        shape: S, material: M, 
        lighting_profile: Option<Arc<Texture<Texel=RGBSpectrumf>>>
    ) -> ShapedPrimitive<S, M> {
        ShapedPrimitive{
            shape: shape, material: material, lighting_profile: lighting_profile,
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

    #[inline]
    fn can_intersect(&self, ray: &RawRay) -> bool {
        self.shape.can_intersect(ray)
    }
}

impl<S, M> Light for ShapedPrimitive<S, M>
    where S: Shape, M: Material
{
    #[inline]
    fn flags(&self) -> LightFlag {
        LIGHT_AREA
    }

    /// test if the light has delta distribution
    #[inline]
    fn is_delta(&self) -> bool {
        false
    }

    /// Given a position and an incoming direction in local coordinates,
    /// evaluate the light's radiance along that direction.
    #[inline]
    fn evaluate(&self, pos: Point3f, wi: Vector3f) -> RGBSpectrumf {
        if let Some(ref lp) = self.lighting_profile {
            // match `wi` against surface normal
            let ray = RawRay::from_od(pos, wi);
            if let Some((_t, si)) = self.shape.intersect_ray(&ray) {
                // retrive (u, v)
                let dxy = DxyInfo::from_duv(&si.duv);
                return lp.evaluate(&si, &dxy);
            }
        }
        RGBSpectrumf::black()
    }

    /// Given a `pos` in local frame with a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    fn evaluate_sampled(&self, pos: Point3f, sample: Point2f) -> LightSample {
        let (sampled, norm) = self.shape.sample(sample);
        let mut ret = LightSample{
            radiance: RGBSpectrumf::black(),
            pdf: 0. as Float,
            pfrom: sampled,
            pto: pos,
        };
        // match against surface normal
        if let Some(ref lp) = self.lighting_profile {
            if (pos - sampled).dot(norm) > 0. as Float {
                let ray = RawRay::from_od(pos, -norm);
                // debug_assert!(self.shape.can_intersect(&ray));
                if let Some((_t, si)) = self.shape.intersect_ray(&ray) {
                    // print!("aha! ");
                    let dxy = DxyInfo::from_duv(&si.duv);
                    ret.radiance = lp.evaluate(&si, &dxy);
                    ret.pdf = self.shape.pdf(sampled, norm);
                }
            }
        }
        ret
    }

    /// returns an estimation of total power of this light
    fn power(&self) -> RGBSpectrumf {
        if self.lighting_profile.is_some() {
            // FIXME: wrong
            RGBSpectrumf::new(0.5 as Float, 0.5 as Float, 0.5 as Float) * self.shape.surface_area()
        } else {
            RGBSpectrumf::black()
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
    fn is_emissive(&self) -> bool {
        self.lighting_profile.is_some()
    }

    // #[inline]
    // fn get_area_light(&self) -> Option<&Light> {
    //     if let Some(ref al) = self.area_light {
    //         Some(al.as_ref())
    //     } else {
    //         None
    //     }
    // }
}
