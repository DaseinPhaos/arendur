use num_traits;
use super::cgmath_prelude::*;

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
pub fn ln_2() -> Float {
    <Float as num_traits::FloatConst>::LN_2()
}

#[inline]
pub fn sqrt_2() -> Float {
    <Float as num_traits::FloatConst>::SQRT_2()
}