//! The sampling interface

use geometry::prelude::*;
use filming;

/// The sampling interface
/// Smaplers return sampled values in $[0, 1)$
pub trait Sampler: Clone {
    /// Start sampling a new pixel
    fn start_pixel(&mut self, p: Point2<u32>);

    /// get next 1-dimensional sample
    fn next(&mut self) -> Float;

    /// get next 2-dimensional sample
    #[inline]
    fn next_2d(&mut self) -> Point2f {
        Point2f::new(self.next(), self.next())
    }

    /// convinient method to sample a camera
    #[inline]
    fn get_camera_sample(&mut self, idx: Point2<u32>) -> filming::SampleInfo {
        filming::SampleInfo{
            pfilm:  self.next_2d() + idx.cast().to_vec(),
            plens: self.next_2d(),
        }
    }

    /// prequest `n` 1d samples
    fn prequest(&mut self, n: u32);

    /// prequest `n` 2d samples
    fn prequest_2d(&mut self, n: u32);

    // /// request the prequest, `n` to checksum
    // fn request(&mut self, n: u32) -> &[Float];

    // /// request the prequest, `n` to checksum
    // fn request(&mut self, n: u32) -> &[Point2f];

    /// Optimal round count, as a hint
    #[inline]
    fn round_count(&self, n: u32) -> u32 {
        n
    }

    /// maximum sample count per pixel
    fn sample_per_pixel(&self) -> u64;

    /// Try advance to the next sample
    /// `true` if the sampling process can continue
    /// `false` when overflowing `sample_per_pixel`, eg
    fn next_sample(&mut self) -> bool;

    /// set current sample to a particular index
    fn set_sample_index(&mut self, idx: u64) -> bool;
}


pub mod naive;
pub mod strata;
mod sink;