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
use self::image::Pixel;
use self::image::Luma;
use spectrum::{RGBSpectrum, ToNorm};
use num_traits::NumCast;
use sample::distribution::Distribution2D;

/// an image texture
pub struct ImageTexture<TM, TP, M>
    where TM: BaseNum + image::Primitive,
          TP: Pixel<Subpixel=TM>,
{
    mapping: M,
    mipmap: Arc<MipMap<TM, TP>>,
}


pub type RGBImageTexture<TM, M> = ImageTexture<TM, RGBSpectrum<TM>, M>;
pub type LumaImageTexture<TM, M> = ImageTexture<TM, Luma<TM>, M>;
pub type RGBMipMapHashTable<TM> = HashMap<ImageInfo, Weak<MipMap<TM, RGBSpectrum<TM>>>>;
pub type LumaMipMapHashTable<TM> = HashMap<ImageInfo, Weak<MipMap<TM, Luma<TM>>>>;

impl<TM, TP, M> ImageTexture<TM, TP, M>
    where TM: BaseNum + image::Primitive + 'static,
          TP: Pixel<Subpixel=TM> + 'static,
          M: Mapping2D
{
    /// Return distributions of this texture
    pub fn distribution<F>(&self, f: F) -> Distribution2D
        where F: FnMut(&TP) -> Float
    {
        let (u, _) = self.mipmap.pyramid[0].dimensions();
        let floats: Vec<_> = self.mipmap.pyramid[0].pixels().map(f).collect();
        Distribution2D::new(&floats, u as usize)
    }
}

// unsafe impl<T: BaseNum + image::Primitive, M> Sync for ImageTexture<T, M> { }
// unsafe impl<T: BaseNum + image::Primitive, M> Send for ImageTexture<T, M> { }

impl<TM, M> Texture for ImageTexture<TM, RGBSpectrum<TM>, M>
    where TM: BaseNum + image::Primitive + ToNorm + 'static + Send + Sync,
          M: Mapping2D + Send + Sync,
{
    type Texel = RGBSpectrum<TM>;

    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        let t2dinfo = self.mapping.map(si, dxy);
        self.mipmap.look_up(t2dinfo.p, t2dinfo.dpdx, t2dinfo.dpdy)
    }

    #[inline]
    fn mean(&self) -> Self::Texel {
        self.mipmap.mean
    }
}

impl<TM, M> Texture for ImageTexture<TM, Luma<TM>, M>
    where TM: BaseNum + image::Primitive + ToNorm + 'static + Send + Sync,
          M: Mapping2D + Send + Sync,
{
    type Texel = TM;

    #[inline]
    fn evaluate(&self, si: &SurfaceInteraction, dxy: &DxyInfo) -> Self::Texel {
        let t2dinfo = self.mapping.map(si, dxy);
        self.mipmap.look_up(t2dinfo.p, t2dinfo.dpdx, t2dinfo.dpdy).data[0]
    }

    #[inline]
    fn mean(&self) -> Self::Texel {
        self.mipmap.mean.data[0]
    }
}

