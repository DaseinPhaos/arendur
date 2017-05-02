// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines general information about a projective camera

use geometry::prelude::*;

#[derive(Clone)]
pub struct ProjCameraInfo {
    pub view_screen: Matrix4f,
    pub screen_raster: Matrix4f,
    pub raster_screen: Matrix4f,
    pub raster_view: Matrix4f,
    pub screen: BBox2f,
}

impl ProjCameraInfo {
    /// construction
    pub fn new(
        view_screen: Matrix4f,
        screen: BBox2f,
        resolution: Vector2f
    ) -> ProjCameraInfo {
        let raster_screen = Matrix4f::from_translation(
            Vector3f::new(screen.pmin.x, screen.pmax.y, 0. as Float)
        ) * Matrix4f::from_nonuniform_scale(
            (screen.pmax.x-screen.pmin.x)/resolution.x,
            (screen.pmin.y-screen.pmax.y)/resolution.y, 
             1. as Float
        );
        let screen_raster = raster_screen.invert().unwrap();
        let raster_view = view_screen.invert().expect("matrix inversion failure") * raster_screen;
        ProjCameraInfo {
            view_screen: view_screen,
            screen_raster: screen_raster,
            raster_screen: raster_screen,
            raster_view: raster_view,
            screen: screen,
        }
    }
}
