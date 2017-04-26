// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A path tracing renderer

use bxdf::prelude::*;
use sample::prelude::*;
use filming::prelude::*;
use filming::film::FilmTile;
use super::Renderer;
use std::sync::Arc;
use super::scene::Scene;
use spectrum::{RGBSpectrumf, Spectrum};
use rayon::prelude::*;
use copy_arena::{Allocator, Arena};
use geometry::prelude::*;
use std::path::{PathBuf, Path};

/// A path tracing renderer
pub struct PTRenderer<S> {
    sampler: S,
    camera: Arc<Camera>,
    filename: PathBuf,
    max_depth: usize,
}

impl<S: Sampler> PTRenderer<S> {
    pub fn new<P: AsRef<Path> + ?Sized>(
        sampler: S, camera: Arc<Camera>, filename: &P, max_depth: usize
    ) -> PTRenderer<S> {
        PTRenderer{
            sampler: sampler,
            camera: camera,
            filename: filename.as_ref().to_path_buf(),
            max_depth: max_depth,
        }
    }
}


// helper function for whitted rendering's light computation
fn calculate_lighting<S: Sampler>(
    mut ray: RayDifferential, 
    scene: &Scene, 
    sampler: &mut S, 
    alloc: &mut Allocator, 
    depth: usize,
    max_depth: usize
) -> RGBSpectrumf {
    let mut ret = RGBSpectrumf::black();
    if depth > max_depth { return ret; }
    let mut beta = RGBSpectrumf::new(1. as Float, 1. as Float, 1. as Float);
    let mut specular_bounce = false;
    let mut bounces = 0;
    loop {
        if let Some(mut si) = scene.aggregate.intersect_ray(&mut ray.ray) {
            if bounces == 0 || specular_bounce {
                ret += beta * si.le(-ray.ray.direction());
            }
            if let Some(primitive) = si.primitive_hit {
                let dxy = si.compute_dxy(&ray);
                let bsdf = primitive.get_material().compute_scattering(
                    &mut si, &dxy, alloc
                );
                // sample illumination, skip perfect specular
                let mut tags = BXDF_ALL;
                tags.remove(BXDF_SPECULAR);
                if bsdf.have_n(tags) > 0 {
                    let ld = beta * scene.uniform_sample_one_light(&si, sampler, &bsdf);
                    ret += ld;
                }
                // sample bsdf to get new path direction
                let wo = -ray.ray.direction();
                let (f, wi, pdf, bt) = bsdf.evaluate_sampled(wo, sampler.next_2d(), BXDF_ALL);
                specular_bounce = bt.intersects(BXDF_SPECULAR);
                if f.is_black() || pdf == 0. as Float { break; }
                beta *= f * (wi.dot(si.shading_norm).abs() / pdf);
                debug_assert!(beta.inner.y >= 0. as Float);
                ray = si.spawn_ray_differential(wi, Some(&dxy));

            } else {
                // TODO: handle media boundary
                break;
            }
        } else {
            // TODO: infinite area lighting
            break;
        }

        // TODO: terminate the path with Russian roulette
        bounces += 1;
        if bounces >= max_depth { break; }
    }
    ret
}

impl<S: Sampler> Renderer for PTRenderer<S> {
    fn render(&mut self, scene: &Scene) {
        let mut tiles: Vec<FilmTile<RGBSpectrumf>> = self.camera.get_film().spawn_tiles(16, 16);
        // tiles.par_iter_mut().for_each(|tile| {
        for tile in &mut tiles {
            let mut arena = Arena::new();
            let mut allocator = arena.allocator();
            let mut sampler = self.sampler.clone();
            let tile_bound = tile.bounding();
            for p in tile_bound {
                let p: Point2<u32> = p.cast();
                sampler.start_pixel(p);
                loop {
                    let camera_sample_info = sampler.get_camera_sample(p);
                    let mut ray_differential = self.camera.generate_path_differential(camera_sample_info);
                    ray_differential.scale_differentials(1.0 as Float / sampler.sample_per_pixel() as Float);
                    let total_randiance = calculate_lighting(ray_differential, scene, &mut sampler, &mut allocator, 0, self.max_depth);
                    tile.add_sample(camera_sample_info.pfilm, &total_randiance);
                    
                    if !sampler.next_sample() { break; }
                }
            }
        // });
        }
        let render_result = self.camera.get_film().collect_into(tiles);
        render_result.save(self.filename.clone()).expect("saving failure");
    }
}