impl<TM, M> ImageTexture<TM, RGBSpectrum<TM>, M>
    where TM: BaseNum + image::Primitive + ToNorm + 'static + Send + Sync,
          M: Mapping2D + Send + Sync + 'static,
{
    /// Contructing a new texture with image described by `info`.
    /// The actual image would be looked up from `ref_table`.
    /// If the `ref_table` don't contain such an image, an attempt
    /// to construct one would be made. If succeed, the texture would
    /// be returned.
    pub fn new(
        info: ImageInfo,
        mapping: M, 
        ref_table: &mut RGBMipMapHashTable<TM>
    ) -> Option<Self> {
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
            let mipmap = MipMap::<TM, RGBSpectrum<TM>>::new(info.clone());
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

    pub fn new_as_arc(
        info: ImageInfo,
        mapping: M, 
        ref_table: &mut RGBMipMapHashTable<TM>
    ) -> Option<Arc<Texture<Texel=RGBSpectrum<TM>>>> {
        if let Some(i) = RGBImageTexture::new(info, mapping, ref_table) {
            Some(Arc::new(i))
        } else {
            None
        }
    }
}

impl<TM, M> ImageTexture<TM, Luma<TM>, M>
    where TM: BaseNum + image::Primitive + ToNorm + 'static + Send + Sync,
          M: Mapping2D + Send + Sync + 'static,
{
    /// Contructing a new texture with image described by `info`.
    /// The actual image would be looked up from `ref_table`.
    /// If the `ref_table` don't contain such an image, an attempt
    /// to construct one would be made. If succeed, the texture would
    /// be returned.
    pub fn new(
        info: ImageInfo,
        mapping: M, 
        ref_table: &mut LumaMipMapHashTable<TM>
    ) -> Option<Self> {
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
            let mipmap = MipMap::<TM, Luma<TM>>::new(info.clone());
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

    pub fn new_as_arc(
        info: ImageInfo,
        mapping: M, 
        ref_table: &mut LumaMipMapHashTable<TM>
    ) -> Option<Arc<Texture<Texel=TM>>> {
        if let Some(i) = LumaImageTexture::new(info, mapping, ref_table) {
            Some(Arc::new(i))
        } else {
            None
        }
    }
}

pub struct MipMap<TM: BaseNum + image::Primitive, TP: Pixel<Subpixel=TM>> {
    info: ImageInfo,
    pyramid: Vec<image::ImageBuffer<TP, Vec<TM>>>,
    mean: TP,
}

impl<T> MipMap<T, RGBSpectrum<T>>
    where T: BaseNum + image::Primitive + ToNorm + Zero + Copy + 'static,
{
    /// load a new mipmap with infomation given by `info`
    fn new(info: ImageInfo) -> Option<MipMap<T, RGBSpectrum<T>>> {
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
                let dx = cmp::max(np2x/(1<<i), 1);
                let dy = cmp::max(np2y/(1<<i), 1);
                let cb: Vec<T> = opened.resize_exact(
                    dx, dy, image::FilterType::Lanczos3
                ).to_rgb().into_raw().into_iter().map(|x| {
                    MipMap::convert_in(info.gamma, info.scale, x)
                }).collect();
                pyramid.push(image::ImageBuffer::from_raw(dx, dy, cb).unwrap());
            }

            let z = <T as Zero>::zero();
            let slice = [z, z, z, z];
            let mut sum = *RGBSpectrum::from_slice(&slice);
            let mut count = 0u32;
            for p in pyramid[0].pixels() {
                sum = add_two(sum, p);
                count += 1;
            }
            let inv_count = 1. as Float / count as Float;

            Some(MipMap{
                info: info,
                pyramid: pyramid,
                mean: mul_float(sum, inv_count),
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

    pub fn save(&self, idx: usize, name: &str) {
        let buf = self.pyramid[idx].clone();
        let dim = buf.dimensions();
        let buf = buf.into_raw();
        let mut target: Vec<u8> = Vec::with_capacity(buf.len());
        for i in buf {
            let inorm = i.to_norm();
            target.push(<u8 as ToNorm>::from_norm(inorm));
        }
        let target = image::RgbImage::from_raw(dim.0, dim.1, target).unwrap();
        target.save(name).unwrap();
    }
}

impl<T> MipMap<T, Luma<T>>
    where T: BaseNum + image::Primitive + ToNorm + Zero + Copy + 'static,
{
    /// load a new mipmap with infomation given by `info`
    fn new(info: ImageInfo) -> Option<MipMap<T, Luma<T>>> {
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
                let dx = cmp::max(np2x/(1<<i), 1);
                let dy = cmp::max(np2y/(1<<i), 1);
                let cb: Vec<T> = opened.resize_exact(
                    dx, dy, image::FilterType::Lanczos3
                ).to_luma().into_raw().into_iter().map(|x| {
                    MipMap::convert_in(info.gamma, info.scale, x)
                }).collect();
                pyramid.push(image::ImageBuffer::from_raw(dx, dy, cb).unwrap());
            }

            let z = <T as Zero>::zero();
            let mut sum = Luma{data:[z]};
            let mut count = 0u32;
            for p in pyramid[0].pixels() {
                sum = add_two(sum, p);
                count += 1;
            }
            let inv_count = 1. as Float / count as Float;

            Some(MipMap{
                info: info,
                pyramid: pyramid,
                mean: mul_float(sum, inv_count),
            })
        } else {
            None
        }
    }

    pub fn save(&self, idx: usize, name: &str) {
        let buf = self.pyramid[idx].clone();
        let dim = buf.dimensions();
        let buf = buf.into_raw();
        let mut target: Vec<u8> = Vec::with_capacity(buf.len());
        for i in buf {
            let inorm = i.to_norm();
            target.push(<u8 as ToNorm>::from_norm(inorm));
        }
        let target = image::GrayImage::from_raw(dim.0, dim.1, target).unwrap();
        target.save(name).unwrap();
    }
}

impl<T, TP> MipMap<T, TP>
    where T: BaseNum + image::Primitive + ToNorm + Zero + Copy + 'static,
          TP: Pixel<Subpixel=T> + 'static
{
    #[inline]
    fn texel_isize(&self, miplevel: usize, p: Point2<isize>) -> TP {
        let frame = &self.pyramid[miplevel];
        let (dx, dy) = frame.dimensions();
        let (dx, dy) = (dx as usize, dy as usize);
        let p = if p.x as usize >= dx || p.y as usize >= dy {
            match self.info.wrapping {
                ImageWrapMode::Black => {
                    let z = <T as Zero>::zero();
                    let slice = [z, z, z, z];
                    return *TP::from_slice(&slice);
                },
                ImageWrapMode::Clamp => {
                    (
                        if p.x as usize >= dx {dx-1} else {p.x as usize},
                        if p.y as usize >= dy {dy-1} else {p.y as usize}
                    )
                },
                ImageWrapMode::Repeat => {
                    (
                        (p.x % dx as isize).abs() as usize,
                        (p.y % dy as isize).abs() as usize
                    )
                },
            }
        } else { (p.x as usize, p.y as usize) };
        *frame.get_pixel(p.0 as u32, p.1 as u32)
    }

    #[inline]
    fn texel(&self, miplevel: usize, p: Point2<usize>) -> TP {
        let frame = &self.pyramid[miplevel];
        let (dx, dy) = frame.dimensions();
        let (dx, dy) = (dx as usize, dy as usize);
        let p = if p.x >= dx || p.y >= dy {
            match self.info.wrapping {
                ImageWrapMode::Black => {
                    let z = <T as Zero>::zero();
                    let slice = [z, z, z, z];
                    return *TP::from_slice(&slice);
                },
                ImageWrapMode::Clamp => {
                    (
                        if p.x >= dx {dx-1} else {p.x},
                        if p.y >= dy {dy-1} else {p.y}
                    )
                },
                ImageWrapMode::Repeat => {
                    (
                        p.x % dx,
                        p.y % dy
                    )
                },
            }
        } else { (p.x, p.y) };
        *frame.get_pixel(p.0 as u32, p.1 as u32)
    }

    fn look_up_tri(&self, st: Point2f, width: Float) -> TP {
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
            approx_lerp(floorc, &ceilc, delta)
        }
    }

    fn triangle_filter(&self, miplevel: usize, st: Point2f) -> TP {
        let (nx, ny) = self.pyramid[miplevel].dimensions();
        let s = st.x * nx as Float - 0.5 as Float;
        let t = st.y * ny as Float - 0.5 as Float;
        let s0 = s.floor() as usize;
        let t0 = t.floor() as usize;
        let ds = s - s.floor();
        let dt = t - t.floor();
        let one = 1.0 as Float;
        add_two(
            add_two(
                mul_float(self.texel(miplevel, Point2::new(s0, t0)), (one - ds) * (one - dt)),
                &mul_float(self.texel(miplevel, Point2::new(s0, t0 + 1)), (one - ds) * dt)
            ),
            &add_two(
                mul_float(self.texel(miplevel, Point2::new(s0+1, t0)), ds * (one - dt)),
                &mul_float(self.texel(miplevel, Point2::new(s0+1, t0+1)), ds * dt)
            )
        )
    }

    fn look_up(&self, st: Point2f, dst0: Vector2f, dst1: Vector2f) -> TP {
        if self.info.trilinear {
            let width = dst0.x.max(dst0.y).max(dst1.x).max(dst1.y);
            self.look_up_tri(st, width)
        } else {
            let (mut dstmin, dstmaj) = if dst0.magnitude2() < dst1.magnitude2() {
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
                    dstmin *= scale;
                }
                let level = self.find_level(minor).max(0.0 as Float);
                let floor = level.floor();
                let delta = level - floor;
                let level = floor as usize;
                let floorc = self.ewa_filter(level, st, dstmaj, dstmin);
                let ceilc = self.ewa_filter(level + 1, st, dstmaj, dstmin);
                approx_lerp(floorc, &ceilc, delta)
            }
        }
    }

    fn ewa_filter(&self, miplevel: usize, st: Point2f, dstmaj: Vector2f, dstmin: Vector2f) -> TP {
        if miplevel >= self.pyramid.len() {
            return self.texel(self.pyramid.len() -1, Point2::new(0, 0));
        }
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
        let det = -b * b + 4. as Float * a * c;
        let inv2_det = 1.0 as Float / det * 2.0 as Float;
        let usqrt = (det*c).sqrt();
        let vsqrt = (det*a).sqrt();
        let s0 = (s - inv2_det * usqrt).ceil() as isize;
        let s1 = (s + inv2_det * usqrt).ceil() as isize;
        let t0 = (t - inv2_det * vsqrt).ceil() as isize;
        let t1 = (t + inv2_det * vsqrt).ceil() as isize;
        let z = <T as Zero>::zero();
        let slice = [z, z, z, z];
        let mut sum = *TP::from_slice(&slice);
        let mut sumwt = 0.0 as Float;
        for it in t0..t1+1 {
            let tt = it as Float - t;
            for is in s0..s1+1 {
                let ss = is as Float - s;
                let square_radius = a * ss * ss + b * ss * tt + c * tt * tt;
                if square_radius < 1.0 as Float {
                    let idx = (square_radius * WEIGHT_LUT_SIZE as Float) as usize;
                    let idx = cmp::min(idx, WEIGHT_LUT_SIZE - 1);
                    let weight = WEIGHT_LUT[idx];
                    debug_assert!(!weight.is_nan());
                    let to_add = mul_float(self.texel_isize(miplevel, Point2::new(is, it)), weight);
                    sum.apply2(&to_add, |a, b| a+b);
                    sumwt += weight;
                }
            }
        }
        debug_assert!(!sumwt.is_nan());
        mul_float(sum, 1.0 as Float / sumwt)
    }

    #[inline]
    fn find_level(&self, width: Float) -> Float {
        // find an level such that $width\times width$ covers about
        // four texels
        let width = width.max(1e-8 as Float).log2();
        (self.pyramid.len() - 1) as Float * width
    }
}

