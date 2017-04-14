//! A naive sampler.
#![allow(unused_variables)]

extern crate rand;
use self::rand::Rng;
use geometry::prelude::*;
use std::usize::MAX;
use super::Sampler;

// A naive sampler
#[derive(Clone)]
pub struct Naive {
    rng: rand::ThreadRng,
}

impl Naive {
    #[inline]
    pub fn new() -> Naive {
        Naive {
            rng: rand::thread_rng()
        }
    }
}

impl Default for Naive {
    #[inline]
    fn default() -> Self {
        Naive::new()
    }
}

impl Sampler for Naive {
    fn start_pixel(&mut self, p: Point2<u32>) { }

    fn next(&mut self) -> Float {
        self.rng.gen_range(0.0 as Float, 1.0 as Float)
    }

    fn sample_per_pixel(&self) -> usize {
        MAX
    }

    fn next_sample(&mut self) -> bool { true }

    fn set_sample_index(&mut self, idx: usize) -> bool {true }
}
