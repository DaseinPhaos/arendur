// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines the image texture

use texturing::*;
use std::cmp::Eq;
use std::mem;
use std::cmp;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
extern crate image;
use self::image::GenericImage;
use spectrum::{RGBSpectrum, ToNorm, RGBSpectrumf, Spectrum};

/// an image texture
pub struct ImageTexture<TM: BaseNum + image::Primitive, M> {
    mapping: M,
    mipmap: Arc<MipMap<TM>>,
}

impl<TM: BaseNum + image::Primitive + ToNorm + 'static, M: Mapping2D> Texture for ImageTexture<TM, M> {
    type Texel = RGBSpectrumf;

    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        let t2dinfo = self.mapping.map(si, dxy);
        self.mipmap.look_up(t2dinfo.p, t2dinfo.dpdx, t2dinfo.dpdy).to_rgbf()
    }
}

impl<TM: BaseNum + image::Primitive + ToNorm + 'static, M: Mapping2D> ImageTexture<TM, M> {
    pub fn new(info: ImageInfo, mapping: M, ref_table: &mut HashMap<ImageInfo, Weak<MipMap<TM>>>) -> Option<Self> {
        // let entry = image_table_u8.entry(info);
        let try_strong = match ref_table.entry(info.clone()) {
            Entry::Occupied(oe) => {
                oe.get().clone().upgrade()
            },
            Entry::Vacant(_) => {
                None
            },
        };
        if let Some(mipmap) = try_strong {
            Some(ImageTexture{
                mapping: mapping,
                mipmap: mipmap,
            })
        } else {
            let mipmap = MipMap::new(info.clone());
            if let Some(mipmap) = mipmap {
                let mipmap = Arc::new(mipmap);
                ref_table.insert(info, Arc::downgrade(&mipmap));
                Some(ImageTexture{
                    mapping: mapping,
                    mipmap: mipmap,
                })
            } else {
                None
            }
        }
    }
}

pub struct MipMap<T: BaseNum + image::Primitive> {
    info: ImageInfo,
    pyramid: Vec<image::ImageBuffer<RGBSpectrum<T>, Vec<T>>>,
}

impl<T: BaseNum + image::Primitive + ToNorm + Zero + Copy + 'static> MipMap<T> {
    fn new(info: ImageInfo) -> Option<MipMap<T>> {
        // treat `info.name` as filename in this case
        if let Ok(opened) = image::open(info.name.clone()) {
            let (nx, ny) = opened.dimensions();
            let np2x = nx.next_power_of_two();
            let np2y = ny.next_power_of_two();

            let miplevels = if np2x > np2y {
                np2x.trailing_zeros() + 1
            } else {
                np2y.trailing_zeros() + 1
            };

            let mut pyramid = Vec::with_capacity(miplevels as usize);
            
            for i in 0..miplevels {
                let dx = cmp::min(np2x/(1<<i), 1);
                let dy = cmp::min(np2y/(1<<i), 1);
                let cb: Vec<T> = opened.resize_exact(
                    dx, dy, image::FilterType::Lanczos3
                ).to_rgb().into_raw().into_iter().map(|x| {
                    MipMap::convert_in(info.gamma, info.scale, x)
                }).collect();
                pyramid.push(image::ImageBuffer::from_raw(dx, dy, cb).unwrap());
            }

            Some(MipMap{
                info: info,
                pyramid: pyramid,
            })
        } else {
            None
        }
    }

    #[inline]
    fn convert_in<R: ToNorm>(gamma: bool, scale: Float, f: R) -> T {
        let f = f.to_norm();
        if gamma {
            <T as ToNorm>::from_norm(inverse_gamma_correct(f)*scale)
        } else {
            <T as ToNorm>::from_norm(f*scale)
        }
    }

    #[inline]
    fn texel(&self, miplevel: usize, p: Point2<usize>) -> RGBSpectrum<T> {
        let frame = &self.pyramid[miplevel];
        let (dx, dy) = frame.dimensions();
        let (dx, dy) = (dx as usize, dy as usize);
        let p = if p.x >= dx || p.y >= dy {
            match self.info.wrapping {
                WrapMode::Black => {
                    let z = <T as Zero>::zero();
                    return RGBSpectrum::new(z, z, z);
                },
                WrapMode::Clamp => {
                    (
                        if p.x >= dx {dx-1} else {p.x},
                        if p.y >= dy {dy-1} else {p.y}
                    )
                },
                WrapMode::Repeat => {
                    (p.x % dx, p.y % dy)
                },
            }
        } else { (p.x, p.y) };
        *frame.get_pixel(p.0 as u32, p.1 as u32)
    }

    fn look_up_tri(&self, st: Point2f, width: Float) -> RGBSpectrum<T> {
        let level = self.find_level(width);
        if level < 0.0 as Float {
            self.triangle_filter(0, st)
        } else if level >= (self.pyramid.len() - 1) as Float {
            self.triangle_filter(self.pyramid.len()-1, st)
        } else {
            let floor = level.floor();
            let flooru = floor as usize;
            let delta = level - floor;
            let floorc = self.triangle_filter(flooru, st);
            let ceilc = self.triangle_filter(flooru + 1, st);
            floorc.approx_lerp(ceilc, delta)
        }
    }

    fn triangle_filter(&self, miplevel: usize, st: Point2f) -> RGBSpectrum<T> {
        let (nx, ny) = self.pyramid[miplevel].dimensions();
        let s = st.x * nx as Float - 0.5 as Float;
        let t = st.y * ny as Float - 0.5 as Float;
        let s0 = s.floor() as usize;
        let t0 = t.floor() as usize;
        let ds = s - s.floor();
        let dt = t - t.floor();
        let one = 1.0 as Float;
        let ret = (one - ds) * (one - dt) * self.texel(miplevel, Point2::new(s0, t0)).to_rgbf() +
        (one - ds) * dt * self.texel(miplevel, Point2::new(s0, t0 + 1)).to_rgbf() +
        ds * (one - dt) * self.texel(miplevel, Point2::new(s0+1, t0)).to_rgbf() +
        ds * dt * self.texel(miplevel, Point2::new(s0+1, t0+1)).to_rgbf();
        RGBSpectrum::from_rgbf(ret)
    }

    fn look_up(&self, st: Point2f, dst0: Vector2f, dst1: Vector2f) -> RGBSpectrum<T> {
        if self.info.trilinear {
            let width = dst0.x.max(dst0.y).max(dst1.x).max(dst1.y);
            self.look_up_tri(st, width)
        } else {
            let (dstmin, mut dstmaj) = if dst0.magnitude2() < dst1.magnitude2() {
                (dst0, dst1)
            } else {
                (dst1, dst0)
            };
            let mut minor = dstmin.magnitude();
            let major = dstmaj.magnitude();
            if minor == 0.0 as Float {
                self.triangle_filter(0, st)
            } else {
                if minor * self.info.max_aniso < major {
                    let scale = major / (minor * self.info.max_aniso);
                    minor *= scale;
                    dstmaj *= scale;
                }
                let level = self.find_level(minor).max(0.0 as Float);
                let floor = level.floor();
                let delta = level - floor;
                let level = floor as usize;
                let floorc = self.ewa_filter(level, st, dstmin, dstmaj);
                let ceilc = self.ewa_filter(level + 1, st, dstmin, dstmaj);
                floorc.approx_lerp(ceilc, delta)
            }
        }
    }

    fn ewa_filter(&self, miplevel: usize, st: Point2f, dstmin: Vector2f, dstmaj: Vector2f) -> RGBSpectrum<T> {
        let (nx, ny) = self.pyramid[miplevel].dimensions();
        let (nxf, nyf) = (nx as Float, ny as Float);
        let s = st.x * nxf - 0.5 as Float;
        let t = st.y * nyf - 0.5 as Float;
        let dmins = dstmin.x * nxf;
        let dmint = dstmin.y * nyf;
        let dmajs = dstmaj.x * nxf;
        let dmajt = dstmaj.y * nyf;

        // compute coefficients
        let mut a = dmint * dmint + dmajt * dmajt + 1.0 as Float;
        let mut b = -2.0 as Float * (dmins * dmint + dmajs * dmajt);
        let mut c = dmins * dmins + dmajs * dmajs + 1.0 as Float;
        let inv_f = 1.0 as Float / (a*c -b*b*0.25 as Float);
        a *= inv_f;
        b *= inv_f;
        c *= inv_f;

        // compute bounding box
        let det = -b * b + 4 as Float * a * c;
        let inv2_det = 1.0 as Float / det * 2.0 as Float;
        let usqrt = (det*c).sqrt();
        let vsqrt = (det*a).sqrt();
        let s0 = (s - inv2_det * usqrt).ceil() as usize;
        let s1 = (s + inv2_det * usqrt).ceil() as usize;
        let t0 = (t - inv2_det * vsqrt).ceil() as usize;
        let t1 = (t + inv2_det * vsqrt).ceil() as usize;

        let mut sum = RGBSpectrumf::black();
        let mut sumwt = 0.0 as Float;
        for it in t0..t1 {
            let tt = it as Float - s;
            for is in s0..s1 {
                let ss = is as Float - t;
                let square_radius = a * ss * ss + b * ss * tt + c * tt * tt;
                if square_radius < 1.0 as Float {
                    let idx = (square_radius * WEIGHT_LUT_SIZE as Float) as usize;
                    let idx = cmp::min(idx, WEIGHT_LUT_SIZE - 1);
                    let weight = WEIGHT_LUT[idx];
                    sum += self.texel(miplevel, Point2::new(is, it)).to_rgbf() * weight;
                    sumwt += weight;
                }
            }
        }
        RGBSpectrum::from_rgbf(sum/sumwt)
    }

    #[inline]
    fn find_level(&self, width: Float) -> Float {
        // find an level such that $width\times width$ covers about
        // four texels
        let width = width.max(1e-8 as Float).log2();
        (self.pyramid.len() - 1) as Float + width
    }

}

#[derive(PartialEq, Clone)]
pub struct ImageInfo {
    pub name: String,
    pub trilinear: bool,
    pub max_aniso: Float,
    pub wrapping: WrapMode,
    pub gamma: bool,
    pub scale: Float,
}

impl Hash for ImageInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.trilinear.hash(state);
        unsafe {
            // FIXME: transmution would break on changing typedef
            (mem::transmute::<Float, u32>(self.max_aniso)).hash(state);
            (mem::transmute::<Float, u32>(self.scale)).hash(state);
        }
        self.wrapping.hash(state);
        self.gamma.hash(state);
    }
}

impl Eq for ImageInfo { }

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum WrapMode {
    Repeat,
    Black,
    Clamp,
}

fn gamma_correct(v: Float) -> Float {
    if v <= 0.0031308 as Float {
        12.92 as Float * v
    } else {
        1.055 as Float * v.powf(1.0 as Float / 2.4 as Float) - 0.055 as Float
    }
}

fn inverse_gamma_correct(v: Float) -> Float {
    if v <= 0.04045 as Float {
        v * (1.0 as Float / 12.92 as Float)
    } else {
        ((1.0 as Float / 1.055 as Float) * v).powf(2.4 as Float)
    }
}

// static mut image_arena: HashMap<ImageInfo, Rc<& MipMap>>

const WEIGHT_LUT_SIZE: usize = 128;

lazy_static! {
    static ref WEIGHT_LUT: Vec<Float> = {
        let mut v = Vec::with_capacity(WEIGHT_LUT_SIZE);
        for i in 0..WEIGHT_LUT_SIZE {
            let alpha = 2.0 as Float;
            let r2 = i as Float / ((WEIGHT_LUT_SIZE - 1) as Float);
            v.push((-alpha * r2).exp() - (-alpha).exp());
        }
        v
    };
}