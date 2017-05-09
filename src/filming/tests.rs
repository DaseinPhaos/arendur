// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// tests
use super::*;

#[cfg(test)]
mod test_projective {
    use super::*;
    use super::projective::*;
    use super::perspective::*;

    // #[test]
    // fn test_perspec_proj() {
    //     let ptrans = PerspecCam::perspective_transform(float::pi()*2. as Float / 3. as Float, 0.01 as Float, 1000. as Float);
    //     let inv_ptrans = ptrans.invert().unwrap();
    //     let point = Point3f::new(0. as Float, 0. as Float, 0. as Float) + Vector3f::new(1. as Float, 1. as Float, 1. as Float);
    //     // let point_t = Point3f::new(1. as Float, 1. as Float, 0.9900099 as Float);
    //     // assert_ulps_eq!(ptrans.transform_point(point), point_t);

    //     // let screen = BBox2f::new(
    //     //     Point2f::new(-1. as Float, -1. as Float),
    //     //     Point2f::new(1. as Float, 1. as Float)
    //     // );
    //     let screen = BBox2f::new(
    //         Point2f::new(-1. as Float, -1. as Float),
    //         Point2f::new(1. as Float, 1. as Float)
    //     );
    //     let resolution = Vector2f::new(800. as Float, 600. as Float);
    //     let projinfo = ProjCameraInfo::new(ptrans, screen, resolution);
    //     // let pixel = Point3f::new(400. as Float, 300. as Float, 1. as Float);
    //     // let pixel_t = projinfo.raster_view.transform_point(pixel);
    //     // assert_ulps_eq!(pixel_t, Point3f::new(0. as Float, 0. as Float, 0.01 as Float));

    //     let pixel = Point3f::new(800. as Float, 600. as Float, 0.0 as Float);
    //     let pixel_v = projinfo.raster_view.transform_point(pixel).to_vec().normalize();
    //     assert_ulps_eq!(pixel_v, Vector3f::new(1. as Float, 1. as Float, 1.0 as Float));
    //     // let screen_view = projinfo.view_screen.invert().unwrap();
    //     // let pixel_s = projinfo.raster_screen.transform_point(pixel);
    //     // assert_ulps_eq!(pixel_s, Point3f::new(1. as Float, -1. as Float, 1.0 as Float));
    //     // let pixel_v = screen_view.transform_point(pixel_s);
    //     // assert_ulps_eq!(pixel_s, Point3f::new(1. as Float, -1. as Float, 1.0 as Float));
    // }
}