// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines the bsdf (bidirectional scattering distribution function)
//! interface, which is an aggregation of several bxdfs
//! It also addresses the frame transformation problem.

use bxdf::*;
use geometry::prelude::*;
use spectrum::{RGBSpectrumf, Spectrum};
use std::cmp;

/// A bsdf
pub struct Bsdf<'a> {
    pub eta: Float,
    /// shading normal
    ns: Vector3f,
    /// geometry normal
    ng: Vector3f,
    /// shading tangent
    ts: Vector3f,
    /// shading bitangent
    bs: Vector3f,
    sink: BsdfSink<'a>,
}

impl<'a> Bsdf<'a> {
    /// construction
    #[inline]
    pub fn new(si: &SurfaceInteraction, eta: Float) -> Bsdf<'a> {
        let ts = si.shading_duv.dpdu.normalize();
        let ns = si.shading_norm;
        let bs = ns.cross(ts);
        let ng = si.basic.norm;
        Bsdf{
            eta: eta, ns: ns, ng: ng, ts: ts, bs: bs, sink: Default::default(),
        }
    }

    /// adding an bxdf
    #[inline]
    pub fn add(&mut self, bxdf: &'a Bxdf) {
        self.sink.add(bxdf);
    }

    /// returns how many bxdfs have `kind`
    #[inline]
    pub fn have_n(&self, kind: BxdfType) -> usize {
        let mut count = 0;
        for bxdf in self.sink.iter() {
            if bxdf.is(kind) {
                count += 1;
            }
        }
        count
    }

    /// transforms an vector from parent frame into local frame
    /// defined by the tangent space
    #[inline]
    pub fn parent_to_local(&self, v: Vector3f) -> Vector3f {
        Vector3f::new(v.dot(self.ts), v.dot(self.bs), v.dot(self.ns))
    }

    /// transforms an vector from local tangent frame into parent frame
    #[inline]
    pub fn local_to_parent(&self, v: Vector3f) -> Vector3f {
        Vector3f::new(
            v.dot(Vector3f::new(self.ts.x, self.bs.x, self.ns.x)),
            v.dot(Vector3f::new(self.ts.y, self.bs.y, self.ns.y)),
            v.dot(Vector3f::new(self.ts.z, self.bs.z, self.ns.z))
        )
    }

    /// evalute this bsdf. vectors given in parent frame
    pub fn evaluate(&self, wow: Vector3f, wiw: Vector3f, types: BxdfType) -> RGBSpectrumf {
        let wo = self.parent_to_local(wow);
        let wi = self.parent_to_local(wiw);
        let is_reflection = wow.dot(self.ng) * wiw.dot(self.ng) > 0.0 as Float;
        let mut ret = RGBSpectrumf::black();
        for bxdf in self.sink.iter() {
            if bxdf.is(types) && (
                (is_reflection && (bxdf.kind() & BXDF_REFLECTION == BXDF_REFLECTION))
                || (!is_reflection && (bxdf.kind() & BXDF_TRANSMISSION == BXDF_TRANSMISSION))
            ) {
                ret += bxdf.evaluate(wo, wi);
            }
        }
        ret
    }

    pub fn evalute_sampled(&self, wow: Vector3f, u: Point2f, types: BxdfType) -> (RGBSpectrumf, Vector3f, Float) {
        let match_count = self.have_n(types);
        let mut ret = (
            RGBSpectrumf::black(),
            Vector3f::new(0.0 as Float, 1.0 as Float, 0.0 as Float),
            0.0 as Float
        );
        if match_count == 0 { return ret; }
        
        let wo = self.parent_to_local(wow);
        let idx = cmp::min((u.x * match_count as Float).floor() as usize, match_count-1);
        let mut i = 0;
        for bxdf in self.sink.iter() {
            if i == idx {
                // sample the target now
                let (f, wi, mut pdf) = bxdf.evaluate_sampled(wo, u);
                if pdf == 0.0 as Float { return ret; }
                ret = (f, wi, pdf);
            }
            if bxdf.is(types) { i += 1; }
        }
        let wi = ret.1;
        ret.1 = self.local_to_parent(wi);
        
        if match_count == 1 { return ret; }

        let mut pdfsum = 0.0 as Float;
        for bxdf in self.sink.iter() {
            if bxdf.is(types) {
                pdfsum += bxdf.pdf(wo, wi);
            }
        }
        pdfsum /= match_count as Float;
        ret.2 /= pdfsum;
        ret
    }

    pub fn pdf(&self, wow: Vector3f, wiw: Vector3f, types: BxdfType) -> Float {
        let wo = self.parent_to_local(wow);
        let wi = self.parent_to_local(wiw);
        let mut pdfsum = 0.0 as Float;
        let mut match_count = 0;
        for bxdf in self.sink.iter() {
            if bxdf.is(types) {
                match_count += 1;
                pdfsum += bxdf.pdf(wo, wi);
            }
        }
        if match_count == 0 {
            pdfsum
        } else {
            pdfsum / match_count as Float
        }
    }

    pub fn rho_hd(&self, wow: Vector3f, samples: &[Point2f]) -> RGBSpectrumf {
        let wo = self.parent_to_local(wow);
        let mut ret = RGBSpectrumf::black();
        for bxdf in self.sink.iter() {
            ret += bxdf.rho_hd(wo, samples);
        }
        ret
    }

    pub fn rho_hh(&self, samples0: &[Point2f], samples1: &[Point2f]) -> RGBSpectrumf {
        let mut ret = RGBSpectrumf::black();
        for bxdf in self.sink.iter() {
            ret += bxdf.rho_hh(samples0, samples1);
        }
        ret
    }
}

struct BsdfSink<'a> {
    bxdfs: [Option<&'a Bxdf>; 8],
    n: usize,
}

impl<'a> Default for BsdfSink<'a> {
    fn default() -> BsdfSink<'a> {
        BsdfSink{
            bxdfs: [None; 8],
            n: 0,
        }
    }
}

impl<'a> BsdfSink<'a> {
    /// adding an bxdf
    #[inline]
    fn add(&mut self, bxdf: &'a Bxdf) {
        assert!(self.n < 8);
        let n = self.n;
        unsafe {
            *self.bxdfs.get_unchecked_mut(n) = Some(bxdf);
        }
        self.n += 1;
    }

    #[inline]
    fn iter<'b>(&'b self) -> BsdfSinkIter<'b, 'a> {
        BsdfSinkIter{
            sink: self,
            i: 0
        }
    }
}

struct BsdfSinkIter<'a, 'b: 'a> {
    sink: &'a BsdfSink<'b>,
    i: usize,
}

impl<'a, 'b: 'a> Iterator for BsdfSinkIter<'a, 'b> {
    type Item = &'a Bxdf;
    fn next(&mut self) -> Option<&'a Bxdf> {
        if self.i >= self.sink.n {
            None
        } else {
            let i = self.i;
            let ret = unsafe {
                self.sink.bxdfs.get_unchecked(i)
            };
            self.i += 1;
            *ret
        }
    }
}