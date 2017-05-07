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
profile_use!();

/// A path tracing renderer
pub struct PTRenderer<S> {
    sampler: S,
    camera: Arc<Camera>,
    filename: PathBuf,
    max_depth: usize,
    multithreaded: bool,
    rr_threshold: Float,
    min_depth: usize,
}

impl<S: Sampler> PTRenderer<S> {
    pub fn new<P: AsRef<Path> + ?Sized>(
        sampler: S, camera: Arc<Camera>, 
        filename: &P, max_depth: usize, multithreaded: bool
    ) -> PTRenderer<S> {
        PTRenderer{
            sampler: sampler,
            camera: camera,
            filename: filename.as_ref().to_path_buf(),
            max_depth: max_depth,
            multithreaded: multithreaded,
            rr_threshold: 0.05 as Float,
            min_depth: max_depth/2,
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
    max_depth: usize,
    min_depth: usize,
    rr_threshold: Float
) -> RGBSpectrumf {
    let mut ret = RGBSpectrumf::black();
    if depth > max_depth { return ret; }
    let mut beta = RGBSpectrumf::new(1. as Float, 1. as Float, 1. as Float);
    let mut specular_bounce = false;
    let mut bounces = 0;
    loop {
        if let Some(mut si) = scene.aggregate.intersect_ray(&mut ray.ray) {
            if bounces == 0 || specular_bounce {
                let term = si.le(-ray.ray.direction());
                ret += beta * term;
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
                    // let term = scene.uniform_sample_all_lights(&si, sampler, &bsdf);
                    let term = scene.uniform_sample_one_light(&si, sampler, &bsdf);
                    ret += beta * term;
                }
                // sample bsdf to get new path direction
                let wo = -(ray.ray.direction());
                let (f, wi, pdf, bt) = bsdf.evaluate_sampled(wo, sampler.next_2d(), BXDF_ALL);
                specular_bounce = bt.intersects(BXDF_SPECULAR);
                if f.is_black() || pdf == 0. as Float { break; }
                beta *= f * (wi.dot(si.shading_norm).abs() / pdf);
                if !beta.valid() {
                    break;
                }
                assert!(beta.inner.y >= 0. as Float);
                ray = si.spawn_ray_differential(wi, Some(&dxy));

            } else {
                // TODO: handle media boundary
                break;
            }
        } else {
            // TODO: infinite area lighting
            break;
        }

        bounces += 1;
        if bounces >= max_depth { break; }

        // possibly terminates the path with russian roulette threshold
        if beta.to_xyz().y < rr_threshold && bounces >= min_depth {
            let q = rr_threshold.max(0.05 as Float);
            if sampler.next() < q { break; }
            beta /= 1.0 as Float - q;
        }
    }
    ret
}

impl<S: Sampler> Renderer for PTRenderer<S> {
    fn render(&mut self, scene: &Scene) {
        profile_start!("pt rendering");
        let mut tiles: Vec<FilmTile<RGBSpectrumf>> = self.camera.get_film().spawn_tiles(16, 16);
        let render_tile = |tile: &mut FilmTile<_>| {
            let mut arena = Arena::new();
            let mut sampler = self.sampler.clone();
            let tile_bound = tile.bounding();
            for p in tile_bound {
                let p: Point2<u32> = p.cast();
                sampler.start_pixel(p);
                loop {
                    let mut allocator = arena.allocator();
                    let camera_sample_info = sampler.get_camera_sample(p);
                    let mut ray_differential = self.camera.generate_path_differential(camera_sample_info);
                    ray_differential.scale_differentials(1.0 as Float / sampler.sample_per_pixel() as Float);
                    profile_start!("pt light calculation");
                    let total_randiance = calculate_lighting(
                        ray_differential, scene, &mut sampler, 
                        &mut allocator, 0, self.max_depth,
                        self.min_depth, self.rr_threshold
                    );
                    profile_end!("pt light calculation");

                    profile_start!("pt add sample");
                    if total_randiance.valid() {
                        tile.add_sample(camera_sample_info.pfilm, &total_randiance);
                    } else {
                        tile.add_sample(camera_sample_info.pfilm, &RGBSpectrumf::black());
                    }
                    profile_end!("pt add sample");
                    if !sampler.next_sample() { break; }
                }
            }
            // println!("tile {:?} done!", tile_bound);
        };
        if self.multithreaded {
            tiles.par_iter_mut().for_each(|tile| render_tile(tile));
        } else {
            for tile in &mut tiles { render_tile(tile); }
        }
        let render_result = self.camera.get_film().collect_into(tiles);
        profile_end!("pt rendering");
        render_result.save(&self.filename).expect("saving failure");
        profile_dump!("pt rendering results.html");
    }
}