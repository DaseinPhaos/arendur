// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Floating point number helper functions
use num_traits;
use super::foundamental::*;

#[inline]
pub fn clamp(f: Float, min: Float, max: Float) -> Float {
    debug_assert!(min <= max);
    if f < min { min }
    else if f < max { f }
    else { max }
}

#[inline]
pub fn epsilon() -> Float {
    <Float as num_traits::Float>::epsilon()
}

#[inline]
pub fn machine_epsilon() -> Float {
    <Float as num_traits::Float>::epsilon() * 0.5 as Float
}

#[inline]
/// error bound term given by Higham(2002)
pub fn eb_term(n: Float) -> Float {
    let ne = n * machine_epsilon();
    ne / (1. as Float - ne)
}

#[inline]
pub fn one_minus_epsilon() -> Float {
    1.0 as Float - epsilon()
}

#[inline]
pub fn nan() -> Float {
    <Float as num_traits::Float>::nan()
}

#[inline]
pub fn infinity() -> Float {
    <Float as num_traits::Float>::infinity()
}

#[inline]
pub fn neg_infinity() -> Float {
    <Float as num_traits::Float>::neg_infinity()
}

#[inline]
pub fn frac_1_pi() -> Float {
    <Float as num_traits::FloatConst>::FRAC_1_PI()
}

#[inline]
pub fn frac_2_pi() -> Float {
    <Float as num_traits::FloatConst>::FRAC_2_PI()
}

#[inline]
pub fn frac_2_sqrt_pi() -> Float {
    <Float as num_traits::FloatConst>::FRAC_2_SQRT_PI()
}

#[inline]
pub fn frac_pi_2() -> Float {
    <Float as num_traits::FloatConst>::FRAC_PI_2()
}

#[inline]
pub fn frac_pi_3() -> Float {
    <Float as num_traits::FloatConst>::FRAC_PI_3()
}

#[inline]
pub fn frac_pi_4() -> Float {
    <Float as num_traits::FloatConst>::FRAC_PI_4()
}

#[inline]
pub fn frac_pi_6() -> Float {
    <Float as num_traits::FloatConst>::FRAC_PI_6()
}

#[inline]
pub fn frac_pi_8() -> Float {
    <Float as num_traits::FloatConst>::FRAC_PI_8()
}

#[inline]
pub fn pi() -> Float {
    <Float as num_traits::FloatConst>::PI()
}

#[inline]
pub fn next_up(f: Float) -> Float {
    if f.is_infinite() && f.is_sign_positive() {
        f
    } else if f == -0. as Float {
        0. as Float
    } else {
        let t = f.to_bits();
        if f.is_sign_positive() {
            Float::from_bits(t+1)
        } else {
            Float::from_bits(t-1)
        }
    }
}

#[inline]
pub fn next_down(f: Float) -> Float {
    if f.is_infinite() && f.is_sign_negative() {
        f
    } else if f == 0. as Float {
        -0. as Float
    } else {
        let t = f.to_bits();
        if f.is_sign_negative() {
            Float::from_bits(t+1)
        } else {
            Float::from_bits(t-1)
        }
    }
}

