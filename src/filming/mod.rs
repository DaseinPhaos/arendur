// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines cameras and films.

use geometry::prelude::*;
use self::film::Film;
use spectrum::RGBSpectrumf;

/// Samples for camera to generate rays.
#[derive(Copy, Clone, PartialEq)]
pub struct SampleInfo {
    pub pfilm: Point2f,
    pub plens: Point2f,
}

/// A camera!
pub trait Camera: Send + Sync {
    /// parent to view-space transform
    fn parent_to_view(&self) -> Matrix4f;

    /// view to parent
    fn view_to_parent(&self) -> Matrix4f {
        self.parent_to_view().inverse_transform().expect("matrix inversion failure")
    }

    /// evaluate importance, given `posw` and `dirw` of a camera ray
    /// returns the importance and the raster position of the ray
    fn evaluate_importance(
        &self, posw: Point3f, dirw: Vector3f
    ) -> Option<(RGBSpectrumf, Point2f)>;

    /// evaludate pdf from the given `posw` and `dirw`, returned as `(pdfpos, pdfdir)`
    fn pdf(&self, posw: Point3f, dirw: Vector3f) -> (Float, Float);

    /// generate a ray based on sample info
    fn generate_ray(&self, sample_info: SampleInfo) -> RawRay;

    /// generate a differential ray based on sample info
    fn generate_ray_differential(&self, sample_info: SampleInfo) -> RayDifferential {
        let ray = self.generate_ray(sample_info);
        let ray_dx = {
            let mut sample_info = sample_info;
            sample_info.pfilm.x += 1.0 as Float;
            self.generate_ray(sample_info)
        };
        let ray_dy = {
            let mut sample_info = sample_info;
            sample_info.pfilm.y += 1.0 as Float;
            self.generate_ray(sample_info)
        };

        RayDifferential{
            ray: ray,
            diffs: Some((ray_dx, ray_dy)),
        }
    }

    // TODO: add medium

    // TODO: add film
    fn get_film(&self) -> &Film;

    fn get_film_mut(&mut self) -> &mut Film;
}

mod projective;
pub mod ortho;
pub mod perspective;
pub mod film;
pub mod prelude;
#[cfg(test)]
mod tests;
