//! Basic geometry definitions

use super::cgmath_prelude::*;
use super::transform::TransformExt;
use super::super::shape::ShapeInfo;
// use super::float;

/// Basic information about an interaction
#[derive(PartialEq, Copy, Clone)]
pub struct InteractInfo {
    /// Position at which the interaction occurs
    pub pos: Point3f,
    /// Negative direction of the associated ray
    pub wo: Vector3f,
    /// Associated normal vector
    pub norm: Vector3f,
    // TODO: Placeholder for medium
}

impl InteractInfo {
    pub fn apply_transform<T>(&self, t: &T) -> Self
        where T: TransformExt
    {
        InteractInfo {
            pos: t.transform_point(self.pos),
            wo: t.transform_vector(self.wo),
            norm: t.transform_norm(self.norm),
        }
    }
}

/// Derivative information about some $(p, n) = f(u, v)$
#[derive(PartialEq, Copy, Clone)]
pub struct DerivativeInfo2D {
    /// partial derivative of position along u-axis
    pub dpdu: Vector3f,
    /// partial derivative of position along v-axis
    pub dpdv: Vector3f,
    /// partial derivative of normal along u-axis
    pub dndu: Vector3f,
    /// partial derivative of normal along v-axis
    pub dndv: Vector3f,
}

impl DerivativeInfo2D {
    pub fn apply_transform<T>(&self, t: &T) -> Self
        where T: TransformExt
    {
        DerivativeInfo2D {
            dpdu: t.transform_vector(self.dpdu),
            dpdv: t.transform_vector(self.dpdv),
            dndu: t.transform_norm(self.dndu),
            dndv: t.transform_norm(self.dndv),
        }
    }
}

/// Interaction at some surface denoted as $f(u, v)$
#[derive(Clone)]
pub struct SurfaceInteraction<'a> {
    /// Basic information about the interaction
    pub basic: InteractInfo,
    /// uv-position
    pub uv: Point2f,
    /// partial derivatives along uv
    pub duv: DerivativeInfo2D,
    /// Normal used for shading, might be different from `self.basic.norm`
    pub shading_norm: Vector3f,
    /// uv-derivatives used for shading, might be different from `self.duv`
    pub shading_duv: DerivativeInfo2D,
    /// shape information of the surface
    pub shape_info: Option<ShapeInfo<'a>>,
}

impl<'a> SurfaceInteraction<'a> {
    /// Construct a new instance from given info
    pub fn new(
        pos: Point3f,
        wo: Vector3f,
        uv: Point2f,
        duv: DerivativeInfo2D,
        shape_info: Option<ShapeInfo<'a>>,
    ) -> SurfaceInteraction<'a> {
        let mut norm = duv.dpdu.cross(duv.dpdv).normalize();

        if let Some(shape_info) = shape_info {
            if shape_info.reverse_orientation ^ shape_info.swap_handedness {
                norm = -norm;
            }
        }
        
        SurfaceInteraction {
            basic: InteractInfo {
                pos: pos,
                wo: wo,
                norm: norm,
            },
            uv: uv,
            duv: duv,
            shading_norm: norm,
            shading_duv: duv,
            shape_info: shape_info,
        }
    }

    /// set `self.shading_`s.
    /// if `orient_norm_by_shading`, `self.norm` would be oriented
    /// according to `self.shading_norm`. Otherwise, the reverse.
    pub fn set_shading(&mut self,duv: DerivativeInfo2D, orient_norm_by_shading: bool)
    {
        self.duv = duv;
        // FIXME: should update according to more cretiarias
        let mut norm = duv.dpdu.cross(duv.dpdv).normalize();

        if let Some(shape_info) = self.shape_info {
            if shape_info.reverse_orientation ^ shape_info.swap_handedness {
                norm = -norm;
            }
        }

        if self.basic.norm.dot(norm) < (0.0 as Float) {
            if orient_norm_by_shading {
                norm = -norm;
            } else {
                self.basic.norm = -self.basic.norm;
            }
        }

        self.shading_norm = norm;
    }

    pub fn apply_transform<T>(&self, t: &T) -> Self
        where T: TransformExt
    {
        /// TODO: implement this
        unimplemented!();
    }
}