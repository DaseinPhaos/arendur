// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Foundamental types

use cgmath;
use super::float;
use std::ops;

pub type Float = f32;
pub type FSize = u32;
pub type Point2f = cgmath::Point2<Float>;
pub type Point3f = cgmath::Point3<Float>;
pub type Vector2f = cgmath::Vector2<Float>;
pub type Vector3f = cgmath::Vector3<Float>;
pub type Vector4f = cgmath::Vector4<Float>;
pub type Matrix2f = cgmath::Matrix2<Float>;
pub type Matrix3f = cgmath::Matrix3<Float>;
pub type Matrix4f = cgmath::Matrix4<Float>;
pub type Basis3f = cgmath::Basis3<Float>;
pub use cgmath::{Point2, Point3, Vector2, Vector3, Vector4, Basis3, BaseNum, BaseFloat, Matrix4, PartialOrd, Deg, Rad, ApproxEq};
pub use cgmath::prelude::*;
//pub use num_traits::{Num, NumCast};

/// Floating point values with accumulative error bounds
#[derive(Debug, Copy, Clone)]
pub struct EFloat {
    pub value: Float,
    pub err: Float,
}

impl EFloat {
    #[inline]
    pub fn lower_bound(self) -> Float {
        float::next_down(self.value - self.err)
    }

    #[inline]
    pub fn upper_bound(self) -> Float {
        float::next_up(self.value + self.err)
    }
}

impl From<Float> for EFloat {
    #[inline]
    fn from(f: Float) -> EFloat {
        EFloat{
            value: f, err: 0. as Float,
        }
    }
}

impl From<EFloat> for Float {
    #[inline]
    fn from(f: EFloat) -> Float {
        f.value
    }
}

impl ops::Add for EFloat {
    type Output = EFloat;
    #[inline]
    fn add(self, rhs: EFloat) -> EFloat {
        let value = self.value + rhs.value; 
        let errsum = self.err + rhs.err;
        EFloat{
            value,
            err: (value.abs() + errsum) * float::eb_term(1. as Float) + errsum
        }
    }
}

impl ops::Sub for EFloat {
    type Output = EFloat;
    #[inline]
    fn sub(self, rhs: EFloat) -> EFloat {
        let value = self.value - rhs.value; 
        let errsum = (self.err - rhs.err).abs();
        EFloat{
            value,
            err: (value.abs() + errsum) * float::eb_term(1. as Float) + errsum
        }
    }
}

impl ops::Mul for EFloat {
    type Output = EFloat;
    #[inline]
    fn mul(self, rhs: EFloat) -> EFloat {
        let value = self.value * rhs.value; 
        let errsum = (self.err*rhs.value + rhs.err*self.value + self.err*rhs.err).abs();
        EFloat{
            value,
            err: (value.abs() + errsum) * float::eb_term(1. as Float) + errsum
        }
    }
}

impl ops::Div for EFloat {
    type Output = EFloat;
    #[inline]
    fn div(self, rhs: EFloat) -> EFloat {
        let value = self.value / rhs.value;
        // FIXME: not conservative here?
        let errsum = self.err / rhs.value.abs();
        EFloat{
            value,
            err: (value.abs() + errsum) * float::eb_term(1. as Float) + errsum
        }
    }
}


/// Point on unit sphere represented as spherical coordinate in radians
#[derive(Copy, Clone, PartialEq)]
pub struct Sphericalf {
    pub theta: Float,
    pub phi: Float,
}

impl Sphericalf {
    /// Construction
    #[inline]
    pub fn new(theta: Float, phi: Float) -> Sphericalf {
        Sphericalf{
            theta: theta, phi: phi
        }
    }

    /// from anything that can be converted into a `Rad`ian
    #[inline]
    pub fn from<T, P>(theta: T, phi: P) -> Sphericalf
        where T: Into<Rad<Float>>,
              P: Into<Rad<Float>>,
    {
        Sphericalf{
            theta: theta.into().0, phi: phi.into().0
        }
    }

    /// from a normalized vector
    #[inline]
    pub fn from_vec(v: Vector3f) -> Sphericalf {
        Sphericalf::new(
            Sphericalf::theta_from_vec(v),
            Sphericalf::phi_from_vec(v)
        )
    }

    /// into a vector
    #[inline]
    pub fn to_vec(&self) -> Vector3f {
        let sintheta = self.theta.sin();
        let costheta = self.theta.cos();
        let sinphi = self.phi.sin();
        let cosphi = self.phi.cos();

        Vector3f::new(
            sintheta * cosphi,
            sintheta * sinphi,
            costheta
        )
    }

