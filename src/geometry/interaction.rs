// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Basic geometric interaction
use super::{RayDifferential, Ray, RawRay};
use super::foundamental::*;
use super::transform::TransformExt;
use super::float;
use component::Primitive;
use spectrum::{Spectrum, RGBSpectrumf};

/// Basic information about an interaction
#[derive(Debug, PartialEq, Copy, Clone)]
#[must_use]
pub struct InteractInfo {
    /// Position at which the interaction occurs
    pub pos: Point3f,
    /// position's error term
    pub pos_err: Vector3f,
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
            pos_err: t.transform_vector(self.pos_err),
            wo: t.transform_vector(self.wo),
            norm: t.transform_norm(self.norm),
        }
    }

    #[inline]
    pub fn offset_towards(&self, dir: Vector3f) -> Point3f {
        let nabs = Vector3f::new(
            self.norm.x.abs(), self.norm.y.abs(), self.norm.z.abs()
        );
        let edn = nabs.dot(self.pos_err);
        let mut offset = edn * self.norm;
        if dir.dot(self.norm) <= 0. as Float { offset = -offset; }
        let mut ret = self.pos + offset;
        if offset.x > 0. as Float {
            ret.x = float::next_up(ret.x);
        } else if offset.x < 0. as Float {
            ret.x = float::next_down(ret.x);
        }

        if offset.y > 0. as Float {
            ret.y = float::next_up(ret.y);
        } else if offset.y < 0. as Float {
            ret.y = float::next_down(ret.y);
        }

        if offset.z > 0. as Float {
            ret.z = float::next_up(ret.z);
        } else if offset.z < 0. as Float {
            ret.z = float::next_down(ret.z);
        }

        ret
    }
}

/// Differential information about some $p(u, v)$, $n(u, v)$
#[derive(Debug, PartialEq, Copy, Clone)]
#[must_use]
pub struct DuvInfo {
    /// partial differential of position along u
    pub dpdu: Vector3f,
    /// partial differential of position along v
    pub dpdv: Vector3f,
    /// partial differential of normal along u
    pub dndu: Vector3f,
    /// partial differential of normal along v
    pub dndv: Vector3f,
}

impl DuvInfo {
    pub fn apply_transform<T>(&self, t: &T) -> Self
        where T: TransformExt
    {
        DuvInfo {
            dpdu: t.transform_vector(self.dpdu),
            dpdv: t.transform_vector(self.dpdv),
            dndu: t.transform_norm(self.dndu),
            dndv: t.transform_norm(self.dndv),
        }
    }
}

/// Interaction at some surface denoted as $f(u, v)$
#[derive(Copy, Clone)]
#[must_use]
pub struct SurfaceInteraction<'b> {
    /// Basic information about the interaction
    pub basic: InteractInfo,
    /// uv-position
    pub uv: Point2f,
    /// partial derivatives along uv
    pub duv: DuvInfo,
    /// Normal used for shading, might be different from `self.basic.norm`
    pub shading_norm: Vector3f,
    /// uv-derivatives used for shading, might be different from `self.duv`
    pub shading_duv: DuvInfo,
    // /// shape information of the surface
    // pub shape_info: Option<&'a ShapeInfo>,
    /// primitive hit
    pub primitive_hit: Option<&'b Primitive>,
}

use std::fmt::*;
impl<'a> Debug for SurfaceInteraction<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "SI{{\n\tbasic: {:?}, \n\tuv:{:?}, \n\tduv:{:?}, \n\tns:{:?}", 
            self.basic, self.uv, self.duv, self.shading_norm
        )
    }
}

impl<'b> SurfaceInteraction<'b> {
    /// Construct a new instance from given info
    pub fn new(
        pos: Point3f,
        perr: Vector3f,
        wo: Vector3f,
        uv: Point2f,
        duv: DuvInfo
    ) -> SurfaceInteraction<'b> {
        let norm = duv.dpdu.cross(duv.dpdv).normalize();

        // if let Some(shape_info) = shape_info {
        //     if shape_info.reverse_orientation ^ shape_info.swap_handedness {
        //         norm = -norm;
        //     }
        // }
        
