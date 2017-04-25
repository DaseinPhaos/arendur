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
use sample;

/// An isotropic point light emitting same amount of light in all directions
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PointLight {
    /// position in parent frame
    pub posw: Point3f,
    /// light intensity
    pub intensity: RGBSpectrumf,
}

impl PointLight {
    pub fn new(pos: Point3f, intensity: RGBSpectrumf) -> PointLight {
        PointLight{ posw: pos, intensity: intensity,}
    }
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

    /// Given a position `pos` in local frame and a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    ///
    /// Pointlights assume a uniform radiance around the sphere,
    /// thus the returned light sample always come from the light,
    /// with a radiance $\propto 1/d^2$. 
    #[inline]
    fn evaluate_sampled(&self, pos: Point3f, _sample: Point2f) -> LightSample {
        let pfrom = self.posw;
        let pto = pos;
        let radiance = self.intensity/(pto-pfrom).magnitude2();
        LightSample {
            radiance: radiance,
            pdf: 1.0 as Float,
            pto: pto,
            pfrom: pfrom,
        }
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        let dir = sample::sample_uniform_sphere(samples.pfilm);
        let ray = RawRay::from_od(self.posw, dir);

        PathInfo{
            ray: ray,
            normal: dir,
            pdfpos: 1. as Float,
            pdfdir: sample::pdf_uniform_sphere(),
            radiance: self.intensity,
        }
    }

    #[inline]
    fn pdf(&self, _pos: Point3f, _dir: Vector3f, _normal: Vector3f) -> (Float, Float) {
        (0. as Float, sample::pdf_uniform_sphere())
    }

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

    /// Given a position `pos` in local frame and a uniform `sample`
    /// in $[0, 1)$, sample an incoming direction from the light to that
    /// location, returns the sampling result in a `LightSample`.
    ///
    /// Spotlights assume all points inside its viewing volume would
    /// be lit by lights from its position. Thus the returned lightsample
    /// always come from `self.posw` to `pos`, with radiance $\propto 1/d^2$.
    #[inline]
    fn evaluate_sampled(&self, pos: Point3f, _sample: Point2f) -> LightSample {
        let pfrom = self.posw;
        let pto = pos;
        let dir = pto - pfrom;
        let mag2 = dir.magnitude2();
        let radiance = self.intensity * self.falloff(dir/mag2.sqrt())/mag2;
        LightSample {
            radiance: radiance,
            pdf: 1.0 as Float,
            pto: pto,
            pfrom: pfrom,
        }
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        let dir = sample::sample_uniform_cone(samples.pfilm, self.cost);
        let ray = RawRay::from_od(self.posw, dir);

        PathInfo{
            ray: ray,
            // TODO: double check if the direction should be the main ray's direction
            normal: dir,
            pdfpos: 1. as Float,
            pdfdir: sample::pdf_uniform_cone(self.cost),
            radiance: self.intensity * self.falloff(dir),
        }
    }

    #[inline]
    fn pdf(&self, _pos: Point3f, dir: Vector3f, _normal: Vector3f) -> (Float, Float) {
        let costheta = normal::cos_theta(dir);
        let pdfdir = if costheta >= self.cost {
            sample::pdf_uniform_cone(self.cost)
        } else {
            0. as Float
        };
        (0. as Float, pdfdir)
    }

    #[inline]
    fn power(&self) -> RGBSpectrumf {
        self.intensity * (float::pi() * 2.0 as Float) * (
            1.0 as Float - 0.5 as Float * (self.cosf - self.cost)
        )
    }
}

// TODO: /// Projection light
// pub struct ProjectionLight {

// }
