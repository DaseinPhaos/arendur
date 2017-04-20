// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! whitted renderer usage example

extern crate arendur;
extern crate cgmath;
extern crate rand;

use arendur::prelude::*;
type NaiveAggregate = arendur::component::naive::Naive;
use std::sync::Arc;

fn main() {
    println!("Whitted example");
    use std::io;
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    let transform0 = Arc::new(Matrix4f::from_translation(Vector3f::new(12.0 as Float, 12.0 as Float, 30.0 as Float)));
    let transform1 = Arc::new(Matrix4f::from_translation(Vector3f::new(-12.0 as Float, -12.0 as Float, 30.0 as Float)));
    let inv_transform0 = Arc::new(transform0.invert().unwrap());
    let inv_transform1 = Arc::new(transform1.invert().unwrap());
    
    // let sphere0 = Sphere::new(8. as Float, -7. as Float, 7. as Float, 6.28 as Float);
    let sphere0 = Sphere::full(8. as Float);
    // let sphere0 = Sphere::new(SphereInfo::new(20. as Float, -28. as Float, 58. as Float, 6.49 as Float), ShapeInfo::new(transform0, inv_transform0, false));
    let sphere1 = Sphere::full(8. as Float);

    let kd = ConstantTexture{value: RGBSpectrumf::new(0.5 as Float, 0.5 as Float, 1.0 as Float)};
    let sigma = ConstantTexture{value: 30. as Float};


    let material0 = MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);

    let sphere0 = ShapedPrimitive::new(sphere0, material0, None);
    let sphere0 = TransformedComposable::new(sphere0, transform0, inv_transform0);

    let kd = ConstantTexture{value: RGBSpectrumf::new(0.5 as Float, 1.0 as Float, 0.5 as Float)};
    let sigma = ConstantTexture{value: 1.0 as Float};


    let material1 = MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);

    let sphere1 = ShapedPrimitive::new(sphere1, material1, None);
    let sphere1 = TransformedComposable::new(sphere1, transform1, inv_transform1);

    let mut naive = NaiveAggregate::from_one(Arc::new(sphere0));
    naive.append(Arc::new(sphere1));

    let lights: Vec<Arc<Light>> = vec![
        // Arc::new(PointLight::new(
        //     Point3f::new(-10.0 as Float, 0.0 as Float, 0.0 as Float),
        //     RGBSpectrumf::new(300.7 as Float, 0.0 as Float, 0.0 as Float))
        // ),
        // Arc::new(PointLight::new(
        //     Point3f::new(0.0 as Float, -10.0 as Float, 0.0 as Float),
        //     RGBSpectrumf::new(0.0 as Float, 300.0 as Float, 0.0 as Float))
        // ), 
        // Arc::new(PointLight::new(
        //     Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float),
        //     RGBSpectrumf::new(0.0 as Float, 0.0 as Float, 300.0 as Float))
        // ), 
        // Arc::new(PointLight::new(
        //     Point3f::new(0.0 as Float, 100.0 as Float, 100.0 as Float),
        //     RGBSpectrumf::new(100000.7 as Float, 0.0 as Float, 0.0 as Float))
        // ),
        Arc::new(PointLight::new(
            Point3f::new(0.0 as Float, -40.0 as Float, 30.0 as Float),
            RGBSpectrumf::new(0.0 as Float, 300.0 as Float, 0.0 as Float))
        ), 
        Arc::new(PointLight::new(
            Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float),
            RGBSpectrumf::new(300.0 as Float, 300.0 as Float, 300.0 as Float))
        ), 
    ];
    
    let scene = Scene{lights: lights, aggregate: Arc::new(naive)};

    let camera = PerspecCam::new(
        Matrix4f::identity(), 
        BBox2f::new(
            Point2f::new(-1.0 as Float, -1.0 as Float), 
            Point2f::new(1.0 as Float, 1.0 as Float)
        ),
        0.1 as Float, 
        1000.0 as Float, 
        // float::pi()*2.0 as Float / 3.0 as Float, 
        float::frac_pi_2(),
        None, 
        Film::new(
            Point2::new(600, 600), 
            BBox2f::new(
                Point2f::new(0.0 as Float, 0.0 as Float), 
                Point2f::new(1.0 as Float, 1.0 as Float)
            ),
            Arc::new(
                MitchellFilter::new(
                    Vector2f::new(2.0 as Float, 2.0 as Float), 
                    0.5 as Float, 0.25 as Float,
                )
            )
        )
    );
    let mut renderer = WhittedRenderer::new(StrataSampler::new(9, 9, 10, rand::StdRng::new().unwrap()), Arc::new(camera), "target/test3.png");

    renderer.render(&scene);

    
}
