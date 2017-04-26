// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines an orthographic camera

use geometry::prelude::*;
use super::{Camera, SampleInfo, ImportanceSample};
use super::projective::ProjCameraInfo;
use super::film::Film;
use spectrum::{RGBSpectrumf, Spectrum};
use sample;

/// An orthographic camera
pub struct OrthoCam {
    view_parent: Matrix4f,
    parent_view: Matrix4f,
    proj_info: ProjCameraInfo,
    dx: Vector3f,
    dy: Vector3f,
    /// lens_radius, focal_distance; if presented
    lens: Option<(Float, Float)>,
    film: Film,
}

impl OrthoCam {
    /// Construction
    pub fn new(
        view_parent: Matrix4f,
        screen: BBox2f,
        znear: Float,
        zfar: Float,
        lens: Option<(Float, Float)>,
        film: Film,
    ) -> OrthoCam {
        let parent_view = view_parent.inverse_transform().expect("matrix inversion failure");
        let proj_info = ProjCameraInfo::new(
            OrthoCam::ortho_transform(znear, zfar),
            screen,
            film.resolutionf(),
        );
        let dx = proj_info.raster_view.transform_vector(Vector3f::new(1.0 as Float, 0.0 as Float, 0.0 as Float));
        let dy = proj_info.raster_view.transform_vector(Vector3f::new(0.0 as Float, 1.0 as Float, 0.0 as Float));
        OrthoCam{
            view_parent: view_parent,
            parent_view: parent_view,
            proj_info: proj_info,
            dx: dx,
            dy: dy,
            lens: lens,
            film: film,
        }
    }

    pub fn ortho_transform(znear: Float, zfar: Float) -> Matrix4f {
        Matrix4f::from_nonuniform_scale(
            1.0 as Float, 
            1.0 as Float, 
            1.0 as Float / (zfar - znear)
        ) * Matrix4f::from_translation(
            Vector3f::new(0.0 as Float, 0.0 as Float, -znear)
        )
    }
}

impl Camera for OrthoCam {
    fn parent_to_view(&self) -> Matrix4f {
        self.parent_view
    }

    fn view_to_parent(&self) -> Matrix4f {
        self.view_parent
    }

    fn evaluate_importance(
        &self, pos: Point3f, dir: Vector3f
    ) -> Option<(RGBSpectrumf, Point2f)> {
        let p2v = self.parent_view;
        let dir_view = p2v.transform_vector(dir);
        let costheta = dir_view.z;
        if !relative_eq!(costheta, 1. as Float) { return None; }

        let focus_t = if let Some(lens) = self.lens {
            lens.1 / costheta
        } else {
            1. as Float/costheta
        };
        let pos_view = p2v.transform_point(pos);
        let focus_view = pos_view + focus_t * dir_view;
        let p_raster = (
            self.proj_info.screen_raster*self.proj_info.view_screen
        ).transform_point(focus_view);
        let p_raster = Point2::new(p_raster.x, p_raster.y);
        
        let bound: BBox2<isize> = BBox2::new(Point2::new(0, 0), self.film.resolution().cast());
        if !bound.contain_lb(p_raster.cast()) { return None; }

        let lens_area = if let Some(lens) = self.lens {
            float::pi() * lens.0 * lens.0
        } else {
            1. as Float
        };
        let importance = 1. as Float / lens_area;
        Some((
            RGBSpectrumf::new(importance, importance, importance),
            Point2f::new(p_raster.x, p_raster.y)
        ))

    }

        fn evaluate_importance_sampled(
        &self, posw: Point3f, _sample: Point2f
    ) -> (ImportanceSample, Point2f) {
        // FIXME: account for lens distortion
        let norm = self.view_parent.transform_vector(
            Vector3f::new(0. as Float, 0. as Float, 1. as Float)
        );
        let pfrom = self.parent_view.transform_point(posw);
        let pfrom = Point3f::new(pfrom.x, pfrom.y, 0. as Float);
        let pfrom = self.view_parent.transform_point(pfrom);

        let pto = posw;

        let dist2 = norm.magnitude2();
        let dir = norm/dist2.sqrt();
        let (importance, praster) = if let Some((i, pr)) = self.evaluate_importance(pto, -dir) {
            (i, pr)
        } else {
            (RGBSpectrumf::black(), Point2f::new(0. as Float, 0. as Float))
        };
        let pdf = if let Some((r, _)) = self.lens {
            dist2 / (r*r*float::pi())
        } else {
            1. as Float
        };
        (ImportanceSample{
            radiance: importance,
            pdf: pdf,
            pfrom: pfrom,
            pto: posw,
        }, praster)
    }

    fn pdf(&self, pos: Point3f, dir: Vector3f) -> (Float, Float) {
        let ret = (0. as Float, 0. as Float);
        let p2v = self.parent_view;
        let dir_view = p2v.transform_vector(dir);
        let costheta = dir_view.z;
        if !relative_eq!(costheta, 1. as Float) { return ret; }

        let focus_t = if let Some(lens) = self.lens {
            lens.1 / costheta
        } else {
            1. as Float/costheta
        };
        
        let pos_view = p2v.transform_point(pos);
        let focus_view = pos_view + focus_t * dir_view;
        let p_raster = (
            self.proj_info.screen_raster*self.proj_info.view_screen
        ).transform_point(focus_view);
        let p_raster = Point2::new(p_raster.x, p_raster.y);
        
        let bound: BBox2<isize> = BBox2::new(Point2::new(0, 0), self.film.resolution().cast());
        if !bound.contain_lb(p_raster.cast()) { return ret; }

        let lens_area = if let Some(lens) = self.lens {
            float::pi() * lens.0 * lens.0
        } else {
            1. as Float
        };
        (
            1. as Float / lens_area, // pdfpos
            1. as Float, // pdfdir
        )
    }

    fn generate_path(&self, sample_info: SampleInfo) -> RawRay {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(pview, Vector3f::new(0.0 as Float, 0.0 as Float, 1.0 as Float));
        if let Some((r, d)) = self.lens {
            debug_assert!(r>0.0 as Float);
            debug_assert!(d>0.0 as Float);
            let plens = r * sample::sample_concentric_disk(sample_info.plens);
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

    fn generate_path_differential(&self, sample_info: SampleInfo) -> RayDifferential {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(pview, Vector3f::new(0.0 as Float, 0.0 as Float, 1.0 as Float));

        if let Some((r, d)) = self.lens {
            debug_assert!(r>0.0 as Float);
            debug_assert!(d>0.0 as Float);
            let plens = r * sample::sample_concentric_disk(sample_info.plens);
            let ft = d/ray.direction().z;
            let pfocus = ray.evaluate(ft);
            let new_origin = Point3f::new(plens.x, plens.y, 0.0 as Float);
            ray = RawRay::from_od(
                new_origin,
                (pfocus - new_origin).normalize()
            );
        }
        // TODO: account for lens
        let rx = RawRay::from_od(ray.origin() + self.dx, ray.direction());
        let ry = RawRay::from_od(ray.origin() + self.dy, ray.direction());
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
