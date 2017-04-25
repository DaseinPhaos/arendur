// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A bidirectional path tracing renderer

use bxdf::*;
use sample::Sampler;
use filming::Camera;
use super::Renderer;
use std::sync::Arc;
use super::scene::Scene;
use filming::film::FilmTile;
use spectrum::{RGBSpectrumf, Spectrum};
use rayon::prelude::*;
use copy_arena::{Allocator, Arena};
use geometry::prelude::*;
use std::path::{PathBuf, Path};
use self::node::Node;
use filming::SampleInfo;

/// A bidirectional path tracing renderer
pub struct BPTRenderer<S> {
    sampler: S,
    camera: Arc<Camera>,
    path_len: usize,
    path: PathBuf,
    max_depth: usize,
}

impl<S: Sampler> Renderer for BPTRenderer<S> {
    fn render(&mut self, scene: &Scene) {
        let mut tiles: Vec<FilmTile<RGBSpectrumf>> = self.camera.get_film().spawn_flat_tiles(16, 16);
        tiles.par_iter_mut().for_each(|tile| {
            let mut arena = Arena::new();
            let mut sampler = self.sampler.clone();
            let tile_bound = tile.bounding();
            for p in tile_bound {
                let p: Point2<u32> = p.cast();
                sampler.start_pixel(p);
                loop {
                    let mut allocator = arena.allocator();
                    let pfilm = sampler.next_2d() + p.to_vec().cast();
                    // let pfilm = p.cast() + caminfo.pfilm;
                    let mut cam_nodes = allocator.alloc_slice_default(self.max_depth + 2);
                    let mut light_nodes = allocator.alloc_slice_default(self.max_depth + 1);
                    // TODO: generate subpaths
                    let ncam = generate_camera_subpath(
                        scene, &mut sampler, &mut allocator, &*self.camera, pfilm, cam_nodes
                    );
                    let nlight = generate_light_subpath(
                        scene, &mut sampler, &mut allocator, light_nodes
                    );
                    let mut l = RGBSpectrumf::black();
                    for t in 0..ncam {
                        for s in 0..nlight {
                            let depth = t as isize + s as isize - 2isize;
                            if (s==1 && t==1) || depth < 0 || depth>self.max_depth as isize {
                                continue;
                            }
                            let mut pfilm_new = pfilm;
                            let mis_weight = 0. as Float;
                            // TODO: connect BPT
                            let lpath = RGBSpectrumf::black();
                            if t!=1 {l+=lpath;}
                            else {tile.add_sample(pfilm_new, &lpath)};
                        }
                    }
                    tile.add_sample(pfilm, &l);
                    if !sampler.next_sample() { break; }
                }
            }
        })
    }
}

fn generate_camera_subpath<'a, S: Sampler>(
    scene: &'a Scene, sampler: &mut S, 
    allocator: &mut Allocator<'a>,
    camera: &'a Camera, pfilm: Point2f, path: &mut [Node<'a>]
) -> usize {
    if path.len() == 0 { return 0; }
    let plens = sampler.next_2d();
    let sampleinfo = SampleInfo{
        pfilm: pfilm, plens: plens,
    };
    let mut ray_differential = camera.generate_path_differential(sampleinfo);
    ray_differential.scale_differentials(1.0 as Float / sampler.sample_per_pixel() as Float);
    // TODO: double check ray direction
    let (pdfpos, pdfdir) = camera.pdf(
        ray_differential.ray.origin(), ray_differential.ray.direction()
    );
    let beta = RGBSpectrumf::new(1. as Float, 1. as Float, 1. as Float);
    path[0] = Node::Camera{
        camera: camera,
        info: InteractInfo{
            pos: ray_differential.ray.origin(),
            wo: ray_differential.ray.direction(),
            norm: ray_differential.ray.direction(),
        },
        beta: beta,
        pdf: 1. as Float,
        pdf_reversed: 1. as Float,
    };
    random_walk(scene, ray_differential, sampler, allocator, beta, pdfdir, TransportMode::Radiance, path) + 1
}

