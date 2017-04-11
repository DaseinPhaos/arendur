use super::cgmath_prelude::*;
use super::float;
use super::bbox::BBox3f;

/// A semi-infinite line
pub trait Ray {
    /// Returns where the ray originates
    fn origin(&self) -> Point3f;

    /// Sets the origin to `o`.
    /// Implementations must ensure that this is valid
    fn set_origin(&mut self, o: Point3f);

    /// Returns the max extend of the ray, in `self.direction().length()`
    fn max_extend(&self) -> Float;

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
}

/// A semi-infinite line specified by its `origin` and `dir`ection.
#[derive(PartialEq, Copy, Clone)]
pub struct RawRay {
    pub origin: Point3f,
    pub dir: Vector3f,
    /// maximum extent of the ray
    pub tmax: Float,
}

impl RawRay {
    #[inline]
    pub fn new(origin: Point3f, dir: Vector3f, tmax: Float) -> RawRay {
        RawRay {
            origin: origin,
            dir: dir,
            tmax: tmax,
        }
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
    }

    #[inline]
    fn max_extend(&self) -> Float {
        self.tmax
    }
    #[inline]
    fn direction(&self) -> Vector3f {
        self.dir
    }

    #[inline]
    fn set_direction(&mut self, d: Vector3f) {
        self.dir = d;
    }

    // FIXME: Deal with rounding error
    #[inline]
    fn apply_transform<T>(&self, t: &T) -> RawRay
        where T: Transform<Point3f>
    {
        RawRay {
            origin: t.transform_point(self.origin),
            dir: t.transform_vector(self.dir),
            tmax: self.tmax,
        }
    }
}