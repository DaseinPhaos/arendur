// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cgmath;
extern crate image;
extern crate num_traits;
extern crate copy_arena;

extern crate rayon;

pub mod geometry;
pub mod shape;
pub mod component;
pub mod spectrum;
pub mod filming;
pub mod sample;
pub mod bxdf;
pub mod material;
pub mod texturing;
pub mod lighting;
pub mod renderer;