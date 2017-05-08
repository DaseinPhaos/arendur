// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines spectral representations
use geometry::prelude::*;
use std;
use std::ops;
use std::mem;
use num_traits::cast::NumCast;
use image;

pub type RGBSpectrumf = RGBSpectrum<Float>;

/// Represents a spectrum
pub trait Spectrum
    where Self: Sized
{
    type Scalar: PartialOrd + BaseNum;
    /// initialize to unified color
    fn grey_scale(n: Self::Scalar) -> Self;

    /// initialize to black
    #[inline]
    fn black() -> Self {
        <Self as Spectrum>::grey_scale(<Self::Scalar as Zero>::zero())
    }

    /// lerp
    fn lerp(&self, other: &Self, t: Float) -> Self;

    /// element-wise clamping
    fn clamp(&self, low: Self::Scalar, high: Self::Scalar) -> Self;

    /// convert to srgb
    fn to_srgb(&self) -> RGBSpectrum<Self::Scalar>;

    /// convert to XYZ
    #[inline]
    fn to_xyz(&self) -> Vector3f {
        let srgb = self.to_srgb();
        let srgb = RGBSpectrumf{
            inner: srgb.inner.cast()
        };
        srgb.to_xyz()
    }
}

/// An spectrum represented in SRGB
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RGBSpectrum<T: BaseNum> {
    pub inner: Vector3<T>,
}

impl<T: BaseNum> Default for RGBSpectrum<T> {
    #[inline]
    fn default() -> Self {
        RGBSpectrum::new(T::zero(), T::zero(), T::zero())
    }
}

impl<T: BaseNum> RGBSpectrum<T> {
    #[inline]
    pub fn new(r: T, g: T, b: T) -> Self {
        RGBSpectrum{
            inner: Vector3::new(r, g, b)
        }
    }
    
    #[inline]
    pub fn r(&self) -> T {
        self.inner.x
    }

    #[inline]
    pub fn g(&self) -> T {
        self.inner.y
    }

    #[inline]
    pub fn b(&self) -> T {
        self.inner.z
    }
}

impl<T: ToNorm + BaseNum> RGBSpectrum<T> {
    #[inline]
    pub fn to_rgbf(self) -> RGBSpectrumf {
        RGBSpectrumf::new(self.inner.x.to_norm(), self.inner.y.to_norm(), self.inner.z.to_norm())
    }

    #[inline]
    pub fn from_rgbf(v: RGBSpectrumf) -> Self {
        RGBSpectrum::new(T::from_norm(v.inner.x), T::from_norm(v.inner.y), T::from_norm(v.inner.z))
    }

    /// lerp
    #[inline]
    pub fn approx_lerp(self, other: Self, t: Float) -> Self {
        let lhs = self.to_rgbf();
        let rhs = other.to_rgbf();
        let inner = <Vector3f as InnerSpace>::lerp(lhs.inner, rhs.inner, t);
        Self::from_rgbf(RGBSpectrumf{
            inner: inner
        })
    }

    #[inline]
    pub fn is_black(self) -> bool {
        self.inner == Vector3::zero()
    }
}

impl<T: BaseNum + image::Primitive> image::Pixel for RGBSpectrum<T> {
    type Subpixel = T;

    #[inline]
    fn channel_count() -> u8 {
        3u8
    }

    #[inline]
    fn channels(&self) -> &[T] {
        let t: &[T; 3] = self.inner.as_ref();
        t
    }

    #[inline]
    fn channels_mut(&mut self) -> &mut [T] {
        let t: &mut [T; 3] = self.inner.as_mut();
        t
    }

