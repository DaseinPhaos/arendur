//! Defines triangle mesh and triangle instance

use geometry::*;
use shape::{Shape, ShapeInfo};
use std::ops;

static IDENTITY_MATRIX: Matrix4f = Matrix4f{
    x: Vector4f{x: 1.0 as Float, y: 0.0 as Float, z: 0.0 as Float, w: 0.0 as Float},
    y: Vector4f{x: 0.0 as Float, y: 1.0 as Float, z: 0.0 as Float, w: 0.0 as Float},
    z: Vector4f{x: 0.0 as Float, y: 0.0 as Float, z: 1.0 as Float, w: 0.0 as Float},
    w: Vector4f{x: 0.0 as Float, y: 0.0 as Float, z: 0.0 as Float, w: 1.0 as Float},
};

/// A triangle mesh
pub struct TriangleMesh {
    vertices: Vec<Point3f>,
    indices: Vec<usize>,
    tangents: Option<Vec<Vector3f>>,
    normals: Option<Vec<Vector3f>>,
    uvs: Option<Vec<Point2f>>,
    bbox: BBox3f,
    reverse_orientation: bool,
}

impl TriangleMesh {
    /// Count of triangles in the mesh
    #[inline]
    pub fn triangle_count(&self) -> usize {
        debug_assert!(self.indices.len() % 3 == 0);
        self.indices.len() / 3
    }

    /// Count of vertices in the mesh
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// bounding box, in local frame
    pub fn bounding(&self) -> BBox3f {
        self.bbox
    }
}

/// An instance of triangle from a triangle mesh.
pub struct TriangleInstance<'a> {
    mesh: &'a TriangleMesh,
    /// ith triangle into the `parent` mesh
    idx: usize,
}

impl<'a> TriangleInstance<'a> {
    /// return points in local frame
    #[inline]
    pub fn x(&self) -> Point3f {
        (*self)[0]
    }
    
    /// return points in local frame
    #[inline]
    pub fn y(&self) -> Point3f {
        (*self)[1]
    }

    /// return points in local frame
    #[inline]
    pub fn z(&self) -> Point3f {
        (*self)[2]
    }

    /// return uv-coordinates
    #[inline]
    pub fn uvs(&self) -> (Point2f, Point2f, Point2f) {
        if let Some(ref uvs) = self.mesh.uvs {(
            uvs[self.vidx(0)],
            uvs[self.vidx(1)],
            uvs[self.vidx(2)],
        )} else {(
            Point2f::new(0.0 as Float, 0.0 as Float),
            Point2f::new(1.0 as Float, 0.0 as Float),
            Point2f::new(1.0 as Float, 1.0 as Float),
        )}
    }

    /// return vertice indices in the parent mesh
    #[inline]
    pub fn vidx(&self, idx: usize) -> usize {
        debug_assert!(self.idx < self.mesh.indices.len());
        debug_assert!(self.idx % 3 == 0);
        debug_assert!(idx < 3);
        self.mesh.indices[idx + self.idx]
    }

    #[inline]
    fn computedpduv(p0: Vector3f, p1: Vector3f, p2: Vector3f, uvs: (Point2f, Point2f, Point2f)) -> (Vector3f, Vector3f) {
        let uvs = (
            uvs.0.to_vec(),
            uvs.1.to_vec(),
            uvs.2.to_vec(),
        );
        let duv02 = uvs.0 - uvs.2;
        let duv12 = uvs.1 - uvs.2;
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;
        let determinant = duv02.x * duv12.y - duv02.y * duv12.x;
        if determinant == 0.0 as Float {
            let up = dp02.cross(p0 - p1);
            let frame = Matrix3f::look_at(dp02, up);
            (frame.x, frame.z)
        } else {
            let inv_determinant = (1.0 as Float) / determinant;
            (
                (duv12.y * dp02 - duv02.y * dp12) * inv_determinant,
                (-duv12.x * dp02 + duv02.x * dp12) * inv_determinant,
            )
        }
    }

