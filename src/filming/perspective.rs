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
use spectrum::RGBSpectrumf;

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
    area: Float,
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
        let resolution = film.resolutionf();
        let proj_info = ProjCameraInfo::new(
            PerspecCam::perspective_transform(fov, znear, zfar),
            screen, resolution
        );
        
        let mut pview_min = proj_info.raster_view.transform_point(
            Point3f::new(0. as Float, 0. as Float, 0. as Float)
        );
        pview_min /= pview_min.z;
        let mut pview_max = proj_info.raster_view.transform_point(
            Point3f::new(resolution.x, resolution.y, 0. as Float)
        );
        pview_max /= pview_max.z;
        let area = (pview_max.x - pview_min.x)*(pview_max.y - pview_min.y);

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
            area: area,
        }
    }

    /// `fov` in radians
    pub fn perspective_transform(fov: Float, znear: Float, zfar: Float) -> Matrix4f {
        assert!(znear < zfar);
        assert!(fov < float::pi());
        let one = Float::one();
        let zero = Float::zero();
        let persp = Matrix4f::new(
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, zfar/(zfar-znear), one,
            zero, zero, -zfar*znear/(zfar-znear), zero
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
        let mut ray = RawRay::from_od(
            Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float), 
            pview.to_vec().normalize()
        );

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

    fn evaluate_importance(
        &self, pos: Point3f, dir: Vector3f
    ) -> Option<(RGBSpectrumf, Point2f)> {
        let p2v = self.parent_view;
        let dir_view = p2v.transform_vector(dir);
        let costheta = dir_view.z;
        if costheta <= 0. as Float { return None; }
        let focus_t = if let Some(lens) = self.lens {
            lens.1 / costheta
        } else {
            1. as Float / costheta
        };
        let pos_view = p2v.transform_point(pos);
        let focus_view = pos_view + dir_view * focus_t;
        let p_raster = (
            self.proj_info.screen_raster*self.proj_info.view_screen
        ).transform_point(focus_view);
        let p_raster = Point2::new(p_raster.x, p_raster.y);
        
        let bound: BBox2<isize> = BBox2::new(Point2::new(0, 0), self.film.resolution().cast());
        if !bound.contain_lb(p_raster.cast()) { return None; }

        let costheta2 = costheta * costheta;
        let lens_area = if let Some(lens) = self.lens {
            float::pi() * lens.0 * lens.0
        } else {
            1. as Float
        };
        let importance = 1. as Float / (self.area * lens_area * costheta2 * costheta2);
        Some((
            RGBSpectrumf::new(importance, importance, importance),
            p_raster
        ))
    }

    fn pdf(&self, pos: Point3f, dir: Vector3f) -> (Float, Float) {
        let ret = (0. as Float, 0. as Float);
        let p2v = self.parent_view;
        let dir_view = p2v.transform_vector(dir);
        let costheta = dir_view.z;
        if costheta <= 0. as Float { return ret; }
        let focus_t = if let Some(lens) = self.lens {
            lens.1 / costheta
        } else {
            1. as Float / costheta
        };
        let pos_view = p2v.transform_point(pos);
        let focus_view = pos_view + dir_view * focus_t;
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
            1. as Float/lens_area, // pdfpos
            1. as Float/(self.area * costheta * costheta * costheta) // pdfdir
        )
    }
}
