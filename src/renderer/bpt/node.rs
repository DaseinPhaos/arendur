// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! implements path nodes

#![allow(dead_code)]

use geometry::prelude::*;
use filming::prelude::*;
use lighting::prelude::*;
use material::prelude::*;
use spectrum::{Spectrum, RGBSpectrumf};
use bxdf::prelude::*;
use renderer::scene::Scene;
use std::ptr;
use std::mem;
use super::TransportMode;

#[derive(Copy, Clone)]
pub enum Node<'a> {
    Camera{
        camera: &'a Camera,
        info: InteractInfo,
        beta: RGBSpectrumf,
        pdf: Float,
        pdf_reversed: Float,
    },
    Light{
        light: &'a Light,
        info: InteractInfo,
        beta: RGBSpectrumf,
        pdf: Float,
        pdf_reversed: Float,
    },
    Surface{
        bsdf: &'a Bsdf<'a>,
        si: SurfaceInteraction<'a>,
        beta: RGBSpectrumf,
        pdf: Float,
        pdf_reversed: Float,
    },
    Medium{
        info: InteractInfo,
        beta: RGBSpectrumf,
        pdf: Float,
        pdf_reversed: Float,
    },
}

impl<'a> Node<'a> {
    #[inline]
    pub fn info(&self) -> InteractInfo {
        match *self {
            Node::Camera{info, ..} => info,
            Node::Light{info, ..} => info,
            Node::Surface{ref si, ..} => si.basic,
            Node::Medium{info, ..} => info,
        }
    }

    #[inline(always)]
    pub fn pos(&self) -> Point3f {
        self.info().pos
    }

    #[inline(always)]
    pub fn norm(&self) -> Vector3f {
        self.info().norm
    }

    #[inline(always)]
    pub fn shading_norm(&self) -> Vector3f {
        match *self {
            Node::Camera{ref info, ..} => info.norm,
            Node::Light{ref info, ..} => info.norm,
            Node::Surface{ref si, ..} => si.shading_norm,
            Node::Medium{ref info, ..} => info.norm,
        }
    }

    #[inline(always)]
    pub fn on_surface(&self) -> bool {
        self.norm() != Vector3f::zero()
    }

    #[inline(always)]
    pub fn wo(&self) -> Vector3f {
        self.info().wo
    }

    #[inline]
    pub fn evaluate(&self, next: &Node, mode: TransportMode) -> RGBSpectrumf {

        let wi = (next.pos() - self.pos()).normalize();
        match *self {
            Node::Surface{bsdf, ref si, ..} => {
                if mode == TransportMode::Radiance {
                    bsdf.evaluate(si.basic.wo, wi, BXDF_ALL)
                } else {
                    bsdf.evaluate_importance(si.basic.wo, wi, BXDF_ALL)
                }
            },
            _ => RGBSpectrumf::black(),
        }
    }

    #[inline]
    pub fn is_connectible(&self) -> bool {
        match *self {
            Node::Light{light, ..} => light.flags().intersects(LIGHT_DDIR),
            Node::Surface{bsdf, ..} => bsdf.have_n(BXDF_DIFFUSE|BXDF_GLOSSY|BXDF_REFLECTION|BXDF_TRANSMISSION) > 0,
            _ => true,
        }
    }

    #[inline(always)]
    pub fn is_light(&self) -> bool {
        match *self {
            Node::Light{..} => true,
            Node::Surface{ref si, ..} => si.is_emissive(),
            _ => false,
        }
    }

