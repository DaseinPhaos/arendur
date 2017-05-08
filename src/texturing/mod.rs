// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The texuture mapping interface

use geometry::prelude::*;
use std::sync::Arc;

/// Represents a texture information
#[derive(Copy, Clone, PartialEq)]
pub struct TexInfo2D {
    /// coordinates in texture plane
    pub p: Point2f,
    /// differentials along pixel plane
    pub dpdx: Vector2f,
    /// differentials along pixel plane
    pub dpdy: Vector2f,
}

/// Represents a texture information
#[derive(Copy, Clone, PartialEq)]
pub struct TexInfo3D {
    /// coordinates in texture plane
    pub p: Point3f,
    /// differentials along pixel plane
    pub dpdx: Vector3f,
    /// differentials along pixel plane
    pub dpdy: Vector3f,
}

/// 2D texture mapping interface
pub trait Mapping2D {
    /// given a surface interaction and its dxyinfo, compute the texture info
    fn map(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> TexInfo2D;
}

/// 3D texture mapping interface
pub trait Mapping3D {
    /// given a surface interaction and its dxyinfo, compute the texture info
    fn map(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> TexInfo3D;
}

/// The texture interface
pub trait Texture: Send + Sync {
    type Texel;

    /// Evaluate the texture given interaction info and partial
    /// differential info
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel;

    /// Mean value of the texture
    fn mean(&self) -> Self::Texel;
}

impl<'a, T: 'a> Texture for &'a T
    where T: Texture
{
    type Texel = <T as Texture>::Texel;

    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        (*self).evaluate(si, dxy)
    }

    #[inline]
    fn mean(&self) -> Self::Texel {
        (*self).mean()
    }
}

impl<T: Texture + ?Sized> Texture for Arc<T> {
    type Texel = <T as Texture>::Texel;

    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        (**self).evaluate(si, dxy)
    }

    #[inline]
    fn mean(&self) -> Self::Texel {
        (**self).mean()
    }
}

pub mod mappings;
pub mod textures;
pub mod prelude;
