// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! path tracing renderer usage example

extern crate arendur;
extern crate cgmath;
extern crate rand;
extern crate rayon;
#[cfg(feature = "flame")]
extern crate flame;

use arendur::prelude::*;
type NaiveAggregate = arendur::component::naive::Naive;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::*;

fn main() {
    rayon::initialize(rayon::Configuration::new().num_threads(4)).unwrap();
    println!("Path tracing example");
    println!("component ptr size : {}", std::mem::size_of::<arendur::component::ComponentPointer>());
    use std::io;
    let mut s = String::new();

    let _ = io::stdin().read_line(&mut s);
    println!("Rendering...");
    let sudato = Instant::now();
    let transform0 = Arc::new(Matrix4f::from_translation(Vector3f::new(12.0 as Float, 12.0 as Float, 30.0 as Float)));
    let transform1 = Arc::new(Matrix4f::from_translation(Vector3f::new(0.0 as Float, 12.0 as Float, 0.0 as Float)));
    let inv_transform0 = Arc::new(transform0.invert().unwrap());
    let inv_transform1 = Arc::new(transform1.invert().unwrap());
    // let transform2 = Arc::new(Matrix4f::from_translation(Vector3f::new(-5.0 as Float, 20.0 as Float, 10.0 as Float)));
    let transform2 = Arc::new(Matrix4f::from_translation(Vector3f::new(-5.0 as Float, 8.0 as Float, 3.50 as Float)));
    // let transform2 = Arc::new(Matrix4f::from_translation(Vector3f::new(0.0 as Float, 6.0 as Float, 3.50 as Float)));
    let inv_transform2 = Arc::new(transform2.invert().unwrap());

    // let sphere0 = Sphere::new(8. as Float, -7. as Float, 7. as Float, 6.28 as Float);
    let sphere0 = Sphere::full(8. as Float);
    // let sphere0 = Sphere::new(SphereInfo::new(20. as Float, -28. as Float, 58. as Float, 6.49 as Float), ShapeInfo::new(transform0, inv_transform0, false));
    let sphere1 = Sphere::full(8. as Float);
    // let sphere2 = Sphere::full(3. as Float);
    let sphere2 = Sphere::full(2. as Float);

    let mut ref_table = HashMap::new();
    let info = ImageInfo{
        name: String::from("target/540.jpg"),
        trilinear: false,
        max_aniso: 16. as Float,
        wrapping: ImageWrapMode::Repeat,
        gamma: false,
        scale: 1. as Float,
    };
    let kd = RGBImageTexture::new(
        info.clone(),
        UVMapping{
            scaling: Vector2f::new(16. as Float, 16. as Float),
            shifting: Vector2f::new(0. as Float, 0. as Float),
        },
        &mut ref_table,
    ).unwrap();
    let sigma = ConstantTexture{value: 0. as Float};


    let material0 = MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);

    let sphere0 = ShapedPrimitive::new(sphere0, material0.clone(), None);
    let sphere0 = TransformedComposable::new(sphere0, transform0, inv_transform0);

    let kd = ConstantTexture{value: RGBSpectrumf::new(0.5 as Float, 0.6 as Float, 0.7 as Float)};
    let sigma = ConstantTexture{value: 10. as Float};
    let material1 = MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);
    
    let texture = ConstantTexture{value: RGBSpectrumf::new(5.5 as Float, 5.5 as Float, 5.0 as Float)};

    let sphere1 = ShapedPrimitive::new(sphere1, material1.clone(), Some(Arc::new(texture)));
    let sphere1 = TransformedComposable::new(sphere1, transform1, inv_transform1);

    let kd = ConstantTexture{value: RGBSpectrumf::new(0.6 as Float, 0.6 as Float, 0.6 as Float)};
    let sigma = ConstantTexture{value: 1.0 as Float};


    let material2 = MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);

    let texture = ConstantTexture{value: RGBSpectrumf::new(35.5 as Float, 34.2 as Float, 27.3 as Float)};

    // let texture = ConstantTexture{value: RGBSpectrumf::new(0.5 as Float, 0.5 as Float, 0.5 as Float)};


    let sphere2 = ShapedPrimitive::new(sphere2, material2.clone(), Some(Arc::new(texture)));
    let sphere2 = Arc::new(TransformedComposable::new(sphere2, transform2, inv_transform2));

    let mut naive = NaiveAggregate::from_one(sphere2.clone());
    
    // std::env::set_current_dir("./target/sponza/").unwrap();
    // let bvh = BVH::load_obj(
    //     "sponza.obj", Matrix4f::from_translation(
    //         Vector3f::new(0.0 as Float, -5.0 as Float, 15.0 as Float)
    //     ) * Matrix4f::from_angle_y(Rad(float::frac_pi_2()))
    //       * Matrix4f::from_scale(1.0 as Float)
    // ).unwrap();
    // println!("bbox:{:?}", bvh.bbox_parent());
    // // naive.append(Arc::new(bvh));
    // let naive = bvh;

    // std::env::set_current_dir("./target/sibenik/").unwrap();
    // #[cfg(feature = "flame")]
    // flame::start("BVH Loading");
    // let bvh = BVH::load_obj(
    //     "sibenik.obj", Matrix4f::from_translation(
    //         Vector3f::new(0.0 as Float, -0.0 as Float, 15.0 as Float)
    //     ) * Matrix4f::from_angle_y(Rad(float::frac_pi_2()))
    //       * Matrix4f::from_scale(1.0 as Float)
    // ).unwrap();
    // #[cfg(feature = "flame")]
    // flame::end("BVH Loading");
    // println!("bbox:{:?}", bvh.bbox_parent());
    // // naive.append(Arc::new(bvh));
    // let naive = bvh;

    std::env::set_current_dir("./target/mitsuba/").unwrap();
    let bvh = BVH::load_obj(
        "mitsuba.obj", Matrix4f::from_translation(
            Vector3f::new(0.0 as Float, -2.50 as Float, 7.0 as Float)
        ) * Matrix4f::from_angle_y(Rad(float::pi()))
          * Matrix4f::from_scale(3.0 as Float)
    ).unwrap();
    println!("bbox:{:?}", bvh.bbox_parent());
    let naive = bvh;

    let mut lights: Vec<Arc<Light>> = vec![
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
        //     RGBSpectrumf::new(0.0 as Float, 0.0 as Float, 1.0 as Float))
        // ), 
        // Arc::new(PointLight::new(
        //     Point3f::new(0.0 as Float, 20. as Float, 15. as Float),
        //     RGBSpectrumf::new(1000.7 as Float, 1000.0 as Float, 1000.0 as Float))
        // ),
        // Arc::new(SpotLight::new(
        //     // Point3f::new(-8.0 as Float, 20.0 as Float, 5.0 as Float),
        //     // Vector3f::new(8.0 as Float, -20.0 as Float, 10.0 as Float).normalize(),
        //     Point3f::new(0.0 as Float, 0.0 as Float, 30.0 as Float),
        //     Vector3f::new(0.0 as Float, -0.5 as Float, 0.0 as Float).normalize(),
        //     RGBSpectrumf::new(350.0 as Float, 73.0 as Float, 345.0 as Float),
        //     float::frac_pi_4()*0.75 as Float, float::frac_pi_4()*0.25 as Float)
        // ), 
        Arc::new(PointLight::new(
            Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float),
            RGBSpectrumf::new(40.0 as Float, 40.0 as Float, 40.0 as Float))
        ), 
    ];
    // lights.push(Arc::new(sphere1));
    lights.push(sphere2);
    
    for light in &lights {
        println!("pawa+{:?}", light.power().to_xyz().y);
    }

    let scene = Scene::new(lights, Arc::new(naive));
    let mut camera = PerspecCam::new(
        Matrix4f::identity(),
        BBox2f::new(
            Point2f::new(-1.0 as Float, -0.75 as Float), 
            Point2f::new(1.0 as Float, 1.0 as Float)
            // Point2f::new(-0.2 as Float, -0.2 as Float), 
            // Point2f::new(0.2 as Float, 0.2 as Float)
        ),
        0.1 as Float, 
        1000.0 as Float, 
        // float::pi()*2.0 as Float / 3.0 as Float, 
        float::frac_pi_2(),
        None, 
        Film::new(
            Point2::new(640, 600),
            BBox2f::new(
                Point2f::new(0.0 as Float, 0.0 as Float), 
                Point2f::new(1.0 as Float, 1.0 as Float)
            ), 
            Arc::new(
                // MitchellFilter::new(
                //     Vector2f::new(2.0 as Float, 2.0 as Float), 
                //     0.5 as Float, 0.25 as Float,
                // )
                LanczosSincFilter::new(
                    Vector2f::new(4.0 as Float, 4.0 as Float),
                    3.0 as Float,
                )
            )
        )
    );
    camera.look_from(
        Point3f::origin(),
        Point3f::new(0.0 as Float, 0.0 as Float, 155.0 as Float),
        Vector3f::unit_y()
    );

    // camera.look_from(
    //     Point3f::new(0.0 as Float, -3.0 as Float, 15. as Float),
    //     Point3f::new(0.0 as Float, 0. as Float, 30. as Float),
    //     Vector3f::unit_y()
    // );
    println!("vray_world: {:?}", camera.view_to_parent().transform_vector(
        Vector3f::unit_z()
    ));
    let mut renderer = PTRenderer::new(StrataSampler::new(1, 1, 8, rand::StdRng::new().unwrap()), Arc::new(camera), "target51.png", 8, false);

    // use arendur::sample;
    // let mut renderer = PTRenderer::new(sample::naive::Naive::new(16), Arc::new(camera), "mitsuba15s16_naive.png", 5, true);
    #[cfg(feature = "flame")]
    flame::start("Rendering");
    renderer.render(&scene);
    #[cfg(feature = "flame")]
    flame::end("Rendering");
    
    let duration = sudato.elapsed();
    println!("Done! Time used: {:.4}s", duration.as_secs() as f64 + (duration.subsec_nanos() as f64/1_000_000_000.0f64));
    #[cfg(feature = "flame")]
    flame::dump_html(&mut std::fs::File::create("flaming.html").unwrap()).unwrap();
}
