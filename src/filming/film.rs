// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines the `Film`, along with tiles and images it uses

use geometry::prelude::*;
use spectrum::{Spectrum, RGBSpectrumf, ToNorm};
use sample::Filter;
use std::ops;
use std::mem;
use std::sync::Arc;
use image;
use std::path::Path;
use std::io::Result;
// use std::marker::PhantomData;

#[inline]
fn pidx_to_pcenter(idx: Point2<usize>) -> Point2f {
    let mut ret: Point2f = idx.cast();
    ret.x += 0.5 as Float;
    ret.y += 0.5 as Float;
    ret
}

#[inline]
fn pcenter_to_pidx(mut center: Point2f) -> Point2<usize> {
    center.x -= 0.5 as Float;
    center.y -= 0.5 as Float;
    center.cast()
}



/// The mighty film
///
/// # Intended Usage:
/// 1. Create with `new`.
/// 2. Spawn an array of tiles
/// 3. tracing was done within those tiles, possibly multithreaded
/// 4. when done, collect the result into a single `image`
pub struct Film {
    resolution: Point2<usize>,
    crop_window: BBox2<usize>,
    filter: Arc<Filter>,
    filter_radius: Vector2f,
    inv_filter_radius: Vector2f,
}

impl Film {
    /// construction. `crop_window` specified in NDC
    pub fn new(resolution: Point2<usize>, crop_window: BBox2f, filter: Arc<Filter>) -> Film {
        let resf: Point2f = resolution.cast();
        let crop_window = BBox2::new(
            Point2::new(
                (resf.x * crop_window.pmin.x).ceil() as usize,
                (resf.y * crop_window.pmin.y).ceil() as usize
            ),
            Point2::new(
                (resf.x * crop_window.pmax.x).ceil() as usize,
                (resf.y * crop_window.pmax.y).ceil() as usize
            )
        );
        let filter_radius = filter.radius();
        let inv_filter_radius = Vector2f::new(
            1.0 as Float / filter_radius.x,
            1.0 as Float / filter_radius.y,
        );
        Film{
            resolution: resolution,
            crop_window: crop_window,
            filter: filter,
            filter_radius: filter_radius,
            inv_filter_radius: inv_filter_radius,
        }
    }

    /// merge output from a tile into a sink
    pub fn merge_into<S>(
        &self, tile: FilmTile<S>,
        sink: &mut BoundedSink2D<TilePixel<RGBSpectrumf>>)
        where S: Spectrum<Scalar=Float>,
    {
        assert!(self.crop_window == sink.bounding);
        assert!(sink.bounding.contain_lb(tile.sink.bounding.pmin));
        assert!(sink.bounding.contain(tile.sink.bounding.pmax));
        for pixel_idx in tile.sink.bounding {
            let (rgbspec, weight) = unsafe {
                let s = tile.sink.get_pixel_unchecked(pixel_idx);
                (s.spectrum_sum.to_srgb(), s.filter_weight_sum)
            };
            let s = unsafe {
                sink.get_pixel_mut_unchecked(pixel_idx)
            };
            s.spectrum_sum += rgbspec;
            s.filter_weight_sum += weight;
        }
    }

    /// spawn tiles
    pub fn spawn_tiles<S>(&self, nx: usize, ny: usize) -> Vec<FilmTile<S>>
        where TilePixel<S>: Clone
    {
        assert!(nx > 0);
        assert!(ny > 0);
        let extend = self.crop_window.diagonal();
        let dx = extend.x / nx;
        let dy = extend.y / ny;
        let lastx = dx + extend.x % dx;
        let lasty = dy + extend.y % dy;
        let mut ret = Vec::with_capacity(nx * ny);
        for ix in 0..nx {
            let cdx = if ix==nx-1 { lastx } else { dx };
            for iy in 0..ny {
                let cdy = if iy==ny-1 { lasty } else { dy };
                let bbox = BBox2::new(
                    Point2::new(ix*dx, iy*dy),
                    Point2::new(ix*dx + cdx, iy*dy + cdy),
                );
                ret.push(FilmTile{
                    filter: &*self.filter,
                    filter_radius: self.filter_radius,
                    inv_filter_radius: self.inv_filter_radius,
                    sink: BoundedSink2D::new(bbox),
                })
            }
        }
        ret
    }

