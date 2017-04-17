// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The Lighting Interface.

use geometry::prelude::*;
use spectrum::*;
use component::Composable;

/// A Light
pub trait Light: Sync+ Send {
    /// return the flags of the light
    fn flags(&self) -> LightFlag;

    /// test if the light is delta
    fn is_delta(&self) -> bool {
        self.flags().is_delta()
    }

    // /// get transforms
    // fn local_parent(&self) -> Matrix4f;

    // /// get transforms
    // fn parent_local(&self) -> Matrix4f;

    /// sample the light at a location in parent frame `posw`, given `sample`
    fn evalute_sampled(&self, posw: Point3f, sample: Point2f) -> LightSample;

    /// returns an estimation of total power of this light
    fn power(&self) -> RGBSpectrumf;

    // /// preporcess with scene components if necessary
    // pub fn preprocess(&mut self, )
}

// /// An area light
// pub trait AreaLight: Light {
//     /// evaluate 
//     fn evalute()
// }

bitflags! {
    pub flags LightFlag: u32 {
        const LIGHT_DPOS = 0x1,
        const LIGHT_DDIR = 0x2,
        const LIGHT_AREA = 0x4,
        const LIGHIT_INFINITE = 0x8,
    }
}

impl LightFlag {
    /// test if the light is delta light
    pub fn is_delta(self) -> bool {
        (self & LIGHT_DPOS == LIGHT_DPOS) ||
        (self & LIGHT_DDIR == LIGHT_DDIR)
    }
}

/// Results of a light sample
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct LightSample {
    /// outgoing radiance
    pub radiance: RGBSpectrumf,
    /// pdf for this sample
    pub pdf: Float,
    /// outgoing point, parent frame
    pub pfrom: Point3f,
    /// receiving point, parent frame
    pub pto: Point3f,
}

impl LightSample {
    /// get light direction vector `wi`
    #[inline]
    pub fn wi(&self) -> Vector3f {
        (self.pto - self.pfrom).normalize()
    }

    /// test if this light would be occulued by any components
    /// in `Composable`, assuming they are in the same world frame
    #[inline]
    pub fn occluded<C: Composable>(&self, components: &C) -> bool {
        let ray = RawRay::spawn(self.pfrom, self.pto);
        components.can_intersect(&ray)
    }

    #[inline]
    pub fn apply_transform<T>(&self, t: &T) -> LightSample
        where T: TransformExt
    {
        LightSample {
            radiance: self.radiance,
            pdf: self.pdf,
            pfrom: t.transform_point(self.pfrom),
            pto: t.transform_point(self.pto),
        }
    }
}

pub mod pointlights;
pub mod distantlight;
pub mod arealights;