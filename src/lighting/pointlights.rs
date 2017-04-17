// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Some point lights.

use super::*;
use cgmath::Quaternion;

/// An isotropic point light emitting same amount of light in all directions
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PointLight {
    /// position in parent frame
    pub posw: Point3f,
    /// light intensity
    pub intensity: RGBSpectrumf,
}

impl Light for PointLight {
    #[inline]
    fn flags(&self) -> LightFlag {
        LIGHT_DPOS
    }

    #[inline]
    fn is_delta(&self) -> bool {
        true
    }

    #[inline]
    fn evaluate_sampled(&self, posw: Point3f, _sample: Point2f) -> LightSample {
        let pfrom = self.posw;
        let pto = posw;
        let radiance = self.intensity/(pto-pfrom).magnitude();
        LightSample {
            radiance: radiance,
            pdf: 1.0 as Float,
            pto: pto,
            pfrom: pfrom,
        }
    }

    /// returns an estimation of total power of this light
    fn power(&self) -> RGBSpectrumf {
        self.intensity * (float::pi() * 4.0 as Float)
    }
}

/// Spot light emit light in a cone of directions
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SpotLight {
    // position in parent frame
    posw: Point3f,
    /// light intensity
    pub intensity: RGBSpectrumf,
    // cosine of the total angle
    cost: Float,
    // cosine of the falloff starting angle
    cosf: Float,
    // local parent
    local_parent: Matrix4f,
    // parent local
    parent_local: Matrix4f,
}

impl SpotLight {
    /// construction, angles are provided in radians
    #[inline]
    pub fn new(pos: Point3f, towards: Vector3f, intensity: RGBSpectrumf, total_angle: Float, start_falloff_angle: Float) -> SpotLight {
        assert!(total_angle>start_falloff_angle);
        assert!(start_falloff_angle> 0.0 as Float);
        assert!(total_angle < (float::pi() * 2.0 as Float));
        let towards = towards.normalize();
        let rotation: Matrix4f = Quaternion::from_arc(towards, Vector3f::new(0.0 as Float, 0.0 as Float, 1.0 as Float), None).into();
        let translation = Matrix4f::from_translation(pos - Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float));
        let parent_local = rotation * translation;
        let local_parent = parent_local.invert().expect("invalid inversion");
        SpotLight{
            posw: pos,
            intensity: intensity,
            cost: total_angle.cos(),
            cosf: start_falloff_angle.cos(),
            local_parent: local_parent,
            parent_local: parent_local,
        }
    }

    /// set angles. angles are provided in radians
    #[inline]
    pub fn set_angles(&mut self, total_angle: Float, start_falloff_angle: Float) {
        assert!(total_angle>start_falloff_angle);
        assert!(start_falloff_angle> 0.0 as Float);
        assert!(total_angle < (float::pi() * 2.0 as Float));
        self.cost = total_angle.cos();
        self.cosf = start_falloff_angle.cos();
    }

    /// set orientations
    #[inline]
    pub fn set_orientations(&mut self, pos: Point3f, towards: Vector3f) {
        let towards = towards.normalize();
        let rotation: Matrix4f = Quaternion::from_arc(towards, Vector3f::new(0.0 as Float, 0.0 as Float, 1.0 as Float), None).into();
        let translation = Matrix4f::from_translation(pos - Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float));
        self.parent_local = rotation * translation;
        self.local_parent = self.parent_local.invert().expect("invalid inversion");
        self.posw = pos;
    }

    // Compute the falloff, given a normalized direction in parent frame
    #[inline]
    fn falloff(&self, dir: Vector3f) -> Float {
        let cos_theta = self.parent_local.transform_vector(dir).z;
        if cos_theta < self.cost {
            0.0 as Float
        } else if cos_theta > self.cosf {
            1.0 as Float
        } else {
            let delta = (cos_theta - self.cost) / (self.cosf - self.cost);
            let delta2 = delta * delta;
            delta2 * delta2
        }
    }
}

impl Light for SpotLight {
    #[inline]
    fn flags(&self) -> LightFlag {
        LIGHT_DPOS
    }

    #[inline]
    fn is_delta(&self) -> bool {
        true
    }

    #[inline]
    fn evaluate_sampled(&self, posw: Point3f, _sample: Point2f) -> LightSample {
        let pfrom = self.posw;
        let pto = posw;
        let dir = pto - pfrom;
        let mag = dir.magnitude();
        let radiance = self.intensity * self.falloff(dir/mag)/mag;
        LightSample {
            radiance: radiance,
            pdf: 1.0 as Float,
            pto: pto,
            pfrom: pfrom,
        }
    }

    /// returns an estimation of total power of this light
    fn power(&self) -> RGBSpectrumf {
        self.intensity * (float::pi() * 2.0 as Float) * (
            1.0 as Float - 0.5 as Float * (self.cosf - self.cost)
        )
    }
}

// TODO: /// Projection light
// pub struct ProjectionLight {

// }