    /// collect results into an image
    pub fn collect_into<'a, S, I>(&self, tiles: I) -> Image
        where S: Spectrum<Scalar=Float>,
              TilePixel<S>: Clone,
              I: IntoIterator<Item=FilmTile<'a, S>>,
    {
        let mut tmp = BoundedSink2D::new(self.crop_window);
        for tile in tiles {
            self.merge_into(tile, &mut tmp);
        }
        Image::from_sink(tmp)
    }

    /// get resolution
    #[inline]
    pub fn resolutionf(&self) -> Vector2f {
        self.resolution.to_vec().cast()
    }
}

/// Memory sink for bounded 2d values
pub struct BoundedSink2D<S> {
    pixels: Vec<S>,
    bounding: BBox2<usize>,
}

impl<S: Clone> BoundedSink2D<S> {
    /// construction
    pub fn new(bbox: BBox2<usize>) -> BoundedSink2D<S> {
        assert!(bbox.pmax.x > bbox.pmin.x);
        assert!(bbox.pmax.y > bbox.pmin.y);
        let diagonal = bbox.diagonal();
        let pixels = unsafe{
            vec![mem::uninitialized(); diagonal.x * diagonal.y]
        };
        BoundedSink2D{
            pixels: pixels,
            bounding: bbox,
        }
    }

    /// construction with default value
    pub fn with_value(value: S, bbox: BBox2<usize>) -> BoundedSink2D<S> {
        assert!(bbox.pmax.x > bbox.pmin.x);
        assert!(bbox.pmax.y > bbox.pmin.y);
        let diagonal = bbox.diagonal();
        let pixels = vec![value; diagonal.x * diagonal.y];
        BoundedSink2D{
            pixels: pixels,
            bounding: bbox,
        }
    }
}

impl<S> BoundedSink2D<S> {
    /// returns the offset in `self.pixels` at p
    #[inline]
    fn get_pixel_offset(&self, p: Point2<usize>) -> usize {
        debug_assert!(self.bounding.contain_lb(p));
        (p.x - self.bounding.pmin.x)
        + (p.y - self.bounding.pmin.y) * (self.bounding.pmax.x - self.bounding.pmin.x)
    }

    /// get pixel at
    #[inline]
    pub fn get_pixel(&self, p: Point2<usize>) -> &S {
        assert!(self.bounding.contain_lb(p));
        unsafe {
            self.get_pixel_unchecked(p)
        }
    }

    #[inline]
    unsafe fn get_pixel_unchecked(&self, p: Point2<usize>) -> &S {
        let idx = self.get_pixel_offset(p);
        self.pixels.get_unchecked(idx)
    }

    /// get mut pixel at
    #[inline]
    pub fn get_pixel_mut(&mut self, p: Point2<usize>) -> &mut S {
        assert!(self.bounding.contain_lb(p));
        unsafe {
            self.get_pixel_mut_unchecked(p)
        }
    }
    

    #[inline]
    unsafe fn get_pixel_mut_unchecked(&mut self, p: Point2<usize>) -> &mut S {
        let idx = self.get_pixel_offset(p);
        self.pixels.get_unchecked_mut(idx)
    }

    /// get bounding
    #[inline]
    pub fn bounding(&self) -> BBox2<usize> {
        self.bounding
    }
}

/// A tile from the film, generated by `film.spawn_tiles()`.
/// Basic building block for multithreaded ray-tracing.
pub struct FilmTile<'a, S> {
    filter: &'a Filter,
    filter_radius: Vector2f,
    inv_filter_radius: Vector2f,
    sink: BoundedSink2D<TilePixel<S>>,
}

use std::marker::Send;

unsafe impl<'a, S> Send for FilmTile<'a, S> { }

