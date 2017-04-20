// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The sampling and filtering interface

// TODO: add more samplers

use geometry::prelude::*;
use filming;

/// The sampling interface.
/// Samplers should return sampled values in $[0, 1)$.
///
/// Additional information are provided through the interface
/// (like pixel location, dimension, samples per pixel etc.)
/// such that implementations might provide better-quality.
pub trait Sampler: Clone + Sync + Send
{
    /// Start sampling a new pixel
    fn start_pixel(&mut self, p: Point2<u32>);

    /// get next 1-dimensional sample
    fn next(&mut self) -> Float;

    /// get next 2-dimensional sample
    #[inline]
    fn next_2d(&mut self) -> Point2f {
        Point2f::new(self.next(), self.next())
    }

    /// convinient method to sample a camera
    #[inline]
    fn get_camera_sample(&mut self, idx: Point2<u32>) -> filming::SampleInfo {
        filming::SampleInfo{
            pfilm: self.next_2d() + idx.cast().to_vec(),
            plens: self.next_2d(),
        }
    }

    /// request `n` samples in place
    /// Default implementation uses succeeding calls to `self.next()`
    /// to fill the `buf`, which might not be ideal
    fn request(&mut self, buf: &mut [Float]) {
        for f in buf.iter_mut() {
            *f = self.next();
        }
    }

    /// request `n` 2d samples in place
    /// Default implementation uses succeeding calls to `self.next_2d()`
    /// to fill the `buf`, which might not be ideal
    fn request_2d(&mut self, buf: &mut [Point2f]) {
        for p in buf.iter_mut() {
            *p = self.next_2d();
        }
    }

    // /// request the prequest, `n` to checksum.
    // fn request(&mut self, n: u32) -> &[Float];

    // /// request the prequest, `n` to checksum
    // fn request(&mut self, n: u32) -> &[Point2f];

    /// Optimal round count, as a hint
    #[inline]
    fn round_count(&self, n: u32) -> u32 {
        n
    }

    /// maximum sample count per pixel
    fn sample_per_pixel(&self) -> usize;

    /// Try to advance to the next sample
    /// `true` if the sampling process can continue
    /// `false` when overflowing `sample_per_pixel`, eg
    fn next_sample(&mut self) -> bool;

    /// try to set current sample to a particular index
    fn set_sample_index(&mut self, idx: usize) -> bool;
}

/// The filter interface.
/// A filter always lies at $(0, 0)$ in its local frame.
pub trait Filter: Send + Sync {
    /// Returns the filter's radius.
    /// The filter's support in local frame is thus given
    /// by $[-radius.x, radius.x]\times [-radius.y, radius.y]$
    fn radius(&self) -> Vector2f;

    /// Returns the filter's support as a bounding box, in local frame.
    #[inline]
    fn support(&self) -> BBox2f {
        let p = self.radius();
        BBox2f::new(Point2f::from_vec(p), Point2f::from_vec(-p))
    }

    /// Evaluate the filter at `p` in its local frame.
    /// Caller MUST ensure that `p` lies inside the support of `self`.
    /// This method is thus marked as `unsafe`.
    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float;

