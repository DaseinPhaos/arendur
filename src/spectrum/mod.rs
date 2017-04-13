//! Defines spectral representations
use geometry::*;
use std::ops;

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

/// An specturm represented in SRGB
#[derive(Copy, Clone, PartialEq)]
pub struct RGBSpectrum<T: BaseNum> {
    pub inner: Vector3<T>,
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






