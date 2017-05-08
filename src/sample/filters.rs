// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines some commonly used filters

use geometry::prelude::*;
use super::Filter;
use std::mem;

// /// Commonly used filter info
// #[derive(Copy, Clone, Debug)]
// struct FilterInfo {
//     radius: Vector2f,
//     inv_radius: Vector2f,
// }

// impl FilterInfo {
//     /// construction
//     fn new(radius: Vector2f) -> FilterInfo {
//         assert!(radius.x > 0.0 as Float);
//         assert!(radius.y > 0.0 as Float);
//         let inv_radius = 1.0 as Float / radius;
//         FilterInfo {
//             radius: radius,
//             inv_radius: inv_radius,
//         }
//     }
// }

/// A box filter!
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct BoxFilter {
    radius: Vector2f,
}

impl BoxFilter {
    /// Construction!
    pub fn new(radius: Vector2f) -> BoxFilter {
        assert!(radius.x > 0.0 as Float);
        assert!(radius.y > 0.0 as Float);
        BoxFilter {
            radius: radius
        }
    }
}

impl Filter for BoxFilter {
    fn radius(&self) -> Vector2f {
        self.radius
    }

    unsafe fn evaluate_unsafe(&self, _p: Point2f) -> Float {
        1.0 as Float
    }
}

/// A triangle filter!
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct TriangleFilter {
    radius: Vector2f,
}

impl TriangleFilter {
    /// Construction!
    pub fn new(radius: Vector2f) -> TriangleFilter {
        assert!(radius.x > 0.0 as Float);
        assert!(radius.y > 0.0 as Float);
        TriangleFilter {
            radius: radius
        }
    }
}

impl Filter for TriangleFilter {
    fn radius(&self) -> Vector2f {
        self.radius
    }

    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float {
        (self.radius.x - p.x.abs()) * (self.radius.y - p.y.abs())
    }
}

/// A Gausssian filter!
/// 1D Gaussian's filter function is given 
/// by $f(x) = e^{-\alpha\times x^2} - e^{-\alpha\times r^2}$
/// $\alpha$ controls the rate of fall-off.
/// Smaller value gives slower fall off.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct GaussianFilter {
    radius: Vector2f,
    // precompute `$e^{-\alpha\times r^2}$ for efficiency
    exp: Vector2f,
    neg_alpha: Float,
}

impl GaussianFilter {
    /// Construction!
    pub fn new(alpha: Float, radius: Vector2f) -> GaussianFilter {
        assert!(radius.x > 0.0 as Float);
        assert!(radius.y > 0.0 as Float);
        let neg_alpha = -alpha;
        let exp = Vector2f::new(
            neg_alpha * radius.x * radius.x,
            neg_alpha * radius.y * radius.y
        );
        GaussianFilter{
            radius: radius,
            exp: exp,
            neg_alpha: neg_alpha,
        }
    }
}

impl Filter for GaussianFilter {
    fn radius(&self) -> Vector2f {
        self.radius
    }

    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float {
        let gx = (self.neg_alpha * p.x * p.x).exp() - self.exp.x;
        let gy = (self.neg_alpha * p.y * p.y).exp() - self.exp.y;
        gx * gy
    }
}

/// Mitchell filter as per Mitchell-Netravali [1988]
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct MitchellFilter {
    radius: Vector2f,
    inv_radius: Vector2f,
    b: Float,
    c: Float,
}

impl MitchellFilter {
    /// construction
    /// `b` and `c` can be any value, but lying along
    /// `$b+2c=1$ might provides optimal result
    pub fn new(radius: Vector2f, b: Float, c: Float) -> MitchellFilter {
        assert!(radius.x > 0.0 as Float);
        assert!(radius.y > 0.0 as Float);
        let inv_radius = 1.0 as Float / radius;
        MitchellFilter {
            radius: radius,
            inv_radius: inv_radius,
            b: b,
            c: c,
        }
    }

    /// compute 1d mitchell filter value given by the original proposal
    fn mitchell_1d(x: Float, b: Float, c: Float) -> Float {
        debug_assert!(x>=-0.0001 as Float);
        debug_assert!(x<=2.0001 as Float);
        const INV_SIX: Float = 1.0 as Float / 6.0 as Float;
        if x > 1.0 as Float {
            (-b - 6.0 as Float * c) * x * x * x
            + (6.0 as Float * b + 30.0 as Float * c) * x * x
            - (12.0 as Float * b + 48.0 as Float * c) * x
            + (8.0 as Float * b + 24.0 as Float * c) * INV_SIX
        } else {
            (12.0 as Float - 9.0 as Float * b - 6.0 as Float * c) * x * x * x
            + ( -18.0 as Float - 12.0 as Float * b + 6.0 as Float * c) * x * x
            + (6.0 as Float - 2.0 as Float * b) * INV_SIX
        }
    }
}

