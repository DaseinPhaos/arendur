// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A scene in the world.

use component::Composable;
use lighting::Light;
use std::sync::Arc;
use sample::prelude::*;
use sample;
use spectrum::{Spectrum, RGBSpectrumf};
use material::bsdf::Bsdf;
use bxdf::prelude::*;
use geometry::prelude::*;
use std::ptr;

/// A scene in the world
pub struct Scene {
    pub lights: Vec<Arc<Light>>,
    // pub area_lights: Vec<Arc<Composable>>,
    pub light_distribution: Distribution1D,
    pub aggregate: Arc<Composable>,
}

impl Scene {
    pub fn new(
        lights: Vec<Arc<Light>>, 
        // area_lights: Vec<Arc<Composable>>, 
        aggregate: Arc<Composable>
    ) -> Scene {
        // let mut func = Vec::with_capacity(lights.len() + area_lights.len());
        let mut func = Vec::with_capacity(lights.len());
        for light in &lights {
            func.push(light.power().to_xyz().y);
        }
        // for component in &area_lights {
        //     func.push(component.as_light().power().to_xyz().y);
        // }
        let light_distribution = Distribution1D::new(func);
        Scene{
            lights: lights,
            // area_lights: area_lights,
            light_distribution: light_distribution,
            aggregate: aggregate,
        }
    }

    #[inline]
    pub fn get_light(&self, idx: usize) -> &Light {
        self.lights[idx].as_ref()
    }

    pub fn uniform_sample_one_light<S: Sampler>(
        &self, si: &SurfaceInteraction, sampler: &mut S, bsdf: &Bsdf
    ) -> RGBSpectrumf {
        trace!("Sampling one light at {:?}", si);
        let (light, lightpdf) = self.sample_one_light(sampler.next());
        let ulight = sampler.next_2d();
        let uscattering = sampler.next_2d();
        self.evaluate_direct(light, ulight, uscattering, si, bsdf)/lightpdf
    }

    pub fn uniform_sample_all_lights<S: Sampler>(
        &self, si: &SurfaceInteraction, sampler: &mut S, bsdf: &Bsdf
    ) -> RGBSpectrumf {
        let mut ret = RGBSpectrumf::black();
        for light in self.lights.iter() {
            let ulight = sampler.next_2d();
            let uscattering = sampler.next_2d();
            let term = self.evaluate_direct(light.as_ref(), ulight, uscattering, si, bsdf);
            if term.valid() {
                ret += term;
            }
        }
        ret
    }

    fn evaluate_direct(&self,
        light: &Light, ulight: Point2f, uscattering: Point2f,
        si: &SurfaceInteraction, bsdf: &Bsdf
    ) -> RGBSpectrumf {
        trace!(
            "evaluating light {:p}, si {:p}, bsdf {:p}， ulight: {:?}, uscatter: {:?}", 
            light, si, bsdf, ulight, uscattering
        );
        let mut ret = RGBSpectrumf::black();
        let ls = light.evaluate_sampled(si.basic.pos, ulight);
        trace!("got light sample: {:?}", ls);
        let wi = ls.wi();
        if !ls.no_effect() {
            let mut f = bsdf.evaluate(
                si.basic.wo, wi, BXDF_ALL
            ).0 * wi.dot(si.shading_norm).abs();
            let spdf = bsdf.pdf(si.basic.wo, wi, BXDF_ALL);
            trace!("got bsdf value {:?}, pdf {:?}", f, spdf);
            if spdf == 0. as Float {
                f = RGBSpectrumf::black();
            }
            if !f.is_black() && ls.occluded(&*self.aggregate) {
                f = RGBSpectrumf::black();
                trace!("occluded");
            }
            if light.is_delta() {
                let addition = ls.radiance * f / ls.pdf;
                trace!(
                    "delta light, adding {:?} to return term", addition
                );
                if !addition.valid() {
                    warn!("invalid adding {:?} from light sampling", addition);
                }
                ret += addition;
            } else {
                let weight = sample::power_heuristic(1, ls.pdf, 1, spdf);
                let addition = ls.radiance * f * weight / ls.pdf;
                trace!("non delta light, sampling with MIS, with weight {}, adding {:?} to return term", weight, addition);
                if !addition.valid() {
                    warn!("invalid adding {:?} from light sampling", addition);
                }
                ret += addition;
            }
        }
        
        // sample BSDF with multiple importance sampling
        if !light.is_delta() {
            let (mut f, wi, pdf, bt) = bsdf.evaluate_sampled(
                si.basic.wo, uscattering, BXDF_ALL
            );
            f *= wi.dot(si.shading_norm).abs();
            trace!(
                "bsdf sampling result value: {:?}, wi {:?}, pdf {}, type {:?}",
                f, wi, pdf, bt
            );
            if !f.is_black() && pdf > 0. as Float {
                let mut weight = 1. as Float;
                if !bt.intersects(BXDF_SPECULAR) {
                    let lpdf = light.pdf(si.basic.pos, wi);
                    if lpdf == 0. as Float { return ret; }
                    weight = sample::power_heuristic(1, pdf, 1, lpdf);
                    trace!("non specular, MIS weight {}", weight);
                }
                let mut ray = si.spawn_ray_differential(wi, None);
                let mut li = RGBSpectrumf::black();
                if let Some(lsi) = self.aggregate.intersect_ray(&mut ray.ray) {
                    if let Some(primitive) = lsi.primitive_hit {
                        if ptr::eq(light, primitive.as_light()) {
                            li = lsi.le(-wi);
                            trace!("valid lighting term {:?}", li);
                        }
                    }
                }
                if !li.is_black() {
                    let addition = f * li * weight / pdf;
                    if !addition.valid() {
                        warn!("invalid adding {:?} from bsdf sampling", addition);
                    }
                    ret += addition;
                }
            }
        }
        ret
    }

    #[inline]
    pub fn sample_one_light(&self, u: Float) -> (&Light, Float) {
        let (idx, pdf, _) = self.light_distribution.sample_discrete(u);
        (self.get_light(idx), pdf)
    }
}