//! A naive sampler.
#![allow(unused_variables)]

extern crate rand;
use self::rand::Rng;
use geometry::prelude::*;
use std::u64::MAX;
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
    /// Start sampling a new pixel
    fn start_pixel(&mut self, p: Point2<u32>) { }

    /// get next 1-dimensional sample
    fn next(&mut self) -> Float {
        self.rng.gen_range(0.0 as Float, 1.0 as Float)
    }

    /// prequest `n` 1d samples
    fn prequest(&mut self, n: u32) {}

    /// prequest `n` 2d samples
    fn prequest_2d(&mut self, n: u32) {}

    // /// request the prequest, `n` to checksum
    // fn request(&mut self, n: u32) -> &[Float];

    // /// request the prequest, `n` to checksum
    // fn request(&mut self, n: u32) -> &[Point2f];

    /// maximum sample count per pixel
    fn sample_per_pixel(&self) -> u64 {
        MAX
    }

    /// Try advance to the next sample
    /// `true` if the sampling process can continue
    /// `false` when overflowing `sample_per_pixel`, eg
    fn next_sample(&mut self) -> bool { true }

    /// set current sample to a particular index
    fn set_sample_index(&mut self, idx: u64) -> bool {true }
}