    #[inline]
    fn compute_shading_at(&self, b: Vector3f, dpdu: Vector3f) -> DerivativeInfo2D
    {
        let p0 = self.x();
        let p1 = self.y();
        let p2 = self.z();

        let (shading_normal, dndu, dndv) = if let Some(ref normals) = self.mesh.normals {
            let n0 = normals[self.vidx(0)];
            let n1 = normals[self.vidx(1)];
            let n2 = normals[self.vidx(2)];
            let surface_normal = (b.x * n0 + b.y * n1 + b.z * n2).normalize();
            let uvs = self.uvs();
            let (dndu, dndv) = TriangleInstance::computedpduv(n0, n1, n2, uvs);
            (surface_normal, dndu, dndv)
        } else {(
            (p2 - p0).cross(p1 - p0).normalize(),
            Vector3f::zero(),
            Vector3f::zero(),
        )};

        let mut shading_tangent = if let Some(ref tangents) = self.mesh.tangents {
            (b.x * tangents[self.vidx(0)] + b.y * tangents[self.vidx(1)] + b.z * tangents[self.vidx(2)]).normalize()
        } else {
            dpdu.normalize()
        };

        let mut shading_bitangent = shading_tangent.cross(shading_normal);
        if shading_bitangent.magnitude2() > (0.0 as Float) {
            shading_bitangent = shading_bitangent.normalize();
            shading_tangent = shading_bitangent.cross(shading_normal);
        } else {
            // FIXME: degenerate
            panic!("invalid shading")
        };

        DerivativeInfo2D {
            dpdu: shading_tangent,
            dpdv: shading_bitangent,
            dndu: dndu,
            dndv: dndv,
        }
    }
}

impl<'a> ops::Index<usize> for TriangleInstance<'a> {
    type Output= Point3f;

    #[inline]
    fn index(&self, index: usize) -> &Point3f {
        let idx = self.vidx(index);
        &self.mesh.vertices[idx]
    }
}

impl<'a> Shape for TriangleInstance<'a> {
    #[inline]
    fn info(&self) -> ShapeInfo {
        ShapeInfo::new(&IDENTITY_MATRIX, &IDENTITY_MATRIX, self.mesh.reverse_orientation)
    }

    #[inline]
    fn bbox_local(&self) -> BBox3f {
        let bbox = BBox3f::new(self.x(), self.y());
        bbox.extend(self.z())
    }

    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.bbox_local()
    }

    #[inline]
    fn intersect_ray<R: Ray>(&self, ray: &R) -> Option<(Float, SurfaceInteraction)> {
        let p0 = self.x();
        let p1 = self.y();
        let p2 = self.z();
        let stc = ray.shearing_transform();
        let (mut p0t, mut p1t, mut p2t) = stc.apply(p0, p1, p2);
        let e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let e2 = p0t.x * p1t.y - p0t.y * p1t.x;

        const ZERO: Float = 0.0 as Float;
        if (e0 < ZERO || e1 < ZERO || e2 < ZERO) && (e0 > ZERO || e1 > ZERO || e2 > ZERO) {
            return None;
        }
        let det = e0 + e1 + e2;
        if det == ZERO { return None; }

        p0t.z *= stc.shear.z;
        p1t.z *= stc.shear.z;
        p2t.z *= stc.shear.z;

        let tscaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        if det < ZERO && (tscaled >= ZERO || tscaled < ray.max_extend() * det) {
            return None;
        } else if det > ZERO && (tscaled <= ZERO || tscaled > ray.max_extend() * det) {
            return None;
        }

        let inv_det = (1.0 as Float) / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let t = tscaled * inv_det;

        let uvs = self.uvs();
        let p0 = p0.to_vec();
        let p1 = p1.to_vec();
        let p2 = p2.to_vec();

        let phit = Point3f::from_vec(b0 * p0 + b1 * p1 + b2 * p2);
        let uvhit = Point2f::from_vec(b0 * uvs.0.to_vec() + b1 * uvs.1.to_vec() + b2 * uvs.2.to_vec());

        let (dpdu, dpdv) = TriangleInstance::computedpduv(p0, p1, p2, uvs);

        let mut surface_interaction = SurfaceInteraction::new(
            phit, -ray.direction(), uvhit,
            DerivativeInfo2D{
                dpdu: dpdu,
                dpdv: dpdv,
                dndu: Vector3f::zero(),
                dndv: Vector3f::zero(),
            },
            Some(self.info())
        );
        surface_interaction.set_shading(
            self.compute_shading_at(Vector3f::new(b0, b1, b2), dpdu), true);

        Some((t, surface_interaction))
    }

    #[inline]
    fn surface_area(&self) -> Float {
        let a = self.x() - self.z();
        let b = self.x() - self.z();
        (0.5 as Float) * (a.cross(b).magnitude())
    }
}