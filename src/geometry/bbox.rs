use super::cgmath_prelude::*;
use std::ops;

/// A 2D bounding box
#[derive(PartialEq, Copy, Clone)]
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
    pub fn intersect(&self, other: &Self) -> Self {
        BBox2{
            pmin: Point2::new(
                <T as  PartialOrd>::partial_max(self.pmin.x, other.pmin.x),
                <T as  PartialOrd>::partial_max(self.pmin.y, other.pmin.y),
            ),
            pmax: Point2::new(
                <T as  PartialOrd>::partial_min(self.pmax.x, other.pmax.x),
                <T as  PartialOrd>::partial_min(self.pmax.y, other.pmax.y),
            )        
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
    pub fn lerp(&self, t: Vector2<f64>) -> Point2<T>
        where f64: From<T>,
              T: From<f64>,
    {
        Point2::new(
            (
                t.x * <T as Into<f64>>::into(self.pmin.x)
                + (1.0 - t.x) * <T as Into<f64>>::into(self.pmax.x)
            ).into(),
            (
                t.y * <T as Into<f64>>::into(self.pmin.y)
                + (1.0 - t.y) * <T as Into<f64>>::into(self.pmax.y)
            ).into(),
        )
    }

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
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct BBox3<T> {
    pub pmin: Point3<T>,
    pub pmax: Point3<T>,
}


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
    pub fn intersect(&self, other: &Self) -> Self {
        BBox3{
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
    pub fn lerp(&self, t: Vector3<f64>) -> Point3<T>
        where f64: From<T>,
              T: From<f64>,
    {
        Point3::new(
            (
                t.x * <T as Into<f64>>::into(self.pmin.x)
                + (1.0 - t.x) * <T as Into<f64>>::into(self.pmax.x)
            ).into(),
            (
                t.y * <T as Into<f64>>::into(self.pmin.y)
                + (1.0 - t.y) * <T as Into<f64>>::into(self.pmax.y)
            ).into(),
            (
                t.z * <T as Into<f64>>::into(self.pmin.z)
                + (1.0 - t.z) * <T as Into<f64>>::into(self.pmax.z)
            ).into(),
        )
    }

    pub fn bsphere(&self) -> (Point3<T>, T)
        where T: BaseFloat
    {
        let two = <T as One>::one() + <T as One>::one();
        let zero = <T as Zero>::zero();
        let pmin: (T, T, T) = self.pmin.into();
        let pmin: Vector3<T> = pmin.into();
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
        where Tr: Transform<Point3<T>>
    {
        let bbox = BBox3::new(
            t.transform_point(Point3::new(self.pmin.x, self.pmin.y, self.pmin.z)),
            t.transform_point(Point3::new(self.pmax.x, self.pmin.y, self.pmin.z))
        );
        bbox.extend(Point3::new(self.pmin.x, self.pmax.y, self.pmin.z))
            .extend(Point3::new(self.pmin.x, self.pmin.y, self.pmax.z))
            .extend(Point3::new(self.pmin.x, self.pmax.y, self.pmax.z))
            .extend(Point3::new(self.pmax.x, self.pmin.y, self.pmax.z))
            .extend(Point3::new(self.pmax.x, self.pmax.y, self.pmin.z))
            .extend(Point3::new(self.pmax.x, self.pmax.y, self.pmax.z))
    }
}