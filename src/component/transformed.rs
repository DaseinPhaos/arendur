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
use spectrum::*;
use renderer::scene::Scene;
use lighting::{LightFlag, LightSample, SampleInfo, PathInfo};

/// Component transformed from another component
#[derive(Clone, Debug)]
pub struct TransformedComposable<T> {
    inner: T,
    local_parent: Arc<Matrix4f>,
    parent_local: Arc<Matrix4f>,
}

impl<T> TransformedComposable<T> {
    pub fn new(inner: T, local_parent: Arc<Matrix4f>, parent_local: Arc<Matrix4f>) -> Self
    {
        #[cfg(debug)]
        {
            assert_relative_eq(*local_parent *(*parent_local), Matrix4f::identity());
        }
        TransformedComposable{
            inner: inner,
            local_parent: local_parent,
            parent_local: parent_local,
        }
    }
}

impl<T: Composable> Composable for TransformedComposable<T>
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.inner.bbox_parent().apply_transform(&*self.local_parent)
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        1.0 as Float + self.inner.intersection_cost()
    }

    #[inline]
    default fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.inner.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            *ret = ret.apply_transform(&*self.local_parent);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }

    #[inline]
    default fn as_light(&self) -> &Light {
        unimplemented!();
    }
}

impl<T: Primitive> Composable for TransformedComposable<T>
{
    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.inner.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            *ret = ret.apply_transform(&*self.local_parent);
            ret.primitive_hit = Some(self);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }

    #[inline]
    fn as_light(&self) -> &Light {
        self
    }
}

impl<T: Primitive> Primitive for TransformedComposable<T>
{
    #[inline]
    fn is_emissive(&self) -> bool {
        self.inner.is_emissive()
    }

    #[inline]
    fn get_material(&self) -> &Material {
        self.inner.get_material()
    }
}

impl<T: Primitive> Light for TransformedComposable<T>
{
    fn flags(&self) -> LightFlag {
        self.inner.flags()
    }

    #[inline]
    fn evaluate_ray(&self, rd: &RayDifferential) -> RGBSpectrumf {
        let rd = rd.apply_transform(&self.parent_local);
        self.inner.evaluate_ray(&rd)
    }

    #[inline]
    fn evaluate_path(&self, pos: Point3f, dir: Vector3f) -> RGBSpectrumf {
        let pos = self.parent_local.transform_point(pos);
        let dir = self.parent_local.transform_vector(dir);
        self.inner.evaluate_path(pos, dir)
    }

    #[inline]
    fn evaluate_sampled(&self, pos: Point3f, sample: Point2f) -> LightSample {
        let pos = self.parent_local.transform_point(pos);
        let ls = self.inner.evaluate_sampled(pos, sample);
        ls.apply_transform(&*self.local_parent)
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        self.inner.generate_path(samples).apply_transform(&*self.local_parent)
    }

    #[inline]
    fn pdf_path(&self, pos: Point3f, dir: Vector3f, norm: Vector3f) -> (Float, Float) {
        let pos = self.parent_local.transform_point(pos);
        let dir = self.parent_local.transform_vector(dir);
        let norm = self.parent_local.transform_norm(norm);
        self.inner.pdf_path(pos, dir, norm)
    }

    #[inline]
    fn pdf(&self, pos: Point3f, wi: Vector3f) -> Float {
        let pos = self.parent_local.transform_point(pos);
        let wi = self.parent_local.transform_vector(wi);
        self.inner.pdf(pos, wi)
    }

    #[inline]
    fn power(&self) -> RGBSpectrumf {
        self.inner.power()
    }

    #[inline]
    fn preprocess(&mut self, s: &Scene) {
        self.inner.preprocess(s);
    }
}

impl<T: Composable> Composable for TransformedComposable<Arc<T>>
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.inner.bbox_parent().apply_transform(&*self.local_parent)
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        2.0 as Float + self.inner.intersection_cost()
    }

    #[inline]
    default fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.inner.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            *ret = ret.apply_transform(&*self.local_parent);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }

    #[inline]
    default fn as_light(&self) -> &Light {
        unimplemented!();
    }
}

impl<T: Primitive> Composable for TransformedComposable<Arc<T>>
{
    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.inner.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            *ret = ret.apply_transform(&*self.local_parent);
            ret.primitive_hit = Some(self);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }

    #[inline]
    fn as_light(&self) -> &Light {
        self
    }
}

impl<T: Primitive> Primitive for TransformedComposable<Arc<T>>
{
    #[inline]
    fn is_emissive(&self) -> bool {
        self.inner.is_emissive()
    }

    #[inline]
    fn get_material(&self) -> &Material {
        self.inner.get_material()
    }
}

