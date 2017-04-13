//! Defines projective camera info

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
        let screen_raster = 
            Matrix4f::from_nonuniform_scale(
                resolution.x, resolution.y, 1.0 as Float
            ) * Matrix4f::from_nonuniform_scale(
                1.0 as Float / (screen.pmax.x - screen.pmin.x),
                1.0 as Float / (screen.pmax.y - screen.pmin.y),
                1.0 as Float
            ) * Matrix4f::from_translation(
                Vector3f::new(-screen.pmin.x, -screen.pmax.y, 0.0 as Float)
            );
        let raster_screen = screen_raster.inverse_transform().expect("matrix inversion failure");
        let raster_view = view_screen.inverse_transform().expect("matrix inversion failure") * raster_screen;
        ProjCameraInfo {
            view_screen: view_screen,
            screen_raster: screen_raster,
            raster_screen: raster_screen,
            raster_view: raster_view,
            screen: screen,
        }
    }
}