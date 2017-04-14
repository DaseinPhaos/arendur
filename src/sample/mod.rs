//! The sampling and filtering interface

// TODO: add more samplers

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

    /// request `n` samples in place
    /// Default implementation uses succeeding calls to `self.next()`
    /// to fill the `buf`, which might not be ideal
    fn request(&mut self, buf: &mut [Float]) {
        for f in buf.iter_mut() {
            *f = self.next();
        }
    }

    /// request `n` 2d samples in place
    /// Default implementation uses succeeding calls to `self.next_2d()`
    /// to fill the `buf`, which might not be ideal
    fn request_2d(&mut self, buf: &mut [Point2f]) {
        for p in buf.iter_mut() {
            *p = self.next_2d();
        }
    }

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
    fn sample_per_pixel(&self) -> usize;

    /// Try advance to the next sample
    /// `true` if the sampling process can continue
    /// `false` when overflowing `sample_per_pixel`, eg
    fn next_sample(&mut self) -> bool;

    /// set current sample to a particular index
    fn set_sample_index(&mut self, idx: usize) -> bool;
}

/// The filter interface
/// A filter always lies at $(0, 0)$ in its local frame.

pub trait Filter {
    /// Returns the filter's radius
    /// The filter's support in local frame is thus given
    /// by $[-radius.x, radius.x]\times [-radius.y, radius.y]$
    fn radius(&self) -> Vector2f;

    /// Returns the filter's support as a bounding box, in local frame
    #[inline]
    fn support(&self) -> BBox2f {
        let p = self.radius();
        BBox2f::new(Point2f::from_vec(p), Point2f::from_vec(-p))
    }

    /// Evaluate the filter at `p` in its local frame.
    /// Caller MUST ensure that `p` lies inside the support of `self`.
    /// This method is thus marked as `unsafe`.
    unsafe fn evaluate_unsafe(&self, p: Point2f) -> Float;

    /// Evluate the filter at `p` in its local frame.
    /// Point outside support is checked.
    #[inline]
    fn evaluate(&self, p: Point2f) -> Float {
        let bounding = self.support();
        if bounding.contain(p) {unsafe {
            self.evaluate_unsafe(p)
        }} else {
            0.0 as Float
        }
    }
}

pub mod naive;
pub mod strata;
pub mod filters;
mod sink;