    /// into a vector in the given coordinate frame
    #[inline]
    pub fn to_vec_in(&self, basis: &Basis3f) -> Vector3f {
        let vlocal = self.to_vec();
        basis.as_ref() * vlocal
    }

    /// Obtain theta from a normalized vector
    #[inline]
    pub fn theta_from_vec(v: Vector3f) -> Float {
        #[cfg(debug)]
        {
            assert_ulps_eq!(v.normalize(), 1.0 as Float);
        }
        let z = float::clamp(v.z, -1.0 as Float, 1.0 as Float);
        z.acos()
    }

    /// Obtain phi from a normalized vector
    #[inline]
    pub fn phi_from_vec(v: Vector3f) -> Float {
        #[cfg(debug)]
        {
            assert_ulps_eq!(v.normalize(), 1.0 as Float);
        }
        let ret = v.y.atan2(v.x);
        if ret < 0.0 as Float {
            ret + float::pi() * 2.0 as Float
        } else {
            ret
        }
    }
}

/// helper function dealing with normal vectors and spherical coordinates
pub mod normal {
    use super::*;
    #[inline]
    pub fn cos_theta(norm: Vector3f) -> Float {
        norm.z
    }

    #[inline]
    pub fn cos2_theta(norm: Vector3f) -> Float {
        norm.z * norm.z
    }

    #[inline]
    pub fn sin2_theta(norm: Vector3f) -> Float {
        ((1.0 as Float) - cos2_theta(norm)).abs()
    }

    #[inline]
    pub fn sin_theta(norm: Vector3f) -> Float {
        sin2_theta(norm).sqrt()
    }

    #[inline]
    pub fn tan_theta(norm: Vector3f) -> Float {
        sin_theta(norm)/cos_theta(norm)
    }

    #[inline]
    pub fn tan2_theta(norm: Vector3f) -> Float {
        sin2_theta(norm)/cos2_theta(norm)
    }

    #[inline]
    pub fn cos_phi(norm: Vector3f) -> Float {
        let sin_theta = sin_theta(norm);
        if sin_theta == 0.0 as Float {
            1.0 as Float
        } else {
            float::clamp(norm.x / sin_theta, -1.0 as Float, 1.0 as Float)
        }
    }

    pub fn sin_phi(norm: Vector3f) -> Float {
        let sin_theta = sin_theta(norm);
        if sin_theta == 0.0 as Float {
            0.0 as Float
        } else {
            float::clamp(norm.y / sin_theta, -1.0 as Float, 1.0 as Float)
        }
    }

    #[inline]
    pub fn cos2_phi(norm: Vector3f) -> Float {
        cos_phi(norm).powi(2)
    }

    #[inline]
    pub fn sin2_phi(norm: Vector3f) -> Float {
        sin_phi(norm).powi(2)
    }

    #[inline]
    pub fn cos_dphi(n0: Vector3f, n1: Vector3f) -> Float {
        float::clamp(
            (n0.x * n1.x + n0.y * n1.y) / (
                (n0.x * n0.x + n0.y * n0.y) * (n1.x * n1.x + n1.y * n1.y)
            ).sqrt(),
            -1.0 as Float,
            1.0 as Float
        )
    }

    #[inline]
    pub fn reflect(wo: Vector3f, n: Vector3f) -> Vector3f {
        -wo + 2.0 as Float * wo.dot(n) * n
    }

    #[inline]
    pub fn refract(wo: Vector3f, n: Vector3f, eta: Float) -> Option<Vector3f> {
        let cos_theta = wo.dot(n);
        let sin2_theta = 1. as Float - cos_theta * cos_theta;
        let sin2_thetat = eta * eta * sin2_theta.max(0. as Float);
        if sin2_thetat >= 1. as Float {
            None
        } else {
            let cos_thetat = (1. as Float - sin2_thetat).sqrt();
            Some(
                -eta * wo + (eta*cos_theta - cos_thetat)*n
            )
        }
    }

    /// given `e1`, returns `e2` and `e3` that forms a basis
    #[inline]
    pub fn get_basis_from(dir: Vector3f) -> (Vector3f, Vector3f) {
        let mut up = Vector3f::new(0. as Float, 0. as Float, 1. as Float);
        if relative_eq!(up, dir) {
            up = Vector3f::new(0. as Float, 1. as Float, 0. as Float);
        };
        let u = up.cross(dir).normalize();
        let v = dir.cross(u).normalize();
        (u, v)
    }
}