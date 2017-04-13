//! Defines a utility sink for sampler's memory management

use std;
use num_traits::Zero;
use geometry::prelude::*;

pub type Sinkf = Sink<Float>;
pub type Sink2f = Sink<Point2f>;

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

impl<T: Copy + Zero> Sink<T> {
    /// Constructs a new sink from given dimension and sample count
    pub fn new(ndim: usize, nsample: u64) -> Sink<T> {
        debug_assert!(nsample < std::usize::MAX as u64);
        let nsample = nsample as usize;
        let mut inner = vec![<T as Zero>::zero(); ndim * nsample];
        Sink{
            inner: inner,
            ndim: ndim,
            nsample: nsample,
            idim: 0usize,
            isample: 0usize,
        }
    }

    /// get sample value in next dimension
    pub fn next_dim(&mut self) -> T {
        self.idim += 1usize;
        let isample = self.isample;
        let idim = self.idim;
        (*self)[(isample, idim)]
    }

    /// advance to next sample
    pub fn next_sample(&mut self) -> bool {
        if self.isample + 1 == self.nsample {
            false
        } else {
            self.isample += 1;
            true
        }
    }
}

impl<T> std::ops::Index<(usize, usize)> for Sink<T> {
    type Output = T;
    
    /// index in (isample, idim)
    #[inline]
    fn index(&self, index: (usize, usize)) -> &T {
        &self.inner[index.0 * self.ndim + self.idim]
    }
}

impl<T> std::ops::IndexMut<(usize, usize)> for Sink<T> {
    /// index in (isample, idim)
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut T {
        &mut self.inner[index.0 * self.ndim + self.idim]
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