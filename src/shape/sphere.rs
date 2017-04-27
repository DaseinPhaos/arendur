// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use geometry::prelude::*;
use super::Shape;

/// A (possibly-partial) sphere, as a geometry definition
#[derive(Copy, Clone, PartialEq)]
pub struct Sphere {
    /// The radius of the sphere
    pub radius: Float,
    /// The lower bound xy-plane. Points with `z<zmin` are excluded.
    pub zmin: Float,
    /// The upper bound xy-plane. Points with `z>zmax` are excluded.
    pub zmax: Float,
    /// The maximum `phi`. Points with `phi>phimax` are excluded.
    pub phimax: Float,
    // These two are updated accordingly when `zmin` or `zmax` changes.
    thetamin: Float,
    thetamax: Float,
}

impl Sphere {
    /// Constructs a new `Sphere`.
    pub fn new(radius: Float, mut zmin: Float, mut zmax: Float, mut phimax: Float) -> Sphere {
        assert!(radius>(0.0 as Float), "Sphere radius should be positive");
        assert!(zmin<zmax, "zmin should be lower than zmax");

        if zmin < -radius { zmin = -radius; }
        if zmax > radius { zmax = radius; }

        if phimax < (0.0 as Float) { phimax = 0.0 as Float; }
        let twopi = float::pi() * (2.0 as Float);
        if phimax > twopi { phimax = twopi; }

        // TODO: double check
        let thetamin = (zmin/radius).acos();
        let thetamax = (zmax/radius).acos();

        Sphere {
            radius: radius,
            zmin: zmin,
            zmax: zmax,
            thetamin: thetamin,
            thetamax: thetamax,
            phimax: phimax,
        }
    }

    /// Constructs a full sphere
    #[inline]
    pub fn full(radius: Float) -> Sphere {
        Sphere::new(radius, -radius, radius, float::pi() * (2.0 as Float))
    }

    /// returns the local space bounding box
    #[inline]
    pub fn bounding(&self) -> BBox3f {
        BBox3f::new(
            Point3f::new(-self.radius, -self.radius, self.zmin),
            Point3f::new(self.radius, self.radius, self.zmax)
        )
    }

    // /// test intersection in local frame, returns `t` when first hit
    // #[inline]
    // pub fn intersect_ray(&self, ray: &RawRay) -> Option<Float>
    // {
    //     if let Some(t) = Sphere::intersect_ray_full(self.radius, ray) {
    //         let p = ray.evaluate(t);
    //         // TODO: refine sphere intersection
    //         let mut phi = p.y.atan2(p.x);
    //         if phi < (0.0 as Float) { phi += (2.0 as Float) * float::pi(); }
    //         if p.z < self.zmin || p.z > self.zmax || phi > self.phimax {
    //             None
    //         } else {
    //             Some(t)
    //         }
    //     } else {
    //         None
    //     }
    // }

    /// test intersection against the full sphere
    pub fn intersect_ray_full(radius: Float, ray: &RawRay) -> Option<Float>
    {
        let origin = ray.origin().to_vec();
        let direction = ray.direction();
        let a = direction.magnitude2();
        let b = (direction.mul_element_wise(origin) * (2.0 as Float)).sum();
        let c = origin.magnitude2() - radius * radius;

        let delta = b* b - (4.0 as Float) * a * c;
        if delta < (0.0 as Float) { return None; }
        let invert_2a = (1.0 as Float) / ((2.0 as Float) * a);
        let d1 = delta.sqrt() * invert_2a;
        let d0 = -b * invert_2a;

        let(t0, t1) = if invert_2a > 0.0 as Float {
            (d0-d1, d0+d1)
        } else {
            (d0+d1, d0-d1)
        };
        let tmax = ray.max_extend();
        if t0 > tmax || t1 < (0.0 as Float) { return None; }
        if t0 > (0.0 as Float) {
            Some(t0)
        } else if t1 > tmax {
            None
        } else {
            Some(t1)
        }
    }
}

