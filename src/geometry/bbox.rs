// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! 2D and 3D bounding box

use super::foundamental::*;
use std::ops;
use std::mem;
use super::ray::Ray;
use num_traits::NumCast;

pub type BBox2f = BBox2<Float>;
pub type BBox3f = BBox3<Float>;


/// A 2D bounding box
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct BBox2<T> {
    /// min corner of the bounding box
    pub pmin: Point2<T>,
    /// max corner of the bounding box
    pub pmax: Point2<T>,
}

impl<T: BaseNum> BBox2<T> {
    /// Construct a new bounding box marked by two corner vertice
    #[inline]
    pub fn new(p: Point2<T>, q: Point2<T>) -> BBox2<T> {
        BBox2{
            pmin: Point2::new(
                <T as  PartialOrd>::partial_min(p.x, q.x),
                <T as  PartialOrd>::partial_min(p.y, q.y),
            ),
            pmax: Point2::new(
                <T as  PartialOrd>::partial_max(p.x, q.x),
                <T as  PartialOrd>::partial_max(p.y, q.y),
            )
        }
    }

    /// Return the `i`th corner vertex
    pub fn corner(&self, i: usize) -> Point2<T> {
        assert!(i < 4, "index out of bound");
        let x = if (i & 1) == 0 {
            self.pmin.x
        } else {
            self.pmax.x
        };

        let y = if (i & 2) == 0 {
            self.pmin.y
        } else {
            self.pmax.y
        };

        Point2::new(x, y)
    }

    /// Extend the bounding box with `p`, return the resultant new bbox
    #[inline]
    pub fn extend(&self, p: Point2<T>) -> Self {
        BBox2{
            pmin: Point2::new(
                <T as  PartialOrd>::partial_min(self.pmin.x, p.x),
                <T as  PartialOrd>::partial_min(self.pmin.y, p.y),
            ),
            pmax: Point2::new(
                <T as  PartialOrd>::partial_max(self.pmax.x, p.x),
                <T as  PartialOrd>::partial_max(self.pmax.y, p.y),
            )        
        }
    }

