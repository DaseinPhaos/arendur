// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Commonly used implementation of `Mapping2D` and `Mapping3D`.

use super::*;

/// A uv mapping using surface interaction's duv info,
/// with scaling and shifting
#[derive(Copy, Clone, PartialEq)]
pub struct UVMapping {
    pub scaling: Vector2f,
    pub shifting: Vector2f,
}

impl Mapping2D for UVMapping {
    #[inline]
    fn map(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> TexInfo2D {
        TexInfo2D{
            p: Point2f::from_vec(si.uv.to_vec().mul_element_wise(self.scaling) + self.shifting),
            dpdx: Vector2f::new(self.scaling.x * dxy.dudx, self.scaling.y * dxy.dvdx),
            dpdy: Vector2f::new(self.scaling.x * dxy.dudy, self.scaling.y * dxy.dvdy),
        }
    }
}

/// 3D mapping through transform
#[derive(Copy, Clone, PartialEq)]
pub struct TransformedMapping {
    pub transform: Matrix4f,
}

impl Mapping3D for TransformedMapping {
    #[inline]
    fn map(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> TexInfo3D {
        TexInfo3D{
            p: self.transform.transform_point(si.basic.pos),
            dpdx: self.transform.transform_vector(dxy.dpdx),
            dpdy: self.transform.transform_vector(dxy.dpdy),
        }
    }
}
