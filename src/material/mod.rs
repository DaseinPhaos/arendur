// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines the material interface

use geometry::prelude::*;
use texturing::*;
use copy_arena::Allocator;
use std::sync::Arc;

/// The material interface
pub trait Material: Sync + Send {
    /// 
    fn compute_scattering<'a>(
        &self,
        si: &mut SurfaceInteraction,
        dxy: &DxyInfo,
        alloc: &mut Allocator<'a>
    ) -> bsdf::Bsdf<'a>;
}

impl<T: Material + ?Sized> Material for Arc<T> {
    #[inline]
    fn compute_scattering<'a>(
        &self,
        si: &mut SurfaceInteraction,
        dxy: &DxyInfo,
        alloc: &mut Allocator<'a>
    ) -> bsdf::Bsdf<'a> {
        <T as Material>::compute_scattering(
            &*self, si, dxy, alloc
        )
    }
}

// utility to bump a map
fn add_bumping<T: Texture<Texel=Float> + ?Sized>(si: &mut SurfaceInteraction, dxy: &DxyInfo, bump: &T) {
    let mut sie = si.clone();
    let du = {
        // shifting in u
        let mut du = 0.5 as Float * (dxy.dudx.abs() + dxy.dudy.abs());
        if du == 0.0 as Float { du = 0.0005 as Float; }
        sie.basic.pos = si.basic.pos + du * si.shading_duv.dpdu;
        sie.uv.x = si.uv.x + du;
        sie.basic.norm = (si.shading_duv.dpdu.cross(si.shading_duv.dpdv) + du * si.duv.dndu).normalize();
        du
    };

    let displacement_u = bump.evaluate(&sie, dxy);

    let dv = {
        // shifting in v
        let mut dv = 0.5 as Float * (dxy.dvdx.abs() + dxy.dvdy.abs());
        if dv == 0.0 as Float { dv = 0.0005 as Float; }
        sie.basic.pos = si.basic.pos + dv * si.shading_duv.dpdv;
        sie.uv.y = si.uv.y + dv;
        sie.basic.norm = (si.shading_duv.dpdu.cross(si.shading_duv.dpdv) + dv * si.duv.dndv).normalize();
        dv
    };

    let displacement_v = bump.evaluate(&sie, dxy);

    let displacement = bump.evaluate(si, dxy);

    let dpdu = si.shading_duv.dpdu + // original
        (displacement_u - displacement) / du * si.shading_norm + // ddu
        displacement * si.shading_duv.dndu; // d
    let dpdv = si.shading_duv.dpdv + // original
        (displacement_v - displacement) / dv * si.shading_norm + // ddu
        displacement * si.shading_duv.dndv; // d

    let duvinfo = DuvInfo{
        dpdu: dpdu,
        dpdv: dpdv,
        dndu: si.shading_duv.dndu,
        dndv: si.shading_duv.dndv,
    };
    si.set_shading(duvinfo, false);
}

pub mod bsdf;
pub mod matte;
pub mod plastic;
pub mod prelude;
