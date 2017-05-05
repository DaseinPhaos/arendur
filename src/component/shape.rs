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
use texturing::Texture;
use spectrum::*;
use lighting::{LightFlag, LightSample, LIGHT_AREA, SampleInfo, PathInfo};
use sample;

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

    #[inline]
    fn as_light(&self) -> &Light {
        self
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

    /// Given a position and an light direction in local coordinates,
    /// evaluate the light's radiance along that direction.
    #[inline]
    fn evaluate_path(&self, pos: Point3f, dir: Vector3f) -> RGBSpectrumf {
        if let Some(ref lp) = self.lighting_profile {
            let p = pos + dir;
            // match `wi` against surface normal
            let ray = RawRay::from_od(p, -dir);
            if let Some((_t, si)) = self.shape.intersect_ray(&ray) {
                // retrive (u, v)
                let dxy = DxyInfo::from_duv(&si.duv);
                return lp.evaluate(&si, &dxy);
            }
        }
        RGBSpectrumf::black()
    }

    /// Given a surface `pos` and `norm` in local frame with a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    fn evaluate_sampled(
        &self, pos: Point3f, sample: Point2f
    ) -> LightSample {
        let (l_pos, l_norm, l_pdf) = self.shape.sample_wrt(pos, sample);
        let mut ret = LightSample{
            radiance: RGBSpectrumf::black(),
            pdf: l_pdf,
            pfrom: l_pos,
            pto: pos,
        };
        // match against surface normal
        if let Some(ref lp) = self.lighting_profile {
            let ldir = pos - l_pos;
            if ldir.dot(l_norm) > 0. as Float {
                let ray = RawRay::from_od(pos, -ldir);
                if let Some((_, si)) = self.shape.intersect_ray(&ray) {
                    let dxy = DxyInfo::from_duv(&si.duv);
                    ret.radiance = lp.evaluate(&si, &dxy);
                }
            }
        }
        ret
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        let (pos, norm, pdfpos) = self.shape.sample(samples.pfilm);
        let (u, v) = normal::get_basis_from(norm);
        let dir = sample::sample_cosw_hemisphere(samples.plens);
        let dir = dir.x * u + dir.y * v + dir.z * norm;
        PathInfo{
            ray: RawRay::from_od(pos, dir),
            normal: norm,
            pdfpos: pdfpos,
            pdfdir: sample::pdf_cosw_hemisphere(dir.z),
            radiance: self.evaluate_path(pos, dir),
        }
    }

    #[inline]
    fn pdf_path(&self, pos: Point3f, dir: Vector3f, norm: Vector3f) -> (Float, Float) {
        (
            self.shape.pdf(pos, norm),
            sample::pdf_cosw_hemisphere(norm.dot(dir).abs())
        )
    }

    fn pdf(&self, pos: Point3f, wi: Vector3f) -> Float {
        self.shape.pdf_wrt(pos, wi)
    }

    /// returns an estimation of total power of this light
    fn power(&self) -> RGBSpectrumf {
        if let Some(ref lp) = self.lighting_profile {
            debug_assert!(self.shape.surface_area() >= 0. as Float);
            lp.mean() * self.shape.surface_area() * float::pi()
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