        SurfaceInteraction {
            basic: InteractInfo {
                pos: pos,
                pos_err: perr,
                wo: wo,
                norm: norm,
            },
            uv: uv,
            duv: duv,
            shading_norm: norm,
            shading_duv: duv,
            // shape_info: shape_info,
            primitive_hit: None,
        }
    }

    /// set `self.shading_`s.
    /// if `orient_norm_by_shading`, `self.norm` would be oriented
    /// according to `self.shading_norm`. Otherwise, the reverse.
    pub fn set_shading(&mut self, duv: DuvInfo, orient_norm_by_shading: bool)
    {
        self.duv = duv;
        // FIXME: should update according to more cretiarias
        let mut norm = duv.dpdu.cross(duv.dpdv).normalize();

        if self.basic.norm.dot(norm) < (0.0 as Float) {
            if orient_norm_by_shading {
                norm = -norm;
            } else {
                self.basic.norm = -self.basic.norm;
            }
        }

        self.shading_norm = norm;
    }

    pub fn set_primitive<P>(&mut self, primitive: &'b P)
        where P: Primitive
    {
        self.primitive_hit = Some(primitive)
    }

    pub fn apply_transform<T>(&self, t: &T) -> SurfaceInteraction<'b>
        where T: TransformExt
    {
        SurfaceInteraction{
            basic: self.basic.apply_transform(t),
            uv: self.uv,
            duv: self.duv.apply_transform(t),
            shading_norm: t.transform_norm(self.shading_norm),
            shading_duv: self.shading_duv.apply_transform(t),
            primitive_hit: self.primitive_hit,
        }
    }

    /// compute image plane differentials according to the differential ray
    pub fn compute_dxy(&self, ray_diff: &RayDifferential) -> DxyInfo {
        if let Some(ref diffs) = ray_diff.diffs {
            // hitting plane is given by `(self.basic.pos, self.basic.norm)`.
            let d = self.basic.norm.dot(self.basic.pos.to_vec());
            let tx = (d - self.basic.norm.dot(diffs.0.origin().to_vec())) / self.basic.norm.dot(diffs.0.direction());
            let px = diffs.0.evaluate(tx);
            let ty = (d - self.basic.norm.dot(diffs.1.origin().to_vec())) / self.basic.norm.dot(diffs.1.direction());
            let py = diffs.1.evaluate(ty);
            let dpdx = px - self.basic.pos;
            let dpdy = py - self.basic.pos;
            let dudxy = solve_over_constrained_2x3(dpdx, (self.duv.dpdu, self.duv.dpdv), self.basic.norm).unwrap_or(Vector2f::new(0.0 as Float, 0.0 as Float));
            let dvdxy = solve_over_constrained_2x3(dpdy, (self.duv.dpdu, self.duv.dpdv), self.basic.norm).unwrap_or(Vector2f::new(0.0 as Float, 0.0 as Float));
            DxyInfo{
                dpdx: dpdx, dpdy: dpdy,
                dudx: dudxy.x, dudy: dudxy.y,
                dvdx: dvdxy.x, dvdy: dvdxy.y,
            }
        } else {
            Default::default()
        }
    }

    #[inline]
    pub fn is_emissive(&self) -> bool {
        if let Some(hit) = self.primitive_hit {
            hit.is_emissive()
        } else {
            false
        }
    }

    #[inline]
    pub fn spawn_ray_differential(&self, dir: Vector3f, dxy: Option<&DxyInfo>) -> RayDifferential {
        let pos = self.basic.offset_towards(dir);
        let ray = RawRay::from_od(pos, dir);
        let diffs = if let Some(dxy) = dxy {
            let posdx = pos + dxy.dpdx;
            let posdy = pos + dxy.dpdy;
            Some((
                RawRay::from_od(posdx, dir), RawRay::from_od(posdy, dir) 
            ))
        } else {
            None
        };
        RayDifferential{
            ray: ray, diffs: diffs,
        }
    }

    #[inline]
    pub fn le(&self, dir: Vector3f) -> RGBSpectrumf {
        if let Some(hit) = self.primitive_hit {
            if hit.is_emissive() {
                return hit.evaluate_path(self.basic.pos, dir);
            }
        }
        RGBSpectrumf::black()
    }
}

/// Partial differential info about some `p(x, y)`, `u(x, y)`, `v(x, y)`
/// according to some `(x, y)` image space
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct DxyInfo {
    pub dpdx: Vector3f,
    pub dpdy: Vector3f,
    pub dudx: Float,
    pub dudy: Float,
    pub dvdx: Float,
    pub dvdy: Float,
}

impl DxyInfo {
    #[inline]
    pub fn from_duv(duv: &DuvInfo) -> DxyInfo {
        DxyInfo{
            dpdx: duv.dpdu,
            dpdy: duv.dpdv,
            dudx: 1. as Float,
            dudy: 0. as Float,
            dvdx: 0. as Float,
            dvdy: 1. as Float,
        }
    }
}

impl Default for DxyInfo {
    #[inline]
    fn default() -> DxyInfo {
        const ZERO: Float = 0.0 as Float;
        DxyInfo{
            dpdx: Vector3f::new(ZERO, ZERO, ZERO),
            dpdy: Vector3f::new(ZERO, ZERO, ZERO),
            dudx: ZERO,
            dudy: ZERO,
            dvdx: ZERO,
            dvdy: ZERO,
        }
    }
}

/// helper function to solve over constrained system given by
/// $M_{3*2}(x, y)^T = (a, b, c)^T$
#[inline]
fn solve_over_constrained_2x3(abc: Vector3f, m: (Vector3f, Vector3f), n: Vector3f) -> Option<Vector2f> {
    if n.x.abs() > n.y.abs() && n.x.abs() > n.z.abs() {
        Matrix2f::new(m.0.y, m.1.y, m.0.z, m.1.z)
        .invert().map(|m| {
            m * Vector2f::new(abc.y, abc.z)
        })
    } else if n.y.abs() > n.z.abs() {
        Matrix2f::new(m.0.x, m.1.x, m.0.z, m.1.z)
        .invert().map(|m| {
            m * Vector2f::new(abc.x, abc.z)
        })
    } else {
        Matrix2f::new(m.0.x, m.1.x, m.0.y, m.1.y)
        .invert().map(|m| {
            m * Vector2f::new(abc.x, abc.y)
        })
    }
}