    /// Return the union of two bounding boxes
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        BBox2{
            pmin: Point2::new(
                <T as  PartialOrd>::partial_min(self.pmin.x, other.pmin.x),
                <T as  PartialOrd>::partial_min(self.pmin.y, other.pmin.y),
            ),
            pmax: Point2::new(
                <T as  PartialOrd>::partial_max(self.pmax.x, other.pmax.x),
                <T as  PartialOrd>::partial_max(self.pmax.y, other.pmax.y),
            )        
        }
    }

    /// Return the intersection of two bounding boxes
    #[inline]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let ret = BBox2 {
            pmin: Point2::new(
                <T as  PartialOrd>::partial_max(self.pmin.x, other.pmin.x),
                <T as  PartialOrd>::partial_max(self.pmin.y, other.pmin.y),
            ),
            pmax: Point2::new(
                <T as  PartialOrd>::partial_min(self.pmax.x, other.pmax.x),
                <T as  PartialOrd>::partial_min(self.pmax.y, other.pmax.y),
            )        
        };
        if ret.pmin.x > ret.pmax.x || ret.pmin.y > ret.pmax.y {
            None
        } else {
            Some(ret)
        }
    }

    /// Return if two bounding boxes overlap
    #[inline]
    pub fn overlap(&self, other: &Self) -> bool {
        (self.pmax.x >= other.pmin.x && self.pmin.x <= other.pmax.x)
        && (self.pmax.y >= other.pmin.y && self.pmin.y <= other.pmax.y)
    }

    /// Return if `self` contains `p`
    #[inline]
    pub fn contain(&self, p: Point2<T>) -> bool {
        (p.x <= self.pmax.x && p.x >= self.pmin.x)
        && (p.y <= self.pmax.y && p.y >= self.pmin.y)
    }

    /// Return if `self` contains `p`, excluding the case when `p` lies on
    /// the upper-left boundary
    #[inline]
    pub fn contain_lb(&self, p: Point2<T>) -> bool {
        (p.x < self.pmax.x && p.x >= self.pmin.x)
        && (p.y < self.pmax.y && p.y >= self.pmin.y)
    }

    /// Expand each boundary by `delta`. Note that `delta` can be negative
    #[inline]
    pub fn expand_by(&self, delta: T) -> Self
        where T: ops::Neg<Output = T> {
        BBox2 {
            pmin: self.pmin + (-Vector2::new(delta, delta)),
            pmax: self.pmax + Vector2::new(delta, delta),
        }
    }

    /// Return the diagonal vector, from `pmin` to `pmax`
    #[inline]
    pub fn diagonal(&self) -> Vector2<T> {
        self.pmax - self.pmin
    }

    /// Return the surface area of the bounding box
    #[inline]
    pub fn surface_area(&self) -> T {
        let dx = self.pmax.x - self.pmin.x;
        let dy = self.pmax.y - self.pmin.y;
        let zero = <T as Zero>::zero();
        if dx >= zero && dy >= zero {
            dx * dy
        } else {
            zero
        }
    }

    /// Returns the index of the axis along which the bounding
    /// box has maximum extent
    pub fn max_extent(&self) -> usize {
        let d = self.diagonal();
        if d.x > d.y {
            0usize
        } else {
            1usize
        }
    }

    /// Linearly interpolate between the two corners
    pub fn lerp(&self, t: Vector2<Float>) -> Point2<T>
        where T: NumCast,
    {
        Point2::new(
            <T as NumCast>::from(
                t.x * <Float as NumCast>::from(self.pmax.x).unwrap()
                + (1.0 as Float - t.x) * <Float as NumCast>::from(self.pmin.x).unwrap()
            ).unwrap(),
            <T as NumCast>::from(
                t.y * <Float as NumCast>::from(self.pmax.y).unwrap()
                + (1.0 as Float - t.y) * <Float as NumCast>::from(self.pmin.y).unwrap()
            ).unwrap()
        )
    }

    /// Returns the bounding circle as `(center, radius)`
    pub fn bcircle(&self) -> (Point2<T>, T)
        where T: BaseFloat
    {
        let two = <T as One>::one() + <T as One>::one();
        let zero = <T as Zero>::zero();
        let pmin: (T, T) = self.pmin.into();
        let pmin: Vector2<T> = pmin.into();
        let center = (self.pmax + pmin) / two;
        let radius = if self.contain(center) {
            center.distance(self.pmax)
        } else {
            zero
        };

        debug_assert!(radius >= zero);
        (center, radius)
    }

    /// Casting to another type of bbox
    pub fn cast<R: BaseNum>(&self) -> BBox2<R> {
        BBox2{
            pmin: self.pmin.cast(),
            pmax: self.pmax.cast(),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[must_use]
pub struct BBox3<T> {
    /// min corner of the bounding box
    pub pmin: Point3<T>,
    /// max corner of the bounding box
    pub pmax: Point3<T>,
}

/// A 3D bounding box
impl<T: BaseNum> BBox3<T> {
    /// Construct a new bounding box marked by two corner vertice
    #[inline]
    pub fn new(p: Point3<T>, q: Point3<T>) -> BBox3<T> {
        BBox3{
            pmin: Point3::new(
                <T as  PartialOrd>::partial_min(p.x, q.x),
                <T as  PartialOrd>::partial_min(p.y, q.y),
                <T as  PartialOrd>::partial_min(p.z, q.z),
            ),
            pmax: Point3::new(
                <T as  PartialOrd>::partial_max(p.x, q.x),
                <T as  PartialOrd>::partial_max(p.y, q.y),
                <T as  PartialOrd>::partial_max(p.z, q.z),
            )
        }
    }

    /// Return the `i`th corner vertex
    pub fn corner(&self, i: usize) -> Point3<T> {
        assert!(i < 4, "index out of bound");
        let x = if (i & 1) == 0 {
            self.pmin.x
        } else {
            self.pmax.x
        };

        let y = if (i & 2) == 0 {
            self.pmin.y
        } else {
            self.pmax.y
        };

        let z = if (i & 3) == 0 {
            self.pmin.z
        } else {
            self.pmax.z
        };

        Point3::new(x, y, z)
    }

    /// `index==false` for `self.pmin`, else `self.pmax`.
    #[inline]
    fn index(&self, index: bool) -> Point3<T> {
        if index {
            self.pmax
        } else {
            self.pmin
        }
    }

    /// Extend the bounding box with `p`, return the resultant new bbox
    #[inline]
    pub fn extend(&self, p: Point3<T>) -> Self {
        BBox3{
            pmin: Point3::new(
                <T as  PartialOrd>::partial_min(self.pmin.x, p.x),
                <T as  PartialOrd>::partial_min(self.pmin.y, p.y),
                <T as  PartialOrd>::partial_min(self.pmin.z, p.z),
            ),
            pmax: Point3::new(
                <T as  PartialOrd>::partial_max(self.pmax.x, p.x),
                <T as  PartialOrd>::partial_max(self.pmax.y, p.y),
                <T as  PartialOrd>::partial_max(self.pmax.z, p.z),
            )        
        }
    }

    /// Return the union of two bounding boxes
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        BBox3{
            pmin: Point3::new(
                <T as  PartialOrd>::partial_min(self.pmin.x, other.pmin.x),
                <T as  PartialOrd>::partial_min(self.pmin.y, other.pmin.y),
                <T as  PartialOrd>::partial_min(self.pmin.z, other.pmin.z),
            ),
            pmax: Point3::new(
                <T as  PartialOrd>::partial_max(self.pmax.x, other.pmax.x),
                <T as  PartialOrd>::partial_max(self.pmax.y, other.pmax.y),
                <T as  PartialOrd>::partial_max(self.pmax.z, other.pmax.z),
            )        
        }
    }

    /// Return the intersection of two bounding boxes
    #[inline]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let ret = BBox3{
            pmin: Point3::new(
                <T as  PartialOrd>::partial_max(self.pmin.x, other.pmin.x),
                <T as  PartialOrd>::partial_max(self.pmin.y, other.pmin.y),
                <T as  PartialOrd>::partial_max(self.pmin.z, other.pmin.z),
            ),
            pmax: Point3::new(
                <T as  PartialOrd>::partial_min(self.pmax.x, other.pmax.x),
                <T as  PartialOrd>::partial_min(self.pmax.y, other.pmax.y),
                <T as  PartialOrd>::partial_min(self.pmax.z, other.pmax.z),
            )        
        };
        if ret.pmin.x > ret.pmax.x || ret.pmin.y > ret.pmax.y || ret.pmin.z > ret.pmax.z {
            None
        } else {
            Some(ret)
        }
    }

    /// Return if two bounding boxes overlap
    #[inline]
    pub fn overlap(&self, other: &Self) -> bool {
        (self.pmax.x >= other.pmin.x && self.pmin.x <= other.pmax.x)
        && (self.pmax.y >= other.pmin.y && self.pmin.y <= other.pmax.y)
        && (self.pmax.z >= other.pmin.z && self.pmin.z <= other.pmax.z)
    }

    /// Return if `self` contains `p`
    #[inline]
    pub fn contain(&self, p: Point3<T>) -> bool {
        (p.x <= self.pmax.x && p.x >= self.pmin.x)
        && (p.y <= self.pmax.y && p.y >= self.pmin.y)
        && (p.z <= self.pmax.z && p.z >= self.pmin.z)
    }

    /// Return if `self` contains `p`, excluding the case when `p` lies on
    /// the greater boundaries
    #[inline]
    pub fn contain_lb(&self, p: Point3<T>) -> bool {
        (p.x < self.pmax.x && p.x >= self.pmin.x)
        && (p.y < self.pmax.y && p.y >= self.pmin.y)
        && (p.z < self.pmax.z && p.z >= self.pmin.z)
    }

    /// Expand each boundary by `delta`. Note that `delta` can be negative
    #[inline]
    pub fn expand_by(&self, delta: T) -> Self
        where T: ops::Neg<Output = T> {
        BBox3 {
            pmin: self.pmin + (-Vector3::new(delta, delta, delta)),
            pmax: self.pmax + Vector3::new(delta, delta, delta),
        }
    }

    /// Return the diagonal vector, from `pmin` to `pmax`
    #[inline]
    pub fn diagonal(&self) -> Vector3<T> {
        self.pmax - self.pmin
    }

    /// Return the surface area of the bounding box
    #[inline]
    pub fn surface_area(&self) -> T {
        let mut dx = self.pmax.x - self.pmin.x;
        let mut dy = self.pmax.y - self.pmin.y;
        let mut dz = self.pmax.z - self.pmin.z;
        let zero = <T as Zero>::zero();
        
        if dx < zero { dx = zero; }
        if dy < zero { dy = zero; }
        if dz < zero { dz = zero; }
        let two = <T as One>::one() + <T as One>::one();
        two * (dx * dy + dx * dz + dy * dz)
    }

    /// Renturn the volume of the bounding box
    #[inline]
    pub fn volume(&self) -> T {
        let dx = self.pmax.x - self.pmin.x;
        let dy = self.pmax.y - self.pmin.y;
        let dz = self.pmax.z - self.pmin.z;
        let zero = <T as Zero>::zero();
        
        if dx <= zero || dy <= zero || dz <= zero {
            zero
        } else {
            dx * dy * dz
        }
    }

    /// Returns the index of the axis along which the bounding
    /// box has maximum extent
    pub fn max_extent(&self) -> usize {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            0usize
        } else if d.y > d.z {
            1usize
        } else {
            2usize
        }
    }

    /// Linearly interpolate between the two corners
    pub fn lerp(&self, t: Vector3f) -> Point3<T>
         where T: NumCast,
    {
        Point3::new(
            <T as NumCast>::from(
                t.x * <Float as NumCast>::from(self.pmax.x).unwrap()
                + (1.0 as Float - t.x) * <Float as NumCast>::from(self.pmin.x).unwrap()
            ).unwrap(),
            <T as NumCast>::from(
                t.y * <Float as NumCast>::from(self.pmax.y).unwrap()
                + (1.0 as Float - t.y) * <Float as NumCast>::from(self.pmin.y).unwrap()
            ).unwrap(),
            <T as NumCast>::from(
                t.z * <Float as NumCast>::from(self.pmax.z).unwrap()
                + (1.0 as Float - t.z) * <Float as NumCast>::from(self.pmin.z).unwrap()
            ).unwrap(),
        )
    }

    pub fn bsphere(&self) -> (Point3<T>, T)
        where T: BaseFloat
    {
        let two = <T as One>::one() + <T as One>::one();
        let zero = <T as Zero>::zero();
        let pmin = self.pmin.to_vec();
        let center = (self.pmax + pmin) / two;
        let radius = if self.contain(center) {
            center.distance(self.pmax)
        } else {
            zero
        };

        debug_assert!(radius >= zero);
        (center, radius)
    }
    
    /// Apply transform `t` on `self`, returning a new bounding box
    pub fn apply_transform<Tr>(&self, t: &Tr) -> Self
        where Tr: Transform3<T>
    {
        // let bbox = BBox3::new(
        //     t.transform_point(Point3::new(self.pmin.x, self.pmin.y, self.pmin.z)),
        //     t.transform_point(Point3::new(self.pmax.x, self.pmin.y, self.pmin.z))
        // );
        // bbox.extend(Point3::new(self.pmin.x, self.pmax.y, self.pmin.z))
        //     .extend(Point3::new(self.pmin.x, self.pmin.y, self.pmax.z))
        //     .extend(Point3::new(self.pmin.x, self.pmax.y, self.pmax.z))
        //     .extend(Point3::new(self.pmax.x, self.pmin.y, self.pmax.z))
        //     .extend(Point3::new(self.pmax.x, self.pmax.y, self.pmin.z))
        //     .extend(Point3::new(self.pmax.x, self.pmax.y, self.pmax.z))
        let p = t.transform_point(self.pmin);
        let diagonal = t.transform_vector(self.diagonal());
        BBox3::new(
            p, p + diagonal
        )
    }

    /// Casting to another type of bbox
    pub fn cast<R: BaseNum>(&self) -> BBox3<R> {
        BBox3{
            pmin: self.pmin.cast(),
            pmax: self.pmax.cast(),
        }
    }
}

