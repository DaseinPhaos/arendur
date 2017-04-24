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
use spectrum::Spectrum;

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
}