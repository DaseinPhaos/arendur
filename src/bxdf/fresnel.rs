// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Accounts for the Fresnel reflectance
use geometry::prelude::*;
use std::mem;
use spectrum::{Spectrum, RGBSpectrumf};

/// compute fresnel reflectance for dielectrics
fn fresnel_dielectric(mut cos_theta_i: Float, mut etai: Float, mut etat: Float) -> Float {
    if cos_theta_i < 0.0 as Float {
        // swap direction
        mem::swap(&mut etai, &mut etat);
        cos_theta_i = -cos_theta_i;
    }

    let sin_theta_i = (1.0 as Float - cos_theta_i * cos_theta_i).abs().sqrt();
    let sin_theta_t = etai / etat * sin_theta_i;
    if sin_theta_t >= 1.0 as Float {
        // total reflection
        return 1.0 as Float;
    }
    let cos_theta_t = (1.0 as Float - sin_theta_t * sin_theta_t).sqrt();
    let etci = etat * cos_theta_i;
    let eict = etai * cos_theta_t;
    let r_para = (etci - eict) / (etci + eict);
    let eici = etai * cos_theta_i;
    let etct = etat * cos_theta_t;
    let r_perp = (eici - etct) / (eici + etct);
    (r_para * r_para + r_perp * r_perp) * 0.5 as Float
}

/// compute fresnel reflectance for conductors
fn fresnel_conductor(mut cos_theta_i: Float, mut etai: RGBSpectrumf, mut etat: RGBSpectrumf, k: RGBSpectrumf) -> RGBSpectrumf {
    if cos_theta_i < 0.0 as Float {
        // swap direction
        mem::swap(&mut etai, &mut etat);
        cos_theta_i = -cos_theta_i;
    }

    let sin_theta_i2 = 1.0 as Float - cos_theta_i * cos_theta_i;
    let cos_theta_i2 = cos_theta_i * cos_theta_i;
    let sin_theta_i4 = sin_theta_i2 * sin_theta_i2;
    let sin_theta_i2 = RGBSpectrumf::grey_scale(sin_theta_i2);
    let sin_theta_i4 = RGBSpectrumf::grey_scale(sin_theta_i4);
    let cos_theta_i2 = RGBSpectrumf::grey_scale(cos_theta_i2);
    
    let eta = etat/etai;
    let eta2 = eta * eta;
    let k2 = k * k;
    let tmp0 = eta2 - k2 - sin_theta_i2;
    let a2pb2 = (tmp0 * tmp0 + 4.0 as Float * eta2 * k2).sqrt();
    // FIXME: wrong
    let am2 = (a2pb2 * 2.0 as Float).sqrt();

    let r_perp = (a2pb2 + cos_theta_i2 - am2 * cos_theta_i) / (a2pb2 + cos_theta_i2 + am2 * cos_theta_i);
    let tmpa = a2pb2 * cos_theta_i2;
    let tmpb = am2 * cos_theta_i * sin_theta_i2 + sin_theta_i4;
    let r_para = r_perp * (tmpa - tmpb) / (tmpa + tmpb);
    (r_para * r_para + r_perp * r_perp) * 0.5 as Float   
}

/// A fresnel interface
pub trait Fresnel {
    /// given an incoming direction, specify the reflectance
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrumf;
}

/// A fresnel conductor
#[derive(Copy, Clone, Debug)]
pub struct Conductor {
    pub etai: RGBSpectrumf,
    pub etat: RGBSpectrumf,
    pub k: RGBSpectrumf,
}

impl Conductor {
    /// construction
    #[inline]
    pub fn new(etai: RGBSpectrumf, etat: RGBSpectrumf, k: RGBSpectrumf) -> Conductor {
        Conductor {
            etai: etai, etat: etat, k: k
        }
    }
}

impl Fresnel for Conductor {
    #[inline]
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrumf {
        fresnel_conductor(cos_theta_i, self.etai, self.etat, self.k)
    }
}

/// A fresnel dielectric
#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    pub etai: Float,
    pub etat: Float,
}

impl Dielectric {
    /// Construction
    #[inline]
    pub fn new(etai: Float, etat: Float) -> Dielectric {
        Dielectric{
            etai: etai, etat: etat
        }
    }
}

impl Fresnel for Dielectric {
    #[inline]
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrumf {
        RGBSpectrumf::grey_scale(fresnel_dielectric(cos_theta_i, self.etai, self.etat))
    }
}

pub struct Noop;

impl Fresnel for Noop {
    #[inline]
    fn evaluate(&self, _cos_theta_i: Float) -> RGBSpectrumf {
        RGBSpectrumf::grey_scale(1.0 as Float)
    }
}