impl BBox3f {
    /// Test if the `ray` intersects `self`
    pub fn intersect_ray<R>(&self, ray: &R) -> Option<(Float, Float)>
        where R: Ray + ?Sized
    {
        let mut t0 = 0.0 as Float;
        let mut t1 = ray.max_extend();

        let origin = ray.origin();
        let direction = ray.direction();

        for i in 0..3 {
            let inv_direction = (1.0 as Float) / direction[i];
            let mut t_near = (self.pmin[i] - origin[i]) * inv_direction;
            let mut t_far = (self.pmax[i] - origin[i]) * inv_direction;
            if t_near > t_far {
                mem::swap(&mut t_near, &mut t_far);
            }
            
            // TODO: Update to ensure robust ray-bounds intersection

            if t_near > t0 {
                t0 = t_near;
            }
            if t_far < t1 {
                t1 = t_far;
            }

            if t0 > t1 {
                return None;
            }
        }

        Some((t0, t1))
    }

    /// Test if the `ray` intersects `self`, with cache
    #[inline]
    pub fn intersect_ray_cached(&self, cache: &(Point3f, Vector3f, Vector3<bool>, Float)) -> Option<(Float, Float)>
    {
        let mut t0 = (self.index(cache.2.x).x - cache.0.x) * cache.1.x;
        let mut t1 = (self.index(!cache.2.x).x - cache.0.x) * cache.1.x;

        let ty0 = (self.index(cache.2.y).y - cache.0.y) * cache.1.y;
        let ty1 = (self.index(!cache.2.y).y - cache.0.y) * cache.1.y;

        // TODO: update for robustness

        if t0 > ty1 || ty0 > t1 { return None; }
        if ty0 > t0 { t0 = ty0; }
        if ty1 < t1 { t1 = ty1; }

        let tz0 = (self.index(cache.2.z).z - cache.0.z) * cache.1.z;
        let tz1 = (self.index(!cache.2.z).z - cache.0.z) * cache.1.z;

        // TODO: update for robustness

        if t0 > tz1 || tz0 > t1 { return None; }
        if tz0 > t0 { t0 = tz0; }
        if tz1 < t1 { t1 = tz1; }

        if t0 < cache.3 && t1 > (0.0 as Float) {
            Some((t0, t1))
        } else {
            None
        }
    }

