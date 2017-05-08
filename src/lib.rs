// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// required to properly transform primitives
#![feature(specialization)]
// required by `float::next_up` and `float::next_down`
#![feature(float_bits_conv)]

extern crate rand;
#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate cgmath;
extern crate image;
extern crate num_traits;
extern crate copy_arena;
extern crate tobj;
extern crate rayon;
#[cfg(feature = "flame")]
extern crate flame;

macro_rules! profile_use {
    () => (
        #[cfg(feature = "flame")]
        use flame;
    )
}

macro_rules! profile_start {
    ($name:expr) => {
        #[cfg(feature = "flame")]
        flame::start($name);
    }
}

macro_rules! profile_end {
    ($name:expr) => {
        #[cfg(feature = "flame")]
        flame::end($name);
    }
}

macro_rules! profile_dump {
    ($name:expr) => {
        #[cfg(feature = "flame")]
        {
            use std::fs::File;
            if let Ok(mut file) = File::create($name) {
                if let Ok(_) = flame::dump_html(&mut file) {
                    println!("Dumping profiling to {} succeeded.", $name);
                } else {
                    println!("Dumping profiling to {} failed.", $name);
                }
            } else {
                println!("Creating {} failed.", $name);
            }
        }
    }
}

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
pub mod prelude;
