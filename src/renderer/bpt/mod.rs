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
    path: PathBuf,
    max_depth: usize,
}

impl<S: Sampler> BPTRenderer<S> {
    pub fn new<P: AsRef<Path> + ?Sized>(
        sampler: S, camera: Arc<Camera>, path: &P, max_depth: usize
    ) -> BPTRenderer<S> {
        BPTRenderer{
            sampler: sampler,
            camera: camera,
            path: path.as_ref().to_path_buf(),
            max_depth: max_depth,
        }
    }
}

impl<S: Sampler> Renderer for BPTRenderer<S> {
    fn render(&mut self, scene: &Scene) {
        let mut tiles: Vec<FilmTile<RGBSpectrumf>> = self.camera.get_film().spawn_flat_tiles(16, 16);
        // for tile in tiles.iter_mut() {
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
                    let ncam = generate_camera_subpath(
                        scene, &mut sampler, &mut allocator, &*self.camera, pfilm, cam_nodes
                    );
                    let nlight = generate_light_subpath(
                        scene, &mut sampler, &mut allocator, light_nodes
                    );
                    let mut l = RGBSpectrumf::black();
                    for t in 1..ncam {
                        for s in 0..nlight {
                            let depth = t as isize + s as isize - 2isize;
                            if (s==1 && t==1) || depth < 0 || depth>self.max_depth as isize {
                                continue;
                            }
                            let mut pfilm_new = pfilm;
                            let mut mis_weight = 0. as Float;
                            let lpath = connect(scene, &mut cam_nodes[0..t], &mut light_nodes[0..s], &*self.camera, &mut sampler, &mut pfilm_new, &mut mis_weight);
                            // TODO: visualize strategies
                            if t!=1 {l+=lpath;}
                            else {tile.add_sample(pfilm_new, &lpath)};
                        }
                    }
                    tile.add_sample(pfilm, &l);
                    if !sampler.next_sample() { break; }
                }
            }
        });
        // }
        let render_result = self.camera.get_film().collect_into(tiles);
        render_result.save(self.path.clone()).expect("saving failure");
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
            wo: Vector3f::zero(),
            norm: Vector3f::zero(),
        },
        beta: beta,
        pdf: pdfpos,
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
            // TODO: double check
            pos: pathinfo.ray.origin(),
            wo: pathinfo.ray.direction(),
            norm: Vector3f::zero(),
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
    let mut bounces = 1usize;
    // let pathptr = path.as_mut_ptr();
    loop {
        // let (node, prev) = unsafe {
        //     (pathptr.offset(bounces).as_mut().unwrap(),
        //     pathptr.offset(bounces-1).as_mut().unwrap())
        // };
        // TODO: handle medium
        if let Some(mut si) = scene.aggregate.intersect_ray(&mut ray_differential.ray) {
            // TODO: handle infinite lights
            if let Some(primitive) = si.primitive_hit {
                let dxy = si.compute_dxy(&ray_differential);
                // FIXME: accounting for transport modes
                let bsdf = primitive.get_material().compute_scattering(
                    &mut si, &dxy, allocator
                );
                let bsdf = allocator.alloc(bsdf);
                path[bounces] = Node::Surface{
                    // TODO: check if this is valid
                    // bsdf: unsafe {(&bsdf as *const _).as_ref().unwrap()},
                    bsdf: bsdf,
                    si: si,
                    beta: beta,
                    pdf: pdf,
                    pdf_reversed: 1. as Float,
                };
                let pdf_converted = path[bounces-1].convert_density(&path[bounces], pdf);
                *path[bounces].get_pdf_mut() = pdf_converted;
                bounces += 1;
                if bounces as usize >= path.len() { break; }
                let wo = path[bounces].wo();
                let (f, wi, pdffwd) = if mode == TransportMode::Radiance {
                    bsdf.evaluate_sampled(wo, sampler.next_2d(), BXDF_ALL)
                } else {
                    bsdf.evaluate_importance_sampled(wo, sampler.next_2d(), BXDF_ALL)
                };
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
        let pdf_converted = path[bounces-1].convert_density(&path[bounces-2], pdfrev);
        *path[bounces-2].get_pdf_rev_mut() = pdf_converted;
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

fn connect<S: Sampler>(
    scene: &Scene, cam_nodes: &mut [Node],
    light_nodes: &mut [Node], camera: &Camera,
    sampler: &mut S, praster: &mut Point2f, 
    mis_weight: &mut Float
) -> RGBSpectrumf {
    let mut ret = RGBSpectrumf::black();
    let t = cam_nodes.len();
    let s = light_nodes.len();
    if t > 1 
        && s != 0 
        && cam_nodes.last().unwrap().is_light_node() {
        // invalid connection strategy
        return ret;
    }

    let mut sampled;
    if s == 0 {
        // no lights
        let pt = cam_nodes.last().unwrap();
        if let Some(light) = pt.as_light() {
            // TODO: handle infinite light sources
        }
    } else if t == 1 {
        // sample a point on camera, connecting to light subpath
        let qs = light_nodes.last().unwrap();
        if qs.is_connectible() {
            let (importance_sample, pr) = camera.evaluate_importance_sampled(
                qs.pos(), sampler.next_2d()
            );
            *praster = pr;
            if !importance_sample.no_effect() {
                sampled = Node::Camera{
                    camera: camera,
                    info: InteractInfo{
                        pos: importance_sample.pfrom,
                        wo: Vector3f::zero(),
                        norm: Vector3f::zero(),
                    },
                    beta: RGBSpectrumf::new(1. as Float, 1. as Float, 1. as Float)/importance_sample.pdf,
                    pdf: 1. as Float,
                    pdf_reversed: 0. as Float,
                };
                let l = qs.get_beta() * qs.evaluate(&sampled, TransportMode::Importance) * sampled.get_beta();
                if qs.on_surface() && !l.is_black() && !importance_sample.occluded(&*scene.aggregate) {
                    ret = l * importance_sample.wi().dot(qs.shading_norm()).abs();
                }
            }
        }
    } else if s == 1 {
        // sample a point on light, connect it to the camera subpath
        let pt = cam_nodes.last().unwrap();
        if pt.is_connectible() {
            let (lightidx, lightpdf, _) = scene.light_distribution.sample_discrete(sampler.next());
            let light = scene.get_light(lightidx);
            let lightsample = light.evaluate_sampled(pt.pos(), sampler.next_2d());
            if !lightsample.no_effect() {
                sampled = Node::Light{
                    light: light,
                    info: InteractInfo{
                        // TODO: double check
                        pos: lightsample.pfrom,
                        wo: Vector3f::zero(),
                        norm: Vector3f::zero(),
                    },
                    beta: lightsample.radiance / (lightsample.pdf * lightpdf),
                    pdf: 0. as Float,
                    pdf_reversed: 0. as Float,
                };
                let pdffwd = sampled.pdf_light_origin(scene, pt);
                *sampled.get_pdf_mut() = pdffwd;
                let l = pt.get_beta() * pt.evaluate(&sampled, TransportMode::Radiance) * sampled.get_beta();
                if pt.on_surface() && !l.is_black() && !lightsample.occluded(&*scene.aggregate) {
                    ret = l * lightsample.wi().dot(pt.shading_norm()).abs();
                }
            }
        }
    } else {
        let qs = light_nodes.last().unwrap();
        let pt = cam_nodes.last().unwrap();
        if qs.is_connectible() && pt.is_connectible() {
            let l = qs.get_beta() * qs.evaluate(pt, TransportMode::Importance) * pt.evaluate(qs, TransportMode::Radiance) * pt.get_beta();
            if !l.is_black() {
                ret = l * g(scene, sampler, qs, pt);
            }
        }
    }

    *mis_weight = if ret.is_black() {
        0. as Float
    } else {
        cal_mis_weight(scene, cam_nodes, light_nodes)
    };
    ret * (*mis_weight)
}

fn g<S: Sampler>(scene: &Scene, sampler: &mut S, v0: &Node, v1: &Node) -> RGBSpectrumf {
    let d = v0.pos() - v1.pos();
    let mut g = 1. as Float / d.magnitude2();
    let d = d * g.sqrt();
    if v0.on_surface() { g *= v0.shading_norm().dot(d).abs(); }
    if v1.on_surface() { g *= v1.shading_norm().dot(d).abs(); }
    let ray = RawRay::from_od(v1.pos(), d);
    let epsilon = Point3f::default_epsilon();
    let epsilon = Vector3f::new(epsilon, epsilon, epsilon);
    let pfrom = v0.pos() + epsilon;
    let mut ray = RawRay::spawn(pfrom, v1.pos());
    let unoccluded = if let Some(si) = scene.aggregate.intersect_ray(&mut ray) {
        relative_eq!(si.basic.pos, v1.pos())
    } else {
        true
    };
    if unoccluded {
        // TODO: double check
        RGBSpectrumf::new(g, g, g)
    } else {
        RGBSpectrumf::black()
    }
}

fn cal_mis_weight(
    scene: &Scene, cam_nodes: &[Node],
    light_nodes: &[Node]
) -> Float {
    let t = cam_nodes.len() as usize;
    let s = light_nodes.len() as usize;
    if s + t == 2 {return 1. as Float; }
    let mut sum_ri = 0. as Float;
    let remap0 = |f| {
        if f == 0. as Float {
            1. as Float
        } else {
            f
        }
    };
    let mut ri = 1. as Float;
    for i in 1..t {
        let pdfrev = cam_nodes[t-i-1].get_pdf_rev();
        let pdffwd = cam_nodes[t-i-1].get_pdf();
        ri *= remap0(pdfrev)/remap0(pdffwd);
        sum_ri += ri;
    }
    ri = 1. as Float;
    for i in 0..s {
        let pdfrev = light_nodes[s-i-1].get_pdf_rev();
        let pdffwd = light_nodes[s-i-1].get_pdf();
        ri *= remap0(pdfrev)/remap0(pdffwd);
        sum_ri += ri;
    }
    1. as Float / (1. as Float + sum_ri)
}

#[derive(Copy, Clone, PartialEq)]
enum TransportMode {
    Radiance,
    Importance,
}

mod node;