    /// Construct cache for chached intersection
    pub fn construct_ray_cache<R>(ray: &R) -> (Point3f, Vector3f, Vector3<bool>, Float)
        where R: Ray + ?Sized
    {
        let origin = ray.origin();
        let invert_direction = 1.0 as Float / ray.direction();
        let zero = 0.0 as Float;
        let dir_is_neg = Vector3::new(invert_direction.x < zero, invert_direction.y < zero, invert_direction.z < zero);
        let max_extend = ray.max_extend();
        (origin, invert_direction, dir_is_neg, max_extend)
    }
}

pub struct BBox2iIter {
    ix: isize,
    iy: isize,
    nx: isize,
    ny: isize,
    nx_start: isize,
}

impl Iterator for BBox2iIter {
    type Item = Point2<isize>;

    #[inline]
    fn next(&mut self) -> Option<Point2<isize>> {
        while self.iy < self.ny {
            if self.ix < self.nx {
                let ix = self.ix;
                self.ix += 1;
                return Some(Point2::new(ix, self.iy))
            } else {
                self.iy += 1;
                self.ix = self.nx_start;
            }
        }
        None
    }
}

impl IntoIterator for BBox2<isize> {
    type Item = Point2<isize>;
    type IntoIter = BBox2iIter;
    fn into_iter(self) -> BBox2iIter {
        BBox2iIter{
            ix: self.pmin.x,
            iy: self.pmin.y,
            nx: self.pmax.x,
            ny: self.pmax.y,
            nx_start: self.pmin.x,
        }
    }
}
