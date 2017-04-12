use super::cgmath_prelude::*;
use super::float;
use super::bbox::BBox3f;
use std::mem;

/// A semi-infinite line
pub trait Ray {
    /// Returns where the ray originates
    fn origin(&self) -> Point3f;

    /// Sets the origin to `o`.
    /// Implementations must ensure that this is valid
    fn set_origin(&mut self, o: Point3f);

    /// Returns the max extend of the ray, in `self.direction().length()`
    fn max_extend(&self) -> Float;
    
    /// Set the max extend of the ray
    fn set_max_extend(&mut self, tmax: Float);

    /// Returns where the ray heads to.
    /// The length of the returned vector is the unit of the ray
    fn direction(&self) -> Vector3f;

    /// Sets the direction to `d`.
    /// Implementations must ensure that this is valid
    fn set_direction(&mut self, d: Vector3f);

    /// Evaluate the point `t`-units away from `self.origin()`
    fn evaluate(&self, t: Float) -> Point3f {
        self.origin() + self.direction() * t
    }

    /// Apply transform `t` on `self`, returning the new `Ray`.
    fn apply_transform<T>(&self, t: &T) -> Self
        where T: Transform3<Float>;

    /// intersect against a bbox
    fn intersect_bbox(&self, bbox: &BBox3f) -> Option<(Float, Float)>
    {
        bbox.intersect_ray(self)
    }

    /// return a closure for shearing transform
    fn shearing_transform(&self) -> ShearingTransformCache
    {
        ShearingTransformCache::from_ray(self)
    }
}

/// A semi-infinite line specified by its `origin` and `dir`ection.
#[derive(PartialEq, Copy, Clone)]
pub struct RawRay {
    origin: Point3f,
    dir: Vector3f,
    tmax: Float,
    stc: ShearingTransformCache,
}

impl RawRay {
    #[inline]
    pub fn new(origin: Point3f, dir: Vector3f, tmax: Float) -> RawRay {
        let mut ray = RawRay {
            origin: origin,
            dir: dir,
            tmax: tmax,
            stc: unsafe {mem::uninitialized()},
        };
        let stc = ShearingTransformCache::from_ray(&ray);
        ray.stc = stc;
        ray
    }

    fn reset_shearing_transform(&mut self) {
        let stc = ShearingTransformCache::from_ray(self);
        self.stc = stc;
    }
}

impl Default for RawRay {
    #[inline]
    fn default() -> Self {
        RawRay::new(
            Point3::new(0.0 as Float, 0.0 as Float, 0.0 as Float),
            Vector3::new(0.0 as Float, 0.0 as Float, 1.0 as Float),
            float::infinity(),
        )
    }
}

impl Ray for RawRay {
    #[inline]
    fn origin(&self) -> Point3f {
        self.origin
    }

    #[inline]
    fn set_origin(&mut self, o: Point3f) {
        self.origin = o;
        self.reset_shearing_transform();
    }

    #[inline]
    fn max_extend(&self) -> Float {
        self.tmax
    }

    #[inline]
    fn set_max_extend(&mut self, tmax: Float) {
        self.tmax = tmax;
        self.reset_shearing_transform();
    }

    #[inline]
    fn direction(&self) -> Vector3f {
        self.dir
    }

    #[inline]
    fn set_direction(&mut self, d: Vector3f) {
        self.dir = d;
        self.reset_shearing_transform();
    }

    // FIXME: Deal with rounding error
    #[inline]
    fn apply_transform<T>(&self, t: &T) -> RawRay
        where T: Transform<Point3f>
    {
        RawRay::new(
            t.transform_point(self.origin),
            t.transform_vector(self.dir),
            self.tmax,
        )
    }

    #[inline]
    fn shearing_transform(&self) -> ShearingTransformCache {
        self.stc
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct ShearingTransformCache {
    pub perm: Permulation,
    pub neg_o: Vector3f,
    pub shear: Vector3f,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Permulation {
    XZ,
    YZ,
    ZZ,
}

impl ShearingTransformCache {
    pub fn from_ray<R: ?Sized + Ray>(ray: &R) -> ShearingTransformCache {
        let o = ray.origin();
        let neg_o = -Vector3f::new(o.x, o.y, o.z);

        let direction = ray.direction();
        let absdx = direction.x.abs();
        let absdy = direction.y.abs();
        let absdz = direction.z.abs();

        let perm = if absdx > absdy && absdx > absdz {
            Permulation::XZ
        } else if absdy > absdz {
            Permulation::YZ
        } else {
            Permulation::ZZ
        };

        let shear = Vector3f::new(
            -direction.x / direction.z,
            -direction.y / direction.z,
            (1.0 as Float) / direction.z);
        
        ShearingTransformCache{
            perm: perm,
            neg_o: neg_o,
            shear: shear,
        }
    }

    pub fn apply(&self, p0: Point3f, p1: Point3f, p2: Point3f) -> (Point3f, Point3f, Point3f) {
        let (mut p0t, mut p1t, mut p2t) = self.perm.perm(
            p0 + self.neg_o, p1 + self.neg_o, p2 + self.neg_o
        );
        p0t.x += self.shear.x * p0t.z;
        p0t.y += self.shear.y * p0t.z;
        p1t.x += self.shear.x * p1t.z;
        p1t.y += self.shear.y * p1t.z;
        p2t.x += self.shear.x * p2t.z;
        p2t.y += self.shear.y * p2t.z;
        (p0t, p1t, p2t)
    }
}

impl Permulation {
    #[inline]
    pub fn perm(self, p0t: Point3f, p1t: Point3f, p2t: Point3f) -> (Point3f, Point3f, Point3f) {
        match self {
            Permulation::XZ => Permulation::permxz(p0t, p1t, p2t),
            Permulation::YZ => Permulation::permyz(p0t, p1t, p2t),
            Permulation::ZZ => Permulation::permzz(p0t, p1t, p2t),
        }
    }

    #[inline]
    pub fn permxz(p0t: Point3f, p1t: Point3f, p2t: Point3f) -> (Point3f, Point3f, Point3f)
    {
        (Point3f::new(p0t.y, p0t.z, p0t.x),
        Point3f::new(p1t.y, p1t.z, p1t.x),
        Point3f::new(p2t.y, p2t.z, p2t.x))
    }

    #[inline]
    pub fn permyz(p0t: Point3f, p1t: Point3f, p2t: Point3f) -> (Point3f, Point3f, Point3f)
    {
        (Point3f::new(p0t.z, p0t.x, p0t.y),
        Point3f::new(p1t.z, p1t.x, p1t.y),
        Point3f::new(p2t.z, p2t.x, p2t.y))
    }

    #[inline]
    pub fn permzz(p0t: Point3f, p1t: Point3f, p2t: Point3f) -> (Point3f, Point3f, Point3f)
    {
        (p0t, p1t, p2t)
    }
}