impl Shape for Sphere {
    #[inline]
    fn bbox_local(&self) -> BBox3f {
        self.bounding()
    }

    #[inline]
    fn intersect_ray(&self, ray: &RawRay) -> Option<(Float, SurfaceInteraction)> {
        if let Some(t) = Sphere::intersect_ray_full(self.radius, &ray) {
            let mut p = ray.evaluate(t).to_vec();
            // refine sphere intersection
            p = p* self.radius / p.magnitude();
            if p.x == 0.0 as Float && p.y == 0.0 as Float {
                p.x = 1e-5 as Float * self.radius;
            }
            let p = Point3f::from_vec(p);

            let mut phi = p.y.atan2(p.x);
            if phi < (0.0 as Float) { phi += (2.0 as Float) * float::pi(); }

            // TODO: refine test against clipping
            if p.z < self.zmin || p.z > self.zmax || phi > self.phimax {
                None
            } else {
                let phimax = self.phimax;
                let thetamax = self.thetamax;
                let thetamin = self.thetamin;
                let thetadelta = thetamax - thetamin;
                let u = phi / phimax;
                let theta = (p.z / self.radius).acos();
                let v = (theta - thetamin) / thetadelta;
                let inv_z_radius = (1.0 as Float) / (p.x * p.x + p.y * p.y).sqrt();
                let cos_phi = p.x * inv_z_radius;
                let sin_phi = p.y * inv_z_radius;
                let dpdu = Vector3f::new(-phimax * p.y, phimax * p.x, 0.0 as Float);
                let dpdv = thetadelta * Vector3f::new(p.z * cos_phi, p.z * sin_phi, -self.radius * theta.sin());
                let (dndu, dndv) = {
                    let dppduu = - phimax * phimax * Vector3f::new(p.x, p.y, 0.0 as Float);
                    let dppduv = thetadelta * p.z * phimax * Vector3f::new(-sin_phi, cos_phi, 0.0 as Float);
                    let dppdvv = -thetadelta * thetadelta * Vector3f::new(p.x, p.y, p.z);

                    let e = dpdu.dot(dpdu);
                    let f = dpdu.dot(dpdv);
                    let g = dpdv.dot(dpdv);
                    let n = dpdu.cross(dpdv).normalize();
                    let ee = n.dot(dppduu);
                    let ff = n.dot(dppduv);
                    let gg = n.dot(dppdvv);
                    let inv = (1.0 as Float) / (e * g - f * f);
                    (
                        (ff*f - ee*g) * inv * dpdu + (ee*f - ff*e) * inv * dpdv,
                        (gg*f - ff*g) * inv * dpdu + (ff*f - gg*e) * inv * dpdv
                    )
                };
                Some((
                    t, SurfaceInteraction::new(
                        p, -ray.direction(), Point2f::new(u, v),
                        DuvInfo{
                            dpdu: dpdu,
                            dpdv: dpdv,
                            dndu: dndu,
                            dndv: dndv,
                        },
                    )
                ))
            }
        } else {
            None
        }
        
    }

    #[inline]
    fn surface_area(&self) -> Float {
        self.phimax * self.radius * (self.zmax - self.zmin)
    }

    fn sample(&self, sample: Point2f) -> (Point3f, Vector3f, Float) {
        // sample.x scaled to [0, phimax]
        let phi = sample.x * self.phimax;
        // sample.y scaled to [thetamin, thetamax]
        let theta = sample.y * (self.thetamax - self.thetamin) + self.thetamin;
        let dir = Sphericalf::new(theta, phi).to_vec();
        let pos = Point3f::from_vec(dir*self.radius);
        (pos, dir, 1. as Float / self.surface_area())

        // use sample::sample_uniform_sphere;
        // let dir = sample_uniform_sphere(sample);
        // let pos = Point3f::from_vec(dir*self.radius);
        // (pos, dir, 1. as Float / self.surface_area())
    }
}
