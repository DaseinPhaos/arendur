// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines a utility sink for sampler's memory management

use std;
use geometry::prelude::*;

pub type Sinkf = Sink<Float>;
pub type Sink2f = Sink<Point2f>;

#[derive(Debug, Clone)]
pub struct Sink<T> {
    inner: Vec<T>,
    // dimension count
    ndim: usize,
    // total sample count
    nsample: usize,
    // current dimension index
    idim: usize,
    // current sample index
    isample: usize,
}

impl<T: Copy> Sink<T> {
    /// Constructs a new sink from given dimension and sample count
    pub fn new(ndim: usize, nsample: usize) -> Sink<T> {
        // debug_assert!(nsample < std::usize::MAX as u64);
        // let nsample = nsample as usize;
        let inner = unsafe {
            vec![std::mem::uninitialized(); ndim * nsample]
        };
        Sink{
            inner: inner,
            ndim: ndim,
            nsample: nsample,
            idim: 0usize,
            isample: 0usize,
        }
    }

    /// Advance to next pixel, reset indexing
    #[inline]
    pub fn reset(&mut self) {
        self.idim = 0;
        self.isample = 0;
    }

    /// get sample value in next dimension
    #[inline]
    pub fn next_dim(&mut self) -> Option<T> {
        if self.idim >= self.ndim {
            None
        } else {
            let isample = self.isample;
            let idim = self.idim;
            self.idim += 1usize;
            Some((*self)[(isample, idim)])
        }
    }

    /// advance to next sample
    #[inline]
    pub fn next_sample(&mut self) -> bool {
        if self.isample + 1 >= self.nsample {
            false
        } else {
            self.isample += 1;
            true
        }
    }

    /// set sample index
    #[inline]
    pub fn set_sample_index(&mut self, idx: usize) -> bool {
        if idx >= self.nsample {
            false
        } else {
            self.isample = idx;
            true
        }
    }

    /// get total dimension
    #[inline]
    pub fn ndim(&self) -> usize {
        self.ndim
    }

    /// get total sample count
    #[inline]
    pub fn nsample(&self) -> usize {
        self.nsample
    }

    /// get current dim
    #[inline]
    #[allow(dead_code)]
    pub fn idim(&self) -> usize {
        self.idim
    }

    /// get current sample
    #[inline]
    #[allow(dead_code)]
    pub fn isample(&self) -> usize {
        self.isample
    }
}

impl<T> std::ops::Index<(usize, usize)> for Sink<T> {
    type Output = T;
    
    /// index in (isample, idim)
    #[inline]
    fn index(&self, index: (usize, usize)) -> &T {
        &self.inner[index.0 * self.ndim + index.1]
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for Sink<T> {
    /// index in (isample, idim)
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut T {
        &mut self.inner[index.0 * self.ndim + index.1]
    }
}

impl<T> std::ops::Index<usize> for Sink<T> {
    type Output = [T];
    
    #[inline]
    fn index(&self, index: usize) -> &[T] {
        let offset = index * self.ndim;
        &self.inner[offset..(offset+self.ndim)]
    }
}

impl<T> std::ops::IndexMut<usize> for Sink<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut [T] {
        let offset = index * self.ndim;
        &mut self.inner[offset..(offset+self.ndim)]
    }
}
