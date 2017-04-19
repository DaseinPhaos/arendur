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

use arendur::spectrum::*;
use arendur::filming::*;
use arendur::filming::film::*;
use arendur::filming::perspective::PerspecCam;
use arendur::renderer::Renderer;
use arendur::renderer::scene::Scene;
use arendur::renderer::whitted::WhittedRenderer;
use arendur::geometry::prelude::*;
use arendur::component::{Composable, Primitive};
use arendur::shape::sphere::*;
use arendur::shape::*;
use arendur::component::shape::*;
use arendur::component::transformed::*;
use arendur::material::*;
use arendur::texturing::*;
use arendur::texturing::textures::ConstantTexture;
use arendur::lighting::*;
use arendur::sample::*;
use arendur::sample::strata::StrataSampler;
use arendur::sample::filters::*;
type NaiveAggregate = arendur::component::naive::Naive;


use std::sync::Arc;

fn main() {
    println!("Whitted example");
    use std::io;
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    let transform0 = Arc::new(Matrix4f::from_translation(Vector3f::new(2.0 as Float, 2.0 as Float, 30.0 as Float)));
    let transform1 = Arc::new(Matrix4f::from_translation(Vector3f::new(-4.0 as Float, -4.0 as Float, -40.0 as Float)));
    let inv_transform0 = Arc::new(transform0.invert().unwrap());
    let inv_transform1 = Arc::new(transform1.invert().unwrap());
    
    let sphere0 = Sphere::full(20. as Float);
    // let sphere0 = Sphere::new(SphereInfo::new(20. as Float, -28. as Float, 58. as Float, 6.49 as Float), ShapeInfo::new(transform0, inv_transform0, false));
    let sphere1 = Sphere::full(15. as Float);

    let kd = ConstantTexture{value: RGBSpectrumf::new(10.0 as Float, 10.0 as Float, 10.0 as Float)};
    let sigma = ConstantTexture{value: 30.0 as Float};


    let material0 = matte::MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);

    let sphere0 = ShapedPrimitive::new(sphere0, material0, None);
    let sphere0 = TransformedComposable::new(sphere0, transform0, inv_transform0);

    let kd = ConstantTexture{value: RGBSpectrumf::new(0.01 as Float, 0.34 as Float, 0.4 as Float)};
    let sigma = ConstantTexture{value: 1.0 as Float};


    let material1 = matte::MatteMaterial::new(Arc::new(kd), Arc::new(sigma), None);

    let sphere1 = ShapedPrimitive::new(sphere1, material1, None);
    let sphere1 = TransformedComposable::new(sphere1, transform1, inv_transform1);

    let mut naive = NaiveAggregate::from_one(Arc::new(sphere0));
    naive.append(Arc::new(sphere1));

    let lights: Vec<Arc<Light>> = vec![
        Arc::new(pointlights::PointLight::new(
            Point3f::new(-10.0 as Float, 0.0 as Float, 0.0 as Float),
            RGBSpectrumf::new(50.7 as Float, 0.0 as Float, 50.0 as Float))
        ), 
        Arc::new(pointlights::PointLight::new(
            Point3f::new(10.0 as Float, 10.0 as Float, 0.0 as Float),
            RGBSpectrumf::new(50.7 as Float, 20.0 as Float, 5.0 as Float))
        ), 
        Arc::new(pointlights::PointLight::new(
            Point3f::new(0.0 as Float, -10.0 as Float, 0.0 as Float),
            RGBSpectrumf::new(1.7 as Float, 50.0 as Float, 5.0 as Float))
        ), 
        Arc::new(pointlights::PointLight::new(
            Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float),
            RGBSpectrumf::new(10.7 as Float, 10.0 as Float, 10.0 as Float))
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
        float::pi()*2.0 as Float / 3.0 as Float, 
        None, 
        Film::new(
            Point2::new(600, 400), 
            BBox2f::new(
                Point2f::new(0.0 as Float, 0.0 as Float), 
                Point2f::new(1.0 as Float, 1.0 as Float)
            ),
            Arc::new(
                BoxFilter::new(
                    Vector2f::new(2.0 as Float, 2.0 as Float), 
                    // 0.5 as Float, 0.25 as Float,
                )
            )
        )
    );
    let mut renderer = WhittedRenderer::new(StrataSampler::new(9, 9, 10, rand::StdRng::new().unwrap()), Arc::new(camera), "test.png");

    renderer.render(&scene);

    
}
