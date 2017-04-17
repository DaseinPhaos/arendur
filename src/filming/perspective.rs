// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines a perspective camera

use geometry::prelude::*;
use super::{Camera, SampleInfo};
use super::projective::ProjCameraInfo;
use super::film::Film;

/// A perspective camera
pub struct PerspecCam {
    view_parent: Matrix4f,
    parent_view: Matrix4f,
    proj_info: ProjCameraInfo,
    dx: Vector3f,
    dy: Vector3f,
    /// lens_radius, focal_distance; if presented
    lens: Option<(Float, Float)>,
    film: Film,
}

impl PerspecCam {
    /// Construction
    pub fn new(
        view_parent: Matrix4f,
        screen: BBox2f,
        znear: Float,
        zfar: Float,
        fov: Float,
        lens: Option<(Float, Float)>,
        film: Film
    ) -> PerspecCam {
        let parent_view = view_parent.inverse_transform().expect("matrix inversion failure");
        let proj_info = ProjCameraInfo::new(
            PerspecCam::perspective_transform(fov, znear, zfar),
            screen,
            film.resolutionf(),
        );
        let or2v = proj_info.raster_view.transform_point(
            Point3f::new(1.0 as Float, 0.0 as Float, 0.0 as Float)
        );
        let dx = proj_info.raster_view.transform_point(
            Point3f::new(1.0 as Float, 0.0 as Float, 0.0 as Float)
        ) - or2v;
        let dy = proj_info.raster_view.transform_point(
            Point3f::new(0.0 as Float, 1.0 as Float, 0.0 as Float)
        ) - or2v;
        PerspecCam{
            view_parent: view_parent,
            parent_view: parent_view,
            proj_info: proj_info,
            dx: dx,
            dy: dy,
            lens: lens,
            film: film,
        }
    }

    /// `fov` in radians
    pub fn perspective_transform(fov: Float, znear: Float, zfar: Float) -> Matrix4f {
        let one = Float::one();
        let zero = Float::zero();
        let persp = Matrix4f::new(
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, zfar/(zfar-znear), -zfar*znear/(zfar-znear),
            zero, zero, one, zero
        );

        let inv_tan = one/ (fov * 0.5 as Float).tan();
        Matrix4f::from_nonuniform_scale(inv_tan, inv_tan, one) * persp
    }
}

impl Camera for PerspecCam {
    fn parent_to_view(&self) -> Matrix4f {
        self.parent_view
    }

    fn view_to_parent(&self) -> Matrix4f {
        self.view_parent
    }

    fn generate_ray(&self, sample_info: SampleInfo) -> RawRay {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float), pview.to_vec().normalize());

        if let Some((r, d)) = self.lens {
            debug_assert!(r>0.0 as Float);
            debug_assert!(d>0.0 as Float);
            // FIXME: should be disk samples
            let plens = sample_info.plens;
            let ft = d/ray.direction().z;
            let pfocus = ray.evaluate(ft);
            let new_origin = Point3f::new(plens.x, plens.y, 0.0 as Float);
            ray = RawRay::from_od(
                new_origin,
                (pfocus - new_origin).normalize()
            );
        }
        // TODO: update ray medium
        self.view_parent.transform_ray(&ray)
    }

    fn generate_ray_differential(&self, sample_info: SampleInfo) -> RayDifferential {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float), pview.to_vec().normalize());

        if let Some((r, d)) = self.lens {
            debug_assert!(r>0.0 as Float);
            debug_assert!(d>0.0 as Float);
            // FIXME: should be disk samples
            let plens = sample_info.plens;
            let ft = d/ray.direction().z;
            let pfocus = ray.evaluate(ft);
            let new_origin = Point3f::new(plens.x, plens.y, 0.0 as Float);
            ray = RawRay::from_od(
                new_origin,
                (pfocus - new_origin).normalize()
            );
        }
        // TODO: account for lens
        let rx = RawRay::from_od(ray.origin(), (pview.to_vec()+self.dx).normalize());
        let ry = RawRay::from_od(ray.origin(), (pview.to_vec()+self.dy).normalize());
        let ret = RayDifferential{
            ray: ray,
            diffs: Some((rx, ry)),
        };
        self.view_parent.transform_ray_differential(&ret)
    }

    #[inline]
    fn get_film(&self) -> &Film {
        &self.film
    }

    #[inline]
    fn get_film_mut(&mut self) -> &mut Film {
        &mut self.film
    }
}
