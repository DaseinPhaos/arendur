// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A naive sampler.
#![allow(unused_variables)]

extern crate rand;
use self::rand::Rng;
use geometry::prelude::*;
use super::Sampler;

// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// A naive sampler. Who dare use it?
pub struct Naive {
    rng: rand::StdRng,
    nsample: usize,
    isample: usize,
}

impl Naive {
    #[inline]
    pub fn new(nsample: usize) -> Naive {
        Naive {
            rng: rand::StdRng::new().unwrap(), nsample, isample: 0,
        }
    }
}

impl Clone for Naive {
    #[inline]
    fn clone(&self) -> Self {
        Naive {
            rng: rand::StdRng::new().unwrap(),
            nsample: self.nsample,
            isample: 0,
        }
    }
}

impl Default for Naive {
    #[inline]
    fn default() -> Self {
        Naive::new(16)
    }
}

impl Sampler for Naive {
    fn start_pixel(&mut self, _p: Point2<u32>) {
        self.isample = 0;
    }

    fn next(&mut self) -> Float {
        self.rng.gen_range(0.0 as Float, 1.0 as Float)
    }

    fn sample_per_pixel(&self) -> usize {
        self.nsample
    }

    fn next_sample(&mut self) -> bool {
        if self.isample < self.nsample {
            self.isample += 1;
            true
        } else {
            false
        }
    }

    fn set_sample_index(&mut self, idx: usize) -> bool {
        self.isample = idx;
        if idx < self.nsample {
            true
        } else {
            false
        }
    }
}