impl<T: Primitive> Light for TransformedComposable<Arc<T>>
{
    fn flags(&self) -> LightFlag {
        self.inner.flags()
    }

    #[inline]
    fn evaluate_ray(&self, rd: &RayDifferential) -> RGBSpectrumf {
        let rd = rd.apply_transform(&self.parent_local);
        self.inner.evaluate_ray(&rd)
    }

    #[inline]
    fn evaluate_path(&self, pos: Point3f, dir: Vector3f) -> RGBSpectrumf {
        let pos = self.parent_local.transform_point(pos);
        let dir = self.parent_local.transform_vector(dir);
        self.inner.evaluate_path(pos, dir)
    }

    #[inline]
    fn evaluate_sampled(&self, pos: Point3f, sample: Point2f) -> LightSample {
        let pos = self.parent_local.transform_point(pos);
        let ls = self.inner.evaluate_sampled(pos, sample);
        ls.apply_transform(&*self.local_parent)
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        self.inner.generate_path(samples).apply_transform(&*self.local_parent)
    }

    #[inline]
    fn pdf_path(&self, pos: Point3f, dir: Vector3f, norm: Vector3f) -> (Float, Float) {
        let pos = self.parent_local.transform_point(pos);
        let dir = self.parent_local.transform_vector(dir);
        let norm = self.parent_local.transform_norm(norm);
        self.inner.pdf_path(pos, dir, norm)
    }

    #[inline]
    fn pdf(&self, pos: Point3f, wi: Vector3f) -> Float {
        let pos = self.parent_local.transform_point(pos);
        let wi = self.parent_local.transform_vector(wi);
        self.inner.pdf(pos, wi)
    }

    #[inline]
    fn power(&self) -> RGBSpectrumf {
        self.inner.power()
    }
}

impl Composable for TransformedComposable<Arc<Composable>>
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.inner.bbox_parent().apply_transform(&*self.local_parent)
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        2.0 as Float + self.inner.intersection_cost()
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.inner.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            *ret = ret.apply_transform(&*self.local_parent);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }

    #[inline]
    fn as_light(&self) -> &Light {
        unimplemented!();
    }
}

impl Composable for TransformedComposable<Arc<Primitive>>
{
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.inner.bbox_parent().apply_transform(&*self.local_parent)
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        2.0 as Float + self.inner.intersection_cost()
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        *ray = ray.apply_transform(&*self.parent_local);
        let mut ret = self.inner.intersect_ray(ray);
        if let Some(ret) = ret.as_mut() {
            *ret = ret.apply_transform(&*self.local_parent);
            ret.primitive_hit = Some(self);
        }
        *ray = ray.apply_transform(&*self.local_parent);
        ret
    }

    #[inline]
    fn as_light(&self) -> &Light {
        self
    }
}

impl Primitive for TransformedComposable<Arc<Primitive>>
{
    #[inline]
    fn is_emissive(&self) -> bool {
        self.inner.is_emissive()
    }

    #[inline]
    fn get_material(&self) -> &Material {
        self.inner.get_material()
    }
}

impl Light for TransformedComposable<Arc<Primitive>>
{
    fn flags(&self) -> LightFlag {
        self.inner.flags()
    }

    #[inline]
    fn evaluate_ray(&self, rd: &RayDifferential) -> RGBSpectrumf {
        let rd = rd.apply_transform(&self.parent_local);
        self.inner.evaluate_ray(&rd)
    }

    #[inline]
    fn evaluate_path(&self, pos: Point3f, dir: Vector3f) -> RGBSpectrumf {
        let pos = self.parent_local.transform_point(pos);
        let dir = self.parent_local.transform_vector(dir);
        self.inner.evaluate_path(pos, dir)
    }

    #[inline]
    fn evaluate_sampled(&self, pos: Point3f, sample: Point2f) -> LightSample {
        let pos = self.parent_local.transform_point(pos);
        let ls = self.inner.evaluate_sampled(pos, sample);
        ls.apply_transform(&*self.local_parent)
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        self.inner.generate_path(samples).apply_transform(&*self.local_parent)
    }

    #[inline]
    fn pdf_path(&self, pos: Point3f, dir: Vector3f, norm: Vector3f) -> (Float, Float) {
        let pos = self.parent_local.transform_point(pos);
        let dir = self.parent_local.transform_vector(dir);
        let norm = self.parent_local.transform_norm(norm);
        self.inner.pdf_path(pos, dir, norm)
    }

    #[inline]
    fn pdf(&self, pos: Point3f, wi: Vector3f) -> Float {
        let pos = self.parent_local.transform_point(pos);
        let wi = self.parent_local.transform_vector(wi);
        self.inner.pdf(pos, wi)
    }

    #[inline]
    fn power(&self) -> RGBSpectrumf {
        self.inner.power()
    }
}