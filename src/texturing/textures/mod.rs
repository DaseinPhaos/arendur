// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Commonly used implementations of `Texture`.

use super::*;
use std::ops;

/// A constant texture
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConstantTexture<T> {
    pub value: T,
}

impl<T: Clone + Send + Sync> Texture for ConstantTexture<T> {
    type Texel = T;

    #[inline]
    fn evaluate(&self, _si: &SurfaceInteraction, _dxy: &DxyInfo) -> T {
        self.value.clone()
    }

    #[inline]
    fn mean(&self) -> T {
        self.value.clone()
    }
}

/// Texture adapter that takes two textures and returns the product of their values
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProductTexture<T0, T1> {
    pub t0: T0,
    pub t1: T1,
}

impl<T0: Send + Sync, T1: Send + Sync> Texture for ProductTexture<T0, T1>
    where T0: Texture,
          T1: Texture,
          T0::Texel: ops::Mul<T1::Texel>,
{
    type Texel = <T0::Texel as ops::Mul<T1::Texel>>::Output;
    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        self.t0.evaluate(si, dxy) * self.t1.evaluate(si, dxy)
    }

    #[inline]
    // TODO: inappropriate. fix this
    fn mean(&self) -> Self::Texel {
        self.t0.mean() * self.t1.mean()
    }
}

/// Texture adapter that takes two textures, and an additional `Float` texture,
/// and returns lerping between them
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixTexture<T0, T1, L> {
    pub t0: T0,
    pub t1: T1,
    pub l: L,
}

impl<T0: Send + Sync, T1: Send + Sync, L: Send + Sync> Texture for MixTexture<T0, T1, L>
    where T0: Texture,
          T1: Texture,
          L: Texture<Texel=Float>,
          T0::Texel: ops::Mul<Float>,
          T1::Texel: ops::Mul<Float>,
          <T0::Texel as ops::Mul<Float>>::Output: ops::Add<<T1::Texel as ops::Mul<Float>>::Output>,
{
    type Texel = <<T0::Texel as ops::Mul<Float>>::Output as ops::Add<<T1::Texel as ops::Mul<Float>>::Output>>::Output;

    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        let lerp = self.l.evaluate(si, dxy);
        let t0l = self.t0.evaluate(si, dxy) * (1.0 as Float - lerp);
        let t1l = self.t1.evaluate(si, dxy) * lerp;
        t0l + t1l
    }

    #[inline]
    // TODO: inappropriate. fix this
    fn mean(&self) -> Self::Texel {
        let lerp = self.l.mean();
        let t0l = self.t0.mean() * (1.0 as Float - lerp);
        let t1l = self.t1.mean() * lerp;
        t0l + t1l
    }
}

pub mod image;