    #[inline]
    fn color_model() -> &'static str {
        "RGB"
    }

    #[inline]
    fn color_type() -> image::ColorType {
        image::ColorType::RGB(mem::size_of::<T>() as u8 * 8)
    }

    #[inline]
    fn channels4(&self) -> (T, T, T, T) {
        (self.inner.x, self.inner.y, self.inner.z, <T as One>::one())
    }

    #[inline]
    fn from_channels(a: T, b: T, c: T, d: T) -> Self {
        RGBSpectrum::new(a*d, b*d, c*d)
    }

    #[inline]
    fn from_slice<'a>(slice: &'a [T]) -> &'a Self {
        unsafe {
            let ptr: *const _ = mem::transmute(slice.as_ptr());
            ptr.as_ref().unwrap()
        }
    }

    #[inline]
    fn from_slice_mut<'a>(slice: &'a mut [Self::Subpixel]) -> &'a mut Self {
        unsafe {
            let ptr: *mut _ = mem::transmute(slice.as_mut_ptr());
            ptr.as_mut().unwrap()
        }
    }

    #[inline]
    fn to_rgb(&self) -> image::Rgb<T> {
        image::Rgb{
            data: [self.inner.x, self.inner.y, self.inner.z],
        }
    }

    #[inline]
    fn to_rgba(&self) -> image::Rgba<T> {
        image::Rgba{
            data: [self.inner.x, self.inner.y, self.inner.z, <T as One>::one()],
        }
    }

    #[inline]
    fn to_luma(&self) -> image::Luma<Self::Subpixel> {
        let r = RGBSpectrumf{
            inner: self.inner.cast()
        };
        image::Luma{
            data: [<T as NumCast>::from(r.to_xyz().y).unwrap()]
        }
    }

    #[inline]
    fn to_luma_alpha(&self) -> image::LumaA<Self::Subpixel> {
        let r = RGBSpectrumf{
            inner: self.inner.cast()
        };
        image::LumaA{
            data: [<T as NumCast>::from(r.to_xyz().y).unwrap(), <T as One>::one()]
        }
    }

    #[inline]
    fn map<F>(&self, f: F) -> Self 
        where F: Fn(Self::Subpixel) -> Self::Subpixel
    {
        RGBSpectrum::new(
            f(self.r()), f(self.g()), f(self.b())
        )
    }

    #[inline]
    fn apply<F>(&mut self, f: F) 
        where F: Fn(Self::Subpixel) -> Self::Subpixel
    {
        self.inner.x = f(self.inner.x);
        self.inner.y = f(self.inner.y);
        self.inner.z = f(self.inner.z);
    }

    #[inline]
    fn map_with_alpha<F, G>(&self, f: F, _g: G) -> Self 
        where F: Fn(Self::Subpixel) -> Self::Subpixel,
              G: Fn(Self::Subpixel) -> Self::Subpixel
    {
        self.map(f)
    }

    #[inline]
    fn apply_with_alpha<F, G>(&mut self, f: F, _g: G) 
        where F: Fn(Self::Subpixel) -> Self::Subpixel,
              G: Fn(Self::Subpixel) -> Self::Subpixel
    {
        self.apply(f)
    }

    #[inline]
    fn map2<F>(&self, other: &Self, f: F) -> Self 
        where F: Fn(Self::Subpixel, Self::Subpixel) -> Self::Subpixel
    {
        RGBSpectrum::new(
            f(self.r(), other.r()),
            f(self.g(), other.g()),
            f(self.b(), other.b())
        )
    }

    #[inline]
    fn apply2<F>(&mut self, other: &Self, f: F) 
        where F: Fn(Self::Subpixel, Self::Subpixel) -> Self::Subpixel
    {
        *self = RGBSpectrum::new(
            f(self.r(), other.r()),
            f(self.g(), other.g()),
            f(self.b(), other.b())
        );
    }

    #[inline]
    fn invert(&mut self) {
        self.apply(|x| <T as One>::one() / x);
    }

    #[inline]
    fn blend(&mut self, other: &Self) {
        self.inner.x *= other.inner.x;
        self.inner.y *= other.inner.y;
        self.inner.z *= other.inner.z;
    }
}

impl RGBSpectrumf {
    #[inline]
    pub fn from_xyz(xyz: Vector3f) -> RGBSpectrumf {
        RGBSpectrumf::new(
            (3.240479 as Float) * xyz.x - (1.537150 as Float) * xyz.y - (0.498535 as Float) * xyz.z,
            (-0.969256 as Float) * xyz.x + (1.875991 as Float) * xyz.y + (0.041556 as Float) * xyz.z,
            (0.055648 as Float) * xyz.x - (0.204043 as Float) * xyz.y + (1.057311 as Float) * xyz.z
        )
    }

