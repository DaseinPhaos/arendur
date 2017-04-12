//! Defines spectral representations
use geometry::*;
use std::ops;

pub trait Spectrum
{
    type Scalar: PartialOrd;
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
}

pub struct RGBSpectrum<T: BaseNum> {
    pub inner: Vector3<T>,
}

pub type RGBSpectrumf = RGBSpectrum<Float>;
pub type RGBSpectrumu = RGBSpectrum<u32>;
pub type RGBSpectrumi = RGBSpectrum<i32>;

#[macro_use]
mod macros;

delegate_impl_op!(Add, add, add_element_wise for RGBSpectrum);
delegate_impl_op!(Sub, sub, sub_element_wise for RGBSpectrum);
delegate_impl_op!(Mul, mul, mul_element_wise for RGBSpectrum);
delegate_impl_op!(Div, div, div_element_wise for RGBSpectrum);
delegate_impl_op!(Rem, rem, rem_element_wise for RGBSpectrum);
delegate_impl_op!(@both Mul<Float>, mul, mul for RGBSpectrum);
delegate_impl_op!(Div<Float>, div, div for RGBSpectrum);
delegate_impl_spec!(RGBSpectrumf, Vector3f, Vector3f);






