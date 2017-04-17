// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! whitted renderer usage example

extern crate arender;
extern crate cgmath;

use arender::renderer::Renderer;
use arender::renderer::scene::Scene;
use arender::renderer::whitted::WhittedRenderer;
use arender::geometry::prelude::*;
use arender::component::{Composable, Primitive};
use arender::shape::sphere::*;
use arender::shape::*;
use arender::component::shape::*;
use arender::component::transformed::*;
use arender::material::*;

use 

use std::sync::Arc;

fn main() {
    println!("Whitted example");
    let transform0 = Matrix4f::from_translation(Vector3f::new(10.0 as Float, 10.0 as Float, 10.0 as Float));
    let transform1 = Matrix4f::from_translation(Vector3f::new(-10.0 as Float, -10.0 as Float, -10.0 as Float));
    let inv_transform0 = transform0.invert();
    let inv_transform1 = transform1.invert();
    
    let sphere0 = Sphere::new(SphereInfo::new(4.8 as Float, -2.4 as Float, 5.0 as Float, 6.49 as Float), ShapeInfo::new(&transform0, &inv_transform0, false));
    let sphere1 = Sphere::new(SphereInfo::full(2.4 as Float, -2.4 as Float), ShapeInfo::new(&transform1, &inv_transform1, false));

    let material0 = matte::MatteMaterial::new()

    let sphere0 = ShapedPrimitive::new(Arc::new(sphere0), )
}