impl<'a, S> FilmTile<'a, S>
    where S: Spectrum + ops::AddAssign,
          for<'b> &'b S: ops::Mul<Float, Output=S>,
{
    /// add a sample's contribution to every related pixels
    pub fn add_sample(&mut self, pos: Point2f, spectrum: &S) {
        let posidxf: Point2f = pcenter_to_pidx(pos).cast();
        let ceil = posidxf.to_vec() - self.filter_radius;
        let floor = posidxf.to_vec() + self.filter_radius;
        let ceilidx: Vector2<usize> = ceil.cast();
        let flooridx: Vector2<usize> = floor.cast() + Vector2::new(1, 1);
        let relavant_box = BBox2::new(Point2::from_vec(ceilidx), Point2::from_vec(flooridx)).intersect(&self.sink.bounding);
        for pixel_idx in relavant_box {
            let pixel_pos = pidx_to_pcenter(pixel_idx);
            let offset = Point2::from_vec(pixel_pos - pos);
            let weight = unsafe {
                self.filter.evaluate_unsafe(offset)
            };
            let pixel = unsafe {
                self.sink.get_pixel_mut_unchecked(pixel_idx)
            };
            pixel.spectrum_sum += spectrum * weight;
            pixel.filter_weight_sum += weight;
        }
    }

    /// get the bouding box of this tile
    pub fn bounding(&self) -> BBox2<usize> {
        self.sink.bounding
    }
}

/// A pixel in film tile
#[derive(Copy, Clone, Debug)]
pub struct TilePixel<S> {
    pub spectrum_sum: S,
    pub filter_weight_sum: Float,
}

impl<S> TilePixel<S>
    where S: Spectrum + ops::Div<Float, Output=S>,
{
    /// get final result
    pub fn finalize(self) -> S {
        self.spectrum_sum / self.filter_weight_sum
    }
}

/// A mighty image
pub struct Image {
    inner: BoundedSink2D<RGBSpectrumf>,
}

impl Image {
    /// convert into an boundedsink
    fn into_inner(self) -> BoundedSink2D<RGBSpectrumf> {
        self.inner
    }

    /// construct an image with default spectrum
    pub fn new(spectrum: RGBSpectrumf, dim: Point2<usize>) -> Image {
        Image{
            inner: BoundedSink2D::with_value(spectrum, BBox2::new(Point2::new(0, 0), dim))
        }
    }

    fn from_sink(sink: BoundedSink2D<TilePixel<RGBSpectrumf>>) -> Image {
        let mut inner = BoundedSink2D::new(BBox2::new(Point2::new(0, 0), sink.bounding.pmax));
        for p_idx in sink.bounding {unsafe {
            *inner.get_pixel_mut_unchecked(p_idx) = sink.get_pixel(p_idx).finalize();
        }}
        Image { inner: inner }
    }

    /// save this image to `path`
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut support = Vec::with_capacity(self.inner.pixels.len() * 3);
        for p in self.inner.bounding {
            let s = unsafe {
                self.inner.get_pixel_unchecked(p)
            };
            support.push(ToNorm::from_norm(s.r()));
            support.push(ToNorm::from_norm(s.g()));
            support.push(ToNorm::from_norm(s.b()));
        }
        image::save_buffer(path, support.as_slice(), self.inner.bounding.pmax.x as u32, self.inner.bounding.pmax.y as u32, image::ColorType::RGB(8))
    }
}

impl ops::Index<(usize, usize)> for Image {
    type Output = RGBSpectrumf;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &RGBSpectrumf {
        self.inner.get_pixel(Point2::new(index.0, index.1))
    }
}

impl ops::IndexMut<(usize, usize)> for Image {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut RGBSpectrumf {
        self.inner.get_pixel_mut(Point2::new(index.0, index.1))
    }
}

impl ops::Index<Point2<usize>> for Image {
    type Output = RGBSpectrumf;

    #[inline]
    fn index(&self, index: Point2<usize>) -> &RGBSpectrumf {
        self.inner.get_pixel(index)
    }
}

impl ops::IndexMut<Point2<usize>> for Image {
    #[inline]
    fn index_mut(&mut self, index: Point2<usize>) -> &mut RGBSpectrumf {
        self.inner.get_pixel_mut(index)
    }
}