    #[inline(always)]
    pub fn is_light_node(&self) -> bool {
        match *self {
            Node::Light{..} => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_delta_light(&self) -> bool {
        match *self {
            Node::Light{light, ..} => light.is_delta(),
            _ => false,
        }
    }

    #[inline]
    pub fn convert_density(&self, next: &Node, mut pdf: Float) -> Float {
        // TODO: account for infinite area lights
        let wi = next.pos() - self.pos();
        let invdist2 = 1. as Float / wi.magnitude2();
        let norm = next.norm();
        if norm != Vector3f::new(0. as Float, 0. as Float, 0. as Float) {
            pdf *= norm.dot(wi*invdist2.sqrt()).abs();
        }
        pdf * invdist2
    }

    #[inline]
    pub fn pdf(&self, prev: Option<&Node>, next: &Node) -> Float {
        let wp = if let Some(prev) = prev {
            (prev.pos() - self.pos()).normalize()
        } else {
            Vector3f::new(0. as Float, 0. as Float, 0. as Float)
        };
        let wn = (next.pos() - self.pos()).normalize();
        let pdf = match *self {
            Node::Light{light, ref info, ..} => light.pdf(info.pos, wn, info.norm).1,
            Node::Camera{camera, ref info, ..} => camera.pdf(info.pos, wn).1,
            Node::Surface{bsdf, ..} => bsdf.pdf(wp, wn, BXDF_ALL),
            _ => unimplemented!(),
        };
        self.convert_density(next, pdf)
    }

    pub fn pdf_light(&self, next: &Node) -> Float {
        let wi = next.pos() - self.pos();
        let invdist2 = 1. as Float / wi.magnitude2();
        let wn = wi*invdist2.sqrt();
        let mut pdf = match *self {
            Node::Light{light, ref info, ..} => light.pdf(info.pos, wn, info.norm).1,
            Node::Surface{ref si, ..} => {
                if let Some(light) = si.primitive_hit {
                    light.pdf(si.basic.pos, wn, si.basic.norm).1
                } else {
                    0. as Float
                }
            },
            _ => 0. as Float,
        };
        let nnorm = next.norm();
        if nnorm != Vector3f::new(0. as Float, 0. as Float, 0. as Float) {
            pdf *= nnorm.dot(wn).abs();
        }
        pdf * invdist2
    }

    pub fn pdf_light_origin(&self, scene: &Scene, next: &Node) -> Float {
        let wi = (next.pos() - self.pos()).normalize();
        match *self {
            Node::Light{light, ref info, ..} => {
                let pdf_pos = light.pdf(info.pos, wi, info.norm).0;
                let mut pdf_choice = 0. as Float;
                for (i, l) in scene.lights.iter().enumerate() {
                    if ptr::eq(light, l.as_ref()) {
                        pdf_choice = scene.light_distribution.discrete_pdf(i);
                        break;
                    }
                }
                pdf_pos * pdf_choice
            }
            Node::Surface{ref si, ..} => {
                if let Some(light) = si.primitive_hit {
                    let pdf_pos = light.pdf(si.basic.pos, wi, si.basic.norm).0;
                    let mut pdf_choice = 0. as Float;
                    for (i, l) in scene.area_lights.iter().enumerate() {
                        // TODO: double check if desirable
                        if ptr::eq(light.as_light(), l.as_light()) {
                            pdf_choice = scene.light_distribution.discrete_pdf(i+scene.lights.len());
                            break;
                        }
                    }
                    pdf_pos * pdf_choice
                } else {
                    0. as Float
                }
            },
            _ => 0. as Float,
        }
    }

    #[inline]
    pub fn get_pdf_mut(&mut self) -> &mut Float {
        match *self {
            Node::Camera{ref mut pdf, ..} => pdf,
            Node::Light{ref mut pdf, ..} => pdf,
            Node::Surface{ref mut pdf, ..} => pdf,
            Node::Medium{ref mut pdf, ..} => pdf,
        }
    }

    #[inline]
    pub fn get_pdf_rev_mut(&mut self) -> &mut Float {
        match *self {
            Node::Camera{ref mut pdf_reversed, ..} => pdf_reversed,
            Node::Light{ref mut pdf_reversed, ..} => pdf_reversed,
            Node::Surface{ref mut pdf_reversed, ..} => pdf_reversed,
            Node::Medium{ref mut pdf_reversed, ..} => pdf_reversed,
        }
    }

    #[inline]
    pub fn get_pdf(&self) -> Float {
        match *self {
            Node::Camera{pdf, ..} => pdf,
            Node::Light{pdf, ..} => pdf,
            Node::Surface{pdf, ..} => pdf,
            Node::Medium{pdf, ..} => pdf,
        }
    }

    #[inline]
    pub fn get_pdf_rev(&self) -> Float {
        match *self {
            Node::Camera{pdf_reversed, ..} => pdf_reversed,
            Node::Light{pdf_reversed, ..} => pdf_reversed,
            Node::Surface{pdf_reversed, ..} => pdf_reversed,
            Node::Medium{pdf_reversed, ..} => pdf_reversed,
        }
    }

    #[inline]
    pub fn get_beta_mut(&mut self) -> &mut RGBSpectrumf {
        match *self {
            Node::Camera{ref mut beta, ..} => beta,
            Node::Light{ref mut beta, ..} => beta,
            Node::Surface{ref mut beta, ..} => beta,
            Node::Medium{ref mut beta, ..} => beta,
        }
    }

    #[inline]
    pub fn get_beta(&self) -> RGBSpectrumf {
        match *self {
            Node::Camera{beta, ..} => beta,
            Node::Light{beta, ..} => beta,
            Node::Surface{beta, ..} => beta,
            Node::Medium{beta, ..} => beta,
        }
    }

    #[inline]
    pub fn as_light(&self) -> Option<&Light> {
        match *self {
            Node::Light{light, ..} => Some(light),
            Node::Surface{ref si, ..} => si.primitive_hit.map(|p| p.as_light()),
            _ => None,
        }
    }
}

impl<'a> Default for Node<'a> {
    fn default() -> Self {unsafe {
        mem::uninitialized()
    }}
}