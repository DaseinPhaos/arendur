//! defines orthographic camera

use geometry::prelude::*;
use super::{Camera, SampleInfo};
use super::projective::ProjCameraInfo;

/// An orthographic camera
pub struct OrthoCam {
    view_parent: Matrix4f,
    parent_view: Matrix4f,
    proj_info: ProjCameraInfo,
    dx: Vector3f,
    dy: Vector3f,
    /// lens_radius, focal_distance; if presented
    lens: Option<(Float, Float)>,
    // TODO: film?
}

impl OrthoCam {
    /// Construction
    pub fn new(
        view_parent: Matrix4f,
        screen: BBox2f,
        znear: Float,
        zfar: Float,
        lens: Option<(Float, Float)>
    ) -> OrthoCam {
        let parent_view = view_parent.inverse_transform().expect("matrix inversion failure");
        let proj_info = ProjCameraInfo::new(
            OrthoCam::ortho_transform(znear, zfar),
            screen,
            // FIXME: should depdends on the film
            Vector2f::new(800.0 as Float, 600.0 as Float)
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

    fn generate_ray(&self, sample_info: SampleInfo) -> RawRay {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(pview, Vector3f::new(0.0 as Float, 0.0 as Float, 1.0 as Float));
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
        let mut ray = RawRay::from_od(pview, Vector3f::new(0.0 as Float, 0.0 as Float, 1.0 as Float));

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
        let rx = RawRay::from_od(ray.origin() + self.dx, ray.direction());
        let ry = RawRay::from_od(ray.origin() + self.dy, ray.direction());
        let ret = RayDifferential{
            ray: ray, raydx: rx, raydy: ry
        };
        self.view_parent.transform_ray_differential(&ret)
    }
}