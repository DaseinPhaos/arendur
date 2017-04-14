// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines a stratified sampler
extern crate rand;
use super::sink::{Sinkf, Sink2f};
use super::Sampler;
use self::rand::Rng;
use geometry::*;
use std;

/// Represents a stratified sampler
#[derive(Debug, Clone)]
pub struct StrataSampler<T> {
    sinkf: Sinkf,
    sink2f: Sink2f,
    sampledx: u32,
    sampledy: u32,
    rng: T,
}

impl<T: Rng> StrataSampler<T> {
    /// Construction
    pub fn new(sampledx: u32, sampledy: u32, ndim: u32, rng: T) -> StrataSampler<T> {
        let nsample = sampledx as usize * sampledy as usize;
        let sinkf = Sinkf::new(ndim as usize, nsample);
        let sink2f = Sink2f::new(ndim as usize, nsample);
        StrataSampler{
            sinkf: sinkf,
            sink2f: sink2f,
            sampledx: sampledx,
            sampledy: sampledy,
            rng: rng,
        }
    }

    /// generate a series of stratified samples in 1d
    fn generate_strata(&mut self, over: &mut [Float]) {
        let n = over.len();
        let inv_n = (1.0 as Float) / (n as Float);
        for (i, sample) in over.iter_mut().enumerate() {
            let i = i as Float;
            *sample = self.rng.gen_range(0.0 as Float, inv_n) + i * inv_n;
        }
        self.rng.shuffle(over);
    }

    /// generate a series of stratified samples in 2d
    fn generate_strata_2d(&mut self, over: &mut [Point2f]) {
        debug_assert!(self.sampledx as usize * self.sampledy as usize == over.len());
        let inv_x = (1.0 as Float) / (self.sampledx as Float);
        let inv_y = (1.0 as Float) / (self.sampledy as Float);
        let nx = self.sampledx;
        let ny = self.sampledy;
        let mut ptr = over.as_mut_ptr();
        for x in 0..nx {
            let x = x as Float * inv_x;
            for y in 0..ny {
                let y = y as Float * inv_y;
                let sx = x + self.rng.gen_range(0.0 as Float, inv_x);
                let sy = y + self.rng.gen_range(0.0 as Float, inv_y);
                unsafe {
                    std::ptr::write(ptr, Point2f::new(sx, sy));
                    ptr = ptr.offset(1);
                }
            }
        }
        self.rng.shuffle(over);
    }
}

impl<T: Rng + Clone> Sampler for StrataSampler<T> {
    fn start_pixel(&mut self, _p: Point2<u32>) {
        let nsample = self.sinkf.nsample();
        let ndim = self.sinkf.ndim();
        {
            let mut buf = unsafe {
                vec![std::mem::uninitialized(); nsample]
            };
            for idim in 0..ndim {
                self.generate_strata(&mut buf);
                for isample in 0..nsample {
                    self.sinkf[(isample, idim)] = buf[isample];
                }
            }
        }
        {
            let mut buf = unsafe {
                vec![std::mem::uninitialized(); nsample]
            };
            for idim in 0..ndim {
                self.generate_strata_2d(&mut buf);
                for isample in 0..nsample {
                    self.sink2f[(isample, idim)] = buf[isample];
                }
            }
        }
        self.sinkf.reset();
        self.sink2f.reset();
    }

    #[inline]
    fn next(&mut self) -> Float {
        let next = self.sinkf.next_dim();
        next.unwrap_or(self.rng.gen_range(0.0 as Float, 1.0 as Float))
    }

    #[inline]
    fn next_2d(&mut self) -> Point2f {
        let next = self.sink2f.next_dim();
        next.unwrap_or(Point2f::new(
            self.rng.gen_range(0.0 as Float, 1.0 as Float),
            self.rng.gen_range(0.0 as Float, 1.0 as Float)
        ))
    }

    #[inline]
    fn sample_per_pixel(&self) -> usize {
        self.sinkf.nsample()
    }

    #[inline]
    fn next_sample(&mut self) -> bool {
        self.sinkf.next_sample() && self.sink2f.next_sample()
    }

    #[inline]
    fn set_sample_index(&mut self, idx: usize) -> bool {
        self.sinkf.set_sample_index(idx) && self.sink2f.set_sample_index(idx)
    }

    #[inline]
    fn request(&mut self, buf: &mut [Float]) {
        self.generate_strata(buf);
    }

    #[inline]
    fn request_2d(&mut self, buf: &mut [Point2f]) {
        // use Latin-hypertube sampling
        // TODO: double check
        let mut tmp = unsafe {
            vec![std::mem::uninitialized(); buf.len()]
        };
        self.generate_strata(&mut tmp);
        for i in 0..tmp.len() {unsafe {
            buf.get_unchecked_mut(i).x = *tmp.get_unchecked(i);
        }}
        self.generate_strata(&mut tmp);
        for i in 0..tmp.len() {unsafe {
            buf.get_unchecked_mut(i).y = *tmp.get_unchecked(i);
        }}
    }
}

