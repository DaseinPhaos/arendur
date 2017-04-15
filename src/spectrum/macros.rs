// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! delegate_impl_op {
    (@both
        $Trait: ident<$Scalar: ty>,
        $tmethod: ident,
        $withmethod: ident
        for
        $Type: ident
    ) => {
        impl ops::$Trait<$Scalar> for $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Scalar) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs)
                }
            }
        }

        impl<'a> ops::$Trait<$Scalar> for &'a $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Scalar) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs)
                }
            }
        }

        impl<'a> ops::$Trait<&'a $Type> for $Scalar {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: &'a $Type) -> $Type {
                $Type{
                    inner: self.$withmethod(rhs.inner)
                }
            }
        }

        impl<'a, 'b> ops::$Trait<$Type> for $Scalar {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Type) -> $Type {
                $Type{
                    inner: self.$withmethod(rhs.inner)
                }
            }
        }
    };
    (@assign
        $Trait: ident,
        $tmethod: ident,
        $withmethod: ident
        for
        $Type: ident
    ) => {
        impl ops::$Trait for $Type {
            fn $tmethod(&mut self, rhs: $Type) {
                self.inner.$withmethod(rhs.inner)
            }
        }
        impl<'a> ops::$Trait<&'a $Type> for $Type {
            fn $tmethod(&mut self, rhs: &'a $Type) {
                self.inner.$withmethod(rhs.inner)
            }
        }
    };
    (@assign
        $Trait: ident<$Rhs: ty>,
        $tmethod: ident,
        $withmethod: ident
        for
        $Type: ident
    ) => {
        impl ops::$Trait<$Rhs> for $Type {
            fn $tmethod(&mut self, rhs: $Rhs) {
                self.inner.$withmethod(rhs)
            }
        }
    };
    (
        $Trait: ident,
        $tmethod: ident,
        $withmethod: ident
        for
        $Type: ident
    ) => {
        impl ops::$Trait<$Type> for $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Type) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs.inner)
                }
            }
        }

        impl<'a> ops::$Trait<$Type> for &'a $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Type) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs.inner)
                }
            }
        }

        impl<'a> ops::$Trait<&'a $Type> for $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: &'a $Type) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs.inner)
                }
            }
        }

        impl<'a, 'b> ops::$Trait<&'a $Type> for &'b $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: &'a $Type) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs.inner)
                }
            }
        }
    };
    (
        $Trait: ident<$Scalar: ty>,
        $tmethod: ident,
        $withmethod: ident
        for
        $Type: ident
    ) => {
        impl ops::$Trait<$Scalar> for $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Scalar) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs)
                }
            }
        }

        impl<'a> ops::$Trait<$Scalar> for &'a $Type {
            type Output = $Type;
            #[inline]
            fn $tmethod(self, rhs: $Scalar) -> $Type {
                $Type{
                    inner: self.inner.$withmethod(rhs)
                }
            }
        }
    }
}

macro_rules! delegate_impl_to_norm {
    ($Type: ident) => {
        impl ToNorm for $Type {
            #[inline]
            fn to_norm(self) -> Float {
                <Float as NumCast>::from(self).unwrap() / <Float as NumCast>::from(std::$Type::MAX).unwrap()
            }

            #[inline]
            fn from_norm(mut f: Float) -> Self {
                f = float::clamp(f, 0.0 as Float, 1.0 as Float);
                <Self as NumCast>::from(f * <Float as NumCast>::from(std::$Type::MAX).unwrap()).unwrap()
            }
        }
    }
}