fn generate_light_subpath<'a, S: Sampler>(
    scene: &'a Scene, sampler: &mut S, 
    allocator: &mut Allocator<'a>, path: &mut [Node<'a>]
) -> usize {
    if path.len() == 0 { return 0; }
    let (light_index, light_pdf, _) = scene.light_distribution.sample_discrete(sampler.next());
    let light = scene.get_light(light_index);
    // TODO
    let pathinfo = light.generate_path(sampler.get_light_sample());
    if pathinfo.pdfpos == 0. as Float || pathinfo.pdfdir == 0. as Float || pathinfo.radiance.is_black() {
        return 0;
    }
    path[0] = Node::Light{
        light: light,
        info: InteractInfo{
            pos: pathinfo.ray.origin(),
            wo: pathinfo.ray.direction(),
            norm: pathinfo.normal,
        },
        beta: pathinfo.radiance,
        pdf: pathinfo.pdfpos * light_pdf,
        pdf_reversed: 1. as Float,
    };
    let beta = pathinfo.radiance * pathinfo.ray.direction().dot(pathinfo.normal).abs() / (light_pdf * pathinfo.pdfpos * pathinfo.pdfdir);
    // TODO: handle infinite lights
    random_walk(scene, pathinfo.ray.into(), sampler, allocator, beta, pathinfo.pdfdir, TransportMode::Importance, path) + 1
}

fn random_walk<'a, S: Sampler>(
    scene: &'a Scene, mut ray_differential: RayDifferential,
    sampler: &mut S, allocator: &mut Allocator<'a>,
    mut beta: RGBSpectrumf, mut pdf: Float, mode: TransportMode,
    path: &mut [Node<'a>]
) -> usize {
    if path.len() == 1 { return 0; }
    let mut pdfrev = 0. as Float;
    let mut bounces = 1isize;
    let pathptr = path.as_mut_ptr();
    loop {
        let (node, prev) = unsafe {
            (pathptr.offset(bounces).as_mut().unwrap(),
            pathptr.offset(bounces-1).as_mut().unwrap())
        };
        // TODO: handle medium
        if let Some(mut si) = scene.aggregate.intersect_ray(&mut ray_differential.ray) {
            // TODO: handle infinite lights
            if let Some(primitive) = si.primitive_hit {
                let dxy = si.compute_dxy(&ray_differential);
                // FIXME: accounting for transport modes
                let bsdf = primitive.get_material().compute_scattering(
                    &mut si, &dxy, allocator
                );
                *node = Node::Surface{
                    // TODO: check if this is valid
                    bsdf: unsafe {(&bsdf as *const _).as_ref().unwrap()},
                    si: si,
                    beta: beta,
                    pdf: pdf,
                    pdf_reversed: 1. as Float,
                };
                let pdf_converted = prev.convert_density(&node, pdf);
                *node.get_pdf_mut() = pdf_converted;
                bounces += 1;
                if bounces as usize >= path.len() { break; }
                let wo = node.wo();
                let (f, wi, pdffwd) = bsdf.evaluate_sampled(wo, sampler.next_2d(), BXDF_ALL);
                if f.is_black() || pdffwd == 0. as Float { break; }
                pdf = pdffwd;
                beta *= f * wi.dot(si.shading_norm).abs() / pdf;
                pdfrev = bsdf.pdf(wi, wo, BXDF_ALL);
                // FIXME: delta
                beta *= correct_shading_normal(&si, wo, wi, mode);
                // FIXME: spawn ray differential
                ray_differential = RawRay::from_od(si.basic.pos, wi).into();
            } else {
                break;
            }
        } else {
            break;
        }
        let pdf_converted = node.convert_density(&prev, pdfrev);
        *prev.get_pdf_rev_mut() = pdf_converted;
    }
    bounces as usize
}

#[inline]
fn correct_shading_normal(si: &SurfaceInteraction, wo: Vector3f, wi: Vector3f, mode: TransportMode) -> Float {
    if mode == TransportMode::Importance {
        let num = (wo.dot(si.shading_norm) * wi.dot(si.basic.norm)).abs();
        let denom = (wo.dot(si.basic.norm) * wi.dot(si.shading_norm)).abs();
        if denom == 0. as Float { 0. as Float }
        else { num/denom }
    } else { 1. as Float }
}

#[derive(Copy, Clone, PartialEq)]
enum TransportMode {
    Radiance,
    Importance,
}

mod node;
