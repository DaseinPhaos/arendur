// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines whitted renderer

use sample::Sampler;
use filming::Camera;
use super::Renderer;
use std::sync::Arc;
use super::scene::Scene;
use filming::film::FilmTile;
use spectrum::RGBSpectrumf;
use rayon::prelude::*;
use copy_arena::{Allocator, Arena};
use sample::strata::StrataSampler;
use geometry::prelude::*;
use std::path::PathBuf;

/// whitted renderer
pub struct WhittedRenderer<S> {
    sampler: S,
    camera: Arc<Camera>,
    path: PathBuf,
}

fn calculate_lighting<S: Sampler>(ray: RayDifferential, scene: &Scene, sampler: &mut S, alloc: &mut Allocator, depth: usize) -> RGBSpectrumf {
    unimplemented!();
}

impl<S: Sampler> Renderer for WhittedRenderer<S> {
    fn render(&mut self, scene: &Scene) {
        let mut tiles: Vec<FilmTile<RGBSpectrumf>> = self.camera.get_film().spawn_tiles(16, 16);
        let tn = tiles.len();
        tiles.par_iter_mut().for_each(|tile| {
            let mut arena = Arena::new();
            let mut allocator = arena.allocator();
            let mut sampler = self.sampler.clone();
            let tile_bound = tile.bounding();
            for p in tile_bound {
                let p: Point2<u32> = p.cast();
                sampler.start_pixel(p);
                let camera_sample_info = sampler.get_camera_sample(p);
                let ray_differential = self.camera.generate_ray_differential(camera_sample_info);
                let total_randiance = calculate_lighting(ray_differential, scene, &mut sampler, &mut allocator, 0);
                tile.add_sample(camera_sample_info.pfilm, &total_randiance);
            }
        });
        let render_result = self.camera.get_film().collect_into(tiles);
        render_result.save(self.path.clone()).expect("saving failure");
    }
}