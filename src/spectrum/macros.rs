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
