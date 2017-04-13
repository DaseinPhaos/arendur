//! Foundamental types

use cgmath;
use super::float;

pub type Float = f32;
pub type Point2f = cgmath::Point2<Float>;
pub type Point3f = cgmath::Point3<Float>;
pub type Vector2f = cgmath::Vector2<Float>;
pub type Vector3f = cgmath::Vector3<Float>;
pub type Vector4f = cgmath::Vector4<Float>;
pub type Matrix3f = cgmath::Matrix3<Float>;
pub type Matrix4f = cgmath::Matrix4<Float>;
pub type Basis3f = cgmath::Basis3<Float>;
pub use cgmath::{Point2, Point3, Vector2, Vector3, Vector4, Basis3, BaseNum, BaseFloat, Matrix4, PartialOrd, Deg, Rad};
pub use cgmath::prelude::*;
//pub use num_traits::{Num, NumCast};

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