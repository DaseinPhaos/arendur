// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// tests
use super::*;
extern crate rand;
use self::rand::*;
use sample;
use cgmath::Quaternion;

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
        let (t, si) = sphere.intersect_ray(&ray).unwrap();
        assert_relative_eq!(si.basic.norm, Vector3f::new(0. as Float, 0. as Float, -1. as Float));
    }

    #[test]
    fn test_random_intersect() {
        let mut rng = thread_rng();
        const ROUNDS: usize = 256;
        let spheref = Sphere::full(
            1.0 as Float
        );
        for i in 0..ROUNDS {
            let s = Point2f::new(rng.gen_range(0.0 as Float, 1.0 as Float), rng.gen_range(0.0 as Float, 1.0 as Float));
            let s = sample::sample_uniform_sphere(s);
            let p = Point3f::from_vec(s * 2.0 as Float);
            let trans: Matrix4f = Quaternion::from_arc(Vector3f::new(0. as Float, 0. as Float, 1. as Float), -s, None).into();
            let mut unhit = 0;
            for j in 0..ROUNDS {
                let s = Point2f::new(rng.gen_range(0.0 as Float, 1.0 as Float), rng.gen_range(0.0 as Float, 1.0 as Float));
                let v = sample::sample_cosw_hemisphere(s);
                let v = trans.transform_vector(v);
                let ray = RawRay::from_od(p, v);
                if let Some((t, si)) = spheref.intersect_ray(&ray) {
                    assert!(si.basic.wo.dot(si.basic.norm) > 0.0 as Float);
                    assert_relative_eq!(-v, si.basic.wo);
                } else {
                    unhit += 1;
                }
            }
            assert!(unhit != ROUNDS);
        }
    }
}
