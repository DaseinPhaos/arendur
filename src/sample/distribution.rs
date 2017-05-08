// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines 1d and 2d distributions

use super::*;
use std::iter::FromIterator;
use std::cmp::Ordering;

/// A 1d distribution
#[derive(Serialize, Deserialize, Debug)]
pub struct Distribution1D {
    func: Vec<Float>,
    cdf: Vec<Float>,
    func_integral: Float,
}

impl Distribution1D {
    /// construction
    pub fn new<I>(func: I) -> Distribution1D
        where I: IntoIterator<Item=Float>,
    {
        let func: Vec<_> = func.into_iter().collect();
        let mut cdf = Vec::with_capacity(func.len() + 1);
        cdf.push(0. as Float);
        for i in 0..func.len() {
            let lastcdf = unsafe {
                *cdf.get_unchecked(i)
            };
            let curfunc = unsafe {
                *func.get_unchecked(i)
            };
            assert!(curfunc>=0. as Float);
            // if curfunc < 0. as Float { curfunc = 0. as Float; }
            cdf.push(lastcdf + curfunc);
        }
        let func_integral = unsafe {
            *cdf.get_unchecked(func.len())
        };
        if func_integral == 0. as Float {
            for i in 1..cdf.len() {
                unsafe {
                    *cdf.get_unchecked_mut(i) = i as Float / cdf.len() as Float;
                }
            }
        } else {
            for i in 1..cdf.len() {
                unsafe {
                    *cdf.get_unchecked_mut(i) /= func_integral;
                }
            }
        }
        Distribution1D{
            func: func,
            cdf: cdf,
            func_integral: func_integral,
        }
    }

    /// length
    #[inline]
    pub fn len(&self) -> usize {
        self.func.len()
    }

    /// given a uniform sample in $[0, 1)$, return 
    /// a sample as `(value, pdf, offset)`
    #[inline]
    pub fn sample_continuous(&self, u: Float) -> (Float, Float, usize) {
        let offset = self.search_offset(u);
        debug_assert!(offset < self.func.len());
        let floor = unsafe {
            *self.cdf.get_unchecked(offset)
        };
        let ceil = unsafe {
            *self.cdf.get_unchecked(offset + 1)
        };
        let mut du = u - floor;
        if ceil - floor > 0. as Float {
            du /= ceil - floor;
        }
        let pdf = if self.func_integral > 0. as Float {unsafe {
            *self.func.get_unchecked(offset) / self.func_integral
        }} else {
            0. as Float
        };
        let value = (offset as Float + du)/self.len() as Float;
        (value, pdf, offset)
    }

    /// giben a uniform sample in $[0, 1)$, return
    /// a discrete sample as `(offset, pdf, remapped_value)`
    #[inline]
    pub fn sample_discrete(&self, u: Float) -> (usize, Float, Float) {
        let offset = self.search_offset(u);
        debug_assert!(offset < self.func.len());
        let floor = unsafe {
            *self.cdf.get_unchecked(offset)
        };
        let ceil = unsafe {
            *self.cdf.get_unchecked(offset + 1)
        };
        let mut du = u - floor;
        if ceil - floor > 0. as Float {
            du /= ceil - floor;
        }
        let pdf = if self.func_integral > 0. as Float {unsafe {
            *self.func.get_unchecked(offset) / self.func_integral
        }} else {
            0. as Float
        };
        (offset, pdf, du)
    }

    #[inline]
    pub fn discrete_pdf(&self, index: usize) -> Float {
        self.func[index] / (self.func_integral * self.len() as Float)
    }

    #[inline]
    fn search_offset(&self, mut u: Float) -> usize {
        if u == 0. as Float { u += float::epsilon(); }
        self.cdf.binary_search_by(|v| {
            if *v < u { Ordering::Less }
            else if *v == u { Ordering::Equal }
            else { Ordering::Greater }
        }).unwrap_or_else(|v| v) - 1

        // let ret = self.cdf.binary_search_by(|v| {
        //     if *v < u { Ordering::Less }
        //     else if *v == u { Ordering::Equal }
        //     else { Ordering::Greater }
        // }).unwrap_or_else(|v| v);

        // if ret >= self.cdf.len() || ret == 0 {
        //     println!("CDF: {:?}", self.cdf);
        //     println!("u: {}, idx: {}", u, ret);
        // }
        // if ret == 0 {
        //     ret
        // } else {
        //     ret - 1
        // }
    }
}

impl FromIterator<Float> for Distribution1D {
    #[inline] 
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = Float>,
    {
        Distribution1D::new(iter)
    }
}

/// A 2d distribution
#[derive(Serialize, Deserialize, Debug)]
pub struct Distribution2D {
    pcv: Vec<Distribution1D>,
    pmarginal: Distribution1D,
}

impl Distribution2D {
    pub fn new(floats: &[Float], nu: usize) -> Distribution2D {
        let n = floats.len();
        assert!(nu < n);
        let nv = n / nu;
        let mut pcv = Vec::with_capacity(nv);
        let mut marginal_func = Vec::with_capacity(nv);
        for v in 0..nv {
            pcv.push(Distribution1D::new(floats[v*nu..(v+1)*nu].to_vec()));
            marginal_func.push(pcv.last().unwrap().func_integral);
        }
        let pmarginal = Distribution1D::new(marginal_func);
        Distribution2D{
            pcv: pcv,
            pmarginal: pmarginal,
        }
    }

    pub fn sample_continuous(&self, u: Point2f) -> (Point2f, Float) {
        let (d1, pdf1, v) = self.pmarginal.sample_continuous(u.y);
        let (d0, pdf0, _) = self.pcv[v].sample_continuous(u.x);
        (Point2f::new(d0, d1), pdf0 * pdf1)
    }

    pub fn pdf(&self, p: Point2f) -> Float {
        let n = self.pcv[0].len();
        let iu = (p.x * n as Float) as isize;
        let iu = if iu < 0 {
            0
        } else if iu > (n-1) as isize {
            n-1
        } else {
            iu as usize
        };
        let m = self.pmarginal.len();
        let iv = (p.y * m as Float) as isize;
        let iv = if iv < 0 {
            0
        } else if iv > (m-1) as isize {
            m-1
        } else {
            iv as usize
        };
        self.pcv[iv].func[iu] / self.pmarginal.func_integral
    }
}