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
mod test_sphere {
    use super::*;
    use super::sphere::*;
    
    #[test]
    fn test_sy_intersect() {
        let spheref = Sphere::full(
            1.0 as Float
        );
        let ray = RawRay::from_od(
            Point3f::new(0. as Float, 0. as Float, -10. as Float),
            Vector3f::new(0. as Float, 0. as Float, 1. as Float),
        );

        assert!(spheref.can_intersect(&ray));

        let sphere = Sphere::new(
            1.0 as Float, -1.0 as Float, 0.5 as Float, float::pi()*2. as Float
        );
        let ray = RawRay::from_od(
            Point3f::new(0. as Float, 0. as Float, -10. as Float),
            Vector3f::new(0. as Float, 0. as Float, 1. as Float),
        );

        assert!(sphere.can_intersect(&ray));

        let sphere = Sphere::full(20. as Float);
        let ray = RawRay::from_od(
            Point3f::new(0. as Float, 0. as Float, -30. as Float),
            Vector3f::new(0. as Float, 0. as Float, 1. as Float),
        );
        assert!(sphere.intersect_ray(&ray)!=None);
    }
}
