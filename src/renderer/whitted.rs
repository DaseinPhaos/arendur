// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines whitted renderer

use bxdf::*;
use sample::Sampler;
use filming::Camera;
use super::Renderer;
use std::sync::Arc;
use super::scene::Scene;
use filming::film::FilmTile;
use spectrum::{RGBSpectrumf, Spectrum};
use rayon::prelude::*;
use aren_alloc::Allocator;
use geometry::prelude::*;
use std::path::{PathBuf, Path};

/// whitted renderer
pub struct WhittedRenderer<S> {
    sampler: S,
    camera: Arc<Camera>,
    path: PathBuf,
}

impl<S: Sampler> WhittedRenderer<S> {
    pub fn new<P: AsRef<Path> + ?Sized>(sampler: S, camera: Arc<Camera>, path: &P) -> WhittedRenderer<S> {
        WhittedRenderer{
            sampler: sampler,
            camera: camera,
            path: path.as_ref().to_path_buf(),
        }
    }
}

// helper function for whitted rendering's light computation
fn calculate_lighting<S: Sampler>(
    mut ray: RayDifferential, 
    scene: &Scene, 
    sampler: &mut S, 
    alloc: &Allocator, 
    depth: usize
) -> RGBSpectrumf {
    let mut ret = RGBSpectrumf::black();
    if depth > 5 { return ret; }
    if let Some(mut surinter) = scene.aggregate.intersect_ray(&mut ray.ray) {
        let pos = surinter.basic.pos;
        let norm = surinter.shading_norm;
        let wo = surinter.basic.wo;
        let dxy = surinter.compute_dxy(&ray);
        if let Some(primitive) = surinter.primitive_hit {
            if primitive.is_emissive() {
                ret += primitive.evaluate_ray(&ray);
                // let rad = primitive.evaluate_ray(&ray);
                // print!("emission found: {:?}", rad);
            }
            let bsdf = primitive.get_material().compute_scattering(&mut surinter, &dxy, alloc);
            for light in &scene.lights {
                let lightsample = light.evaluate_sampled(pos, sampler.next_2d());
                if lightsample.no_effect() { continue; }
                let wi = lightsample.wi();
                let (bsdfv, _) = bsdf.evaluate(wo, wi, BXDF_ALL);
                if bsdfv != RGBSpectrumf::black() && !lightsample.occluded(&*scene.aggregate) {
                    let coontribution = bsdfv * lightsample.radiance * wi.dot(norm) / lightsample.pdf;
                    ret += coontribution;
                    // TODO: specular reflect, specular transmit
                }
            }
        }
    } else {
        for light in &scene.lights {
            ret += light.evaluate_ray(&ray);
        }
    }
    ret
}

impl<S: Sampler> Renderer for WhittedRenderer<S> {
    fn render(&mut self, scene: &Scene) {
        let mut tiles: Vec<FilmTile<RGBSpectrumf>> = self.camera.get_film().spawn_tiles(16, 16);
        
        // let mut rc = 0;
        // let mut tc = 0;
        tiles.par_iter_mut().for_each(|tile| {
        // for tile in &mut tiles {
            // let mut arena = Arena::new();
            let allocator = Allocator::new();
            let mut sampler = self.sampler.clone();
            let tile_bound = tile.bounding();
            for p in tile_bound {
                let p: Point2<u32> = p.cast();
                sampler.start_pixel(p);
                loop {
                    let camera_sample_info = sampler.get_camera_sample(p);
                    let mut ray_differential = self.camera.generate_path_differential(camera_sample_info);
                    ray_differential.scale_differentials(1.0 as Float / sampler.sample_per_pixel() as Float);
                    let total_randiance = calculate_lighting(ray_differential, scene, &mut sampler, &allocator, 0);
                    // if total_randiance != RGBSpectrumf::black() { rc += 1; }
                    // tc += 1;
                    tile.add_sample(camera_sample_info.pfilm, &total_randiance);
                    
                    if !sampler.next_sample() { break; }
                }
            }
        });
        // }
        let render_result = self.camera.get_film().collect_into(tiles);
        render_result.save(&self.path).expect("saving failure");
    }
}