    /// Evaluate the filter at `p` in its local frame.
    /// Point outside support is checked.
    #[inline]
    fn evaluate(&self, p: Point2f) -> Float {
        let bounding = self.support();
        if bounding.contain(p) {unsafe {
            self.evaluate_unsafe(p)
        }} else {
            0.0 as Float
        }
    }
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into uniform samples on a hemisphere
#[inline]
pub fn sample_uniform_hemisphere(u: Point2f) -> Vector3f {
    let costheta = u.x;
    let sintheta = (1.0 as Float - costheta).max(0.0 as Float).sqrt();
    let phi = 2.0 as Float * float::pi() * u.y;
    Vector3f::new(sintheta*phi.cos(), sintheta*phi.sin(), costheta)
}

/// pdf of uniform samples on a hemisphere
#[inline]
pub fn pdf_uniform_hemisphere() -> Float {
    0.5 as Float * float::frac_1_pi()
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into uniform samples on a sphere
#[inline]
pub fn sample_uniform_sphere(u: Point2f) -> Vector3f {
    let costheta = 1.0 as Float - 2.0 as Float * u.x;
    let sintheta = (1.0 as Float - costheta).max(0.0 as Float).sqrt();
    let phi = 2.0 as Float * float::pi() * u.y;
    Vector3f::new(sintheta*phi.cos(), sintheta*phi.sin(), costheta)
}

/// pdf of uniform samples on a hemisphere
#[inline]
pub fn pdf_uniform_sphere() -> Float {
    0.25 as Float * float::frac_1_pi()
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into concentric samples on a disk, preserving relative
/// distributions
#[inline]
pub fn sample_concentric_disk(u: Point2f) -> Point2f {
    let u = (2.0 as Float * u) - Point2f::new(1.0 as Float, 1.0 as Float);
    if u.x == 0.0 as Float && u.y == 0.0 as Float {
        Point2f::new(0.0 as Float, 0.0 as Float)
    } else {
        let (r, theta) = if u.x.abs() > u.y.abs() {
            (u.x, float::frac_pi_4() * (u.y/u.x))
        } else {
            (u.y, float::frac_pi_2() - float::frac_pi_4() * (u.x/u.y))
        };
        r * Point2f::new(theta.cos(), theta.sin())
    }
}

/// pdf of concentric samples on a disk
#[inline]
pub fn pdf_concentric_disk() -> Float {
    float::pi()
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into uniform samples on a disk
#[inline]
pub fn sample_uniform_disk(u: Point2f) -> Point2f {
    let r = u.x.sqrt();
    let theta = 2.0 as Float * float::pi() * u.y;
    Point2f::new(r*theta.cos(), r*theta.sin())
}

/// pdf of uniform samples on a disk
#[inline]
pub fn pdf_uniform_disk() -> Float {
    float::pi()
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into cosine-theta weighted samples on a hemisphere
#[inline]
pub fn sample_cosw_hemisphere(u: Point2f) -> Vector3f {
    let d = sample_concentric_disk(u);
    let z = (1.0 as Float - d.x*d.x - d.y*d.y).sqrt();
    Vector3f::new(d.x, d.y, z)
}

/// pdf of cosine-theta weighted samples on a hemisphere
#[inline]
pub fn pdf_cosw_hemisphere(cos_theta: Float) -> Float {
    cos_theta * float::frac_1_pi()
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into uniform samples on a cone
#[inline]
pub fn sample_uniform_cone(u: Point2f, cos_max: Float) -> Vector3f {
    let costheta = (1.0 as Float - u.x) + u.x * cos_max;
    let sintheta = (1.0 as Float - costheta*costheta).sqrt();
    let phi = u.y * (2.0 as Float * float::pi());
    Vector3f::new(sintheta*phi.cos(), sintheta*phi.sin(), costheta)
}

/// pdf of uniform samples on a cone
#[inline]
pub fn pdf_uniform_cone(cos_max: Float) -> Float {
    1.0 as Float / ((1.0 as Float - cos_max) * 2.0 as Float * float::pi())
}

/// transform an uniformly sampled `u` in $[0,1)^2$
/// into uniform samples on a triangle's barycentric coordinates
#[inline]
pub fn sample_uniform_triangle(u: Point2f) -> Vector3f {
    let sqrtux = u.x.sqrt();
    let x = 1.0 as Float - sqrtux;
    let y = sqrtux * u.y;
    Vector3f::new(x, y, 1.0 as Float - x - y)
}

/// power heuristic as per
#[inline]
pub fn power_heuristic(nf: usize, pdff: Float, ng: usize, pdfg: Float) -> Float {
    let f = nf as Float * pdff;
    let g = ng as Float * pdfg;
    (f*f)/(f*f+g*g)
}

pub mod naive;
pub mod strata;
pub mod filters;
pub mod prelude;
mod sink;