impl Filter for MitchellFilter {
    #[inline]
    fn radius(&self) -> Vector2f {
        self.radius
    }

    #[inline]
    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float {
        let mp = 2.0 as Float * self.inv_radius.mul_element_wise(p.to_vec());
        MitchellFilter::mitchell_1d(mp.x.abs(), self.b, self.c)
        * MitchellFilter::mitchell_1d(mp.y.abs(), self.b, self.c)
    }
}

/// A windowed sinc filter as per [Lanczos](https://en.wikipedia.org/wiki/Lanczos_resampling).
/// `tau` controls how many circles the function passes through
/// before clamping to zero.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct LanczosSincFilter {
    radius: Vector2f,
    inv_tau: Float,
}

impl LanczosSincFilter {
    /// Construction
    #[inline]
    pub fn new(radius: Vector2f, tau: Float) -> LanczosSincFilter {
        assert!(radius.x > 0.0 as Float);
        assert!(radius.y > 0.0 as Float);
        assert!(tau > 0.0 as Float);
        LanczosSincFilter{
            radius: radius,
            inv_tau: 1.0 as Float / tau,
        }
    }

    /// evaluate lanczos sinc filter given by
    /// $f(x) = sinc(x/tau) * sinc(x)$
    /// x should be greater than zero
    #[inline]
    fn lanczos_sinc(x: Float, inv_tau: Float) -> Float {
        LanczosSincFilter::sinc((x*inv_tau))
        * LanczosSincFilter::sinc(x)
    }

    /// x should be greater than zero
    #[inline]
    fn sinc(x: Float) -> Float {
        if x < 1.0e-5 as Float {
            1.0 as Float
        } else {
            let xpi = x * float::pi();
            xpi.sin() / xpi
        }
    }
}

impl Filter for LanczosSincFilter {
    fn radius(&self) -> Vector2f {
        self.radius
    }

    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float {
        LanczosSincFilter::lanczos_sinc(p.x, self.inv_tau)
        * LanczosSincFilter::lanczos_sinc(p.y, self.inv_tau)
    }
}


const PREC_FILTER_WIDTH: usize = 16;
const PREC_FILTER_SIZE: usize = PREC_FILTER_WIDTH * PREC_FILTER_WIDTH;

/// To help accelerate the sampling process, sometimes it might
/// be desirable to precompute the filtered value at some
/// discrete locations and look'em up at runtime. This struct
/// provides such functionality.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrecomputedFilter {
    // buf: [Float; PREC_FILTER_SIZE],
    buf: Vec<Float>,
    radius: Vector2f,
    mulx: Float,
    muly: Float,
}

impl PrecomputedFilter {
    /// construction
    pub fn new<F: Filter>(from: &F) -> Self {
        let radius = from.radius();
        const INV_PREC_FILTER_WIDTH: Float = 1.0 as Float / PREC_FILTER_WIDTH as Float;
        let dp = radius * INV_PREC_FILTER_WIDTH;
        let mut ret = PrecomputedFilter {
            // buf: unsafe { mem::uninitialized() },
            buf: unsafe { vec![mem::uninitialized(); PREC_FILTER_SIZE] },
            radius: radius,
            mulx: PREC_FILTER_WIDTH as Float / radius.x,
            muly: PREC_FILTER_WIDTH as Float / radius.y,
        };
        for y in 0..PREC_FILTER_WIDTH {
            let py = dp.y * (y as Float + 0.5 as Float);
            for x in 0..PREC_FILTER_WIDTH {
                let px = dp.x * (x as Float + 0.5 as Float);
                unsafe{
                    *ret.buf.get_unchecked_mut(PrecomputedFilter::index_at(x, y)) = from.evaluate_unsafe(Point2f::new(px, py));
                }
            }
        }
        ret
    }

    #[inline]
    fn index_at(x: usize, y: usize) -> usize {
        x*PREC_FILTER_WIDTH + y
    }

    #[inline]
    unsafe fn index_at_p(&self, p: Point2f) -> usize {
        let px = (p.x.abs() * self.mulx) as usize;
        let py = (p.y.abs() * self.muly) as usize;
        Self::index_at(px, py)
    }
}

impl Filter for PrecomputedFilter {
    #[inline]
    fn radius(&self) -> Vector2f {
        self.radius
    }

    #[inline]
    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float {
        debug_assert!(p.x <= self.radius.x);
        debug_assert!(p.y <= self.radius.y);
        *self.buf.get_unchecked(self.index_at_p(p))
    }
}
