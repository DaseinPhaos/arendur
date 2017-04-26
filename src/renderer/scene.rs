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

/// A scene in the world
pub struct Scene {
    pub lights: Vec<Arc<Light>>,
    pub area_lights: Vec<Arc<Composable>>,
    pub light_distribution: Distribution1D,
    pub aggregate: Arc<Composable>,
}

impl Scene {
    pub fn new(
        lights: Vec<Arc<Light>>, 
        area_lights: Vec<Arc<Composable>>, 
        aggregate: Arc<Composable>
    ) -> Scene {
        let mut func = Vec::with_capacity(lights.len() + area_lights.len());
        for light in &lights {
            func.push(light.power().to_xyz().y);
        }
        for component in &area_lights {
            func.push(component.as_light().power().to_xyz().y);
        }
        let light_distribution = Distribution1D::new(func);
        Scene{
            lights: lights,
            area_lights: area_lights,
            light_distribution: light_distribution,
            aggregate: aggregate,
        }
    }

    #[inline]
    pub fn get_light(&self, idx: usize) -> &Light {
        if idx < self.lights.len() {
            self.lights[idx].as_ref()
        } else {
            self.area_lights[idx-self.lights.len()].as_light()
        }
    }

    pub fn uniform_sample_one_light<S: Sampler>(&self, si: &SurfaceInteraction, sampler: &mut S, bsdf: &Bsdf) -> RGBSpectrumf {
        let (light, lightpdf) = self.sample_one_light(sampler.next());
        let ulight = sampler.next_2d();
        let uscattering = sampler.next_2d();
        let mut ret = RGBSpectrumf::black();
        let ls = light.evaluate_sampled(si.basic.pos, ulight);
        let wi = ls.wi();
        if !ls.no_effect() {
            let mut f = bsdf.evaluate(si.basic.wo, wi, BXDF_ALL).0 * wi.dot(si.shading_norm).abs();
            let spdf = bsdf.pdf(si.basic.wo, wi, BXDF_ALL);
            if !f.is_black() && ls.occluded(&*self.aggregate) {
                f = RGBSpectrumf::black();
            }
            if light.is_delta() {
                ret += ls.radiance * f / ls.pdf;
            } else {
                let weight = sample::power_heuristic(1, ls.pdf, 1, spdf);
                ret += ls.radiance * f * weight / ls.pdf;
            }
        }
        
        // // TODO: sample BSDF with multiple importance sampling
        // if !light.is_delta() {
        //     let (mut f, wi, pdf, bt) = bsdf.evaluate_sampled(
        //         si.basic.wo, uscattering, BXDF_ALL
        //     );
            
        //     f *= wi.dot(si.shading_norm).abs();
        //     if !f.is_black() && pdf > 0. as Float {
        //         let mut weight = 1. as Float;
        //         if !bt.intersects(BXDF_SPECULAR) {
        //             let lpdf = light.pdf(si.basic.pos, wi, si.shading_norm).1;
        //             if lpdf == 0. as Float { return ret; }
        //             weight = sample::power_heuristic(1, pdf, 1, lpdf);
        //         }

        //     }
        // }
        ret
    }

    #[inline]
    pub fn sample_one_light(&self, u: Float) -> (&Light, Float) {
        let (idx, pdf, _) = self.light_distribution.sample_discrete(u);
        (self.get_light(idx), pdf)
    }
}