    #[inline]
    pub fn into_xyz(self) -> Vector3f {
        Vector3f::new(
            0.412453 as Float * self.r() + 0.357580 as Float * self.g() + 0.180423 as Float * self.b(),
            0.212671 as Float * self.r() + 0.715160 as Float * self.g() + 0.072169 as Float * self.b(),
            0.019334 as Float * self.r()+  0.119193 as Float * self.g() + 0.950227 as Float * self.b()
        )
    }

    /// sqrt
    #[inline]
    pub fn sqrt(self) -> RGBSpectrumf {
        RGBSpectrumf::new(self.inner.x.sqrt(), self.inner.y.sqrt(), self.inner.z.sqrt())   
    }

    #[inline]
    pub fn valid(&self) -> bool {
        !(self.r().is_nan() || self.g().is_nan() || self.b().is_nan())
        && !(self.r().is_infinite() || self.g().is_infinite() || self.b().is_infinite())
        && self.r() >= 0. as Float &&self.g() >= 0. as Float &&self.b() >= 0. as Float
    }
}

impl Spectrum for RGBSpectrumf {
    type Scalar = Float;

    /// initialize to unified color
    #[inline]
    fn grey_scale(n: Self::Scalar) -> Self {
        RGBSpectrumf{
            inner: Vector3f::new(n, n, n)
        }
    }

    /// lerp
    #[inline]
    fn lerp(&self, other: &Self, t: Float) -> Self {
        let inner = <Vector3f as InnerSpace>::lerp(self.inner, other.inner, t);
        RGBSpectrumf{
            inner: inner
        }
    }

    /// element-wise clamping
    fn clamp(&self, low: Self::Scalar, high: Self::Scalar) -> Self {
        RGBSpectrumf::new(
            float::clamp(self.r(), low, high),
            float::clamp(self.g(), low, high),
            float::clamp(self.b(), low, high)
        )
    }

    /// convert to srgb
    fn to_srgb(&self) -> RGBSpectrum<Self::Scalar> {
        *self
    }

    /// convert to XYZ
    #[inline]
    fn to_xyz(&self) -> Vector3f {
        (*self).into_xyz()
    }
}

#[macro_use]
mod macros;

delegate_impl_op!(Add, add, add_element_wise for RGBSpectrumf);
delegate_impl_op!(Sub, sub, sub_element_wise for RGBSpectrumf);
delegate_impl_op!(Mul, mul, mul_element_wise for RGBSpectrumf);
delegate_impl_op!(Div, div, div_element_wise for RGBSpectrumf);
delegate_impl_op!(Rem, rem, rem_element_wise for RGBSpectrumf);
delegate_impl_op!(@both Mul<Float>, mul, mul for RGBSpectrumf);
delegate_impl_op!(Div<Float>, div, div for RGBSpectrumf);
delegate_impl_op!(@assign AddAssign, add_assign, add_assign_element_wise for RGBSpectrumf);
delegate_impl_op!(@assign SubAssign, sub_assign, sub_assign_element_wise for RGBSpectrumf);
delegate_impl_op!(@assign MulAssign, mul_assign, mul_assign_element_wise for RGBSpectrumf);
delegate_impl_op!(@assign DivAssign, div_assign, div_assign_element_wise for RGBSpectrumf);
delegate_impl_op!(@assign MulAssign<Float>, mul_assign, mul_assign for RGBSpectrumf);
delegate_impl_op!(@assign DivAssign<Float>, div_assign, div_assign for RGBSpectrumf);

pub trait ToNorm {
    fn to_norm(self) -> Float;
    
    fn from_norm(f: Float) -> Self;
}

impl ToNorm for Float {
    #[inline]
    fn to_norm(self) -> Float {
        debug_assert!(self>=0. as Float);
        debug_assert!(self<=1. as Float);
        self
    }

    #[inline]
    fn from_norm(f: Float) -> Self {
        debug_assert!(f>=0. as Float);
        debug_assert!(f<=1. as Float);
        f
    }
}

delegate_impl_to_norm!(u8);
delegate_impl_to_norm!(u16);
delegate_impl_to_norm!(u32);

pub mod prelude {
    pub use super::{RGBSpectrum, RGBSpectrumf, Spectrum};
}