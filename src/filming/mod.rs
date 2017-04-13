//! Defines cameras and films.

use geometry::prelude::*;

/// Samples for camera to generate rays.
#[derive(Copy, Clone, PartialEq)]
pub struct SampleInfo {
    pub pfilm: Point2f,
    pub plens: Point2f,
}

/// A camera!
pub trait Camera {
    /// parent to view-space transform
    fn parent_to_view(&self) -> Matrix4f;

    /// view to parent
    fn view_to_parent(&self) -> Matrix4f {
        self.parent_to_view().inverse_transform().expect("matrix inversion failure")
    }

    // /// view to NDC
    // /// NDC is defined as $[0, 1]\times [0, 1]\times [0, 1]$
    // fn view_to_ndc(&self) -> Matrix4f;

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
            ray: ray, raydx: ray_dx, raydy: ray_dy
        }
    }

    // TODO: add medium

    // TODO: add film
}

/// A film!
pub trait Film {
    /// get resolution
    fn resolution(&self) -> Point2<u32>;

    // TODO: filter
}

pub mod projective;
pub mod ortho;
pub mod perspective;