#[inline]
fn approx_lerp<TM, TP>(pix0: TP, pix1: &TP, t: Float) -> TP
    where TP: Pixel<Subpixel=TM>,
          TM: BaseNum + image::Primitive + Copy,
{
    pix0.map2(pix1, |a, b| {
        let a: Float = <Float as NumCast>::from(a).unwrap();
        let b: Float = <Float as NumCast>::from(b).unwrap();
        <TM as NumCast>::from(a*(1.0 as Float - t) + b * t).unwrap()
    })
}

#[inline]
fn mul_float<TM, TP>(pix: TP, f: Float) -> TP
    where TP: Pixel<Subpixel=TM>,
          TM: BaseNum + image::Primitive + Copy,
{
    pix.map(|a| {
        let a : Float = <Float as NumCast>::from(a).unwrap();
        <TM as NumCast>::from(a*f).unwrap()
    })   
}

#[inline]
fn add_two<TM, TP>(pix0: TP, pix1: &TP) -> TP 
    where TP: Pixel<Subpixel=TM>,
          TM: BaseNum + image::Primitive + Copy,
{
    pix0.map2(&pix1, |a, b| a+b)
}

/// Information abount an image
#[derive(PartialEq, Clone, Deserialize, Serialize)]
pub struct ImageInfo {
    pub name: String,
    pub trilinear: bool,
    pub max_aniso: Float,
    pub wrapping: ImageWrapMode,
    pub gamma: bool,
    pub scale: Float,
}

impl Hash for ImageInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.trilinear.hash(state);
        unsafe {
            (mem::transmute::<Float, FSize>(self.max_aniso)).hash(state);
            (mem::transmute::<Float, FSize>(self.scale)).hash(state);
        }
        self.wrapping.hash(state);
        self.gamma.hash(state);
    }
}

impl Eq for ImageInfo { }

/// Wrapping mode when coordinates out of bound
#[derive(Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum ImageWrapMode {
    /// repeat the texture again
    Repeat,
    /// return black texel
    Black,
    /// clamp to the boundary texel 
    Clamp,
}

// TODO:
#[allow(dead_code)]
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