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