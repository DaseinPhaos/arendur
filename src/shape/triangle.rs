// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines triangle mesh and triangle instance
use geometry::prelude::*;
use super::Shape;
use std::ops;
use sample::*;
use std::sync::Arc;
use tobj;
use lighting::{Light, LightFlag, LightSample, LIGHT_AREA, SampleInfo, PathInfo};
use component::prelude::*;
use material::Material;
use texturing::prelude::*;
use spectrum::prelude::*;
use sample;

pub type Model = tobj::Model;

/// A triangle mesh
pub struct TriangleMesh {
    vertices: Vec<Point3f>,
    indices: Vec<usize>,
    tangents: Option<Vec<Vector3f>>,
    normals: Option<Vec<Vector3f>>,
    uvs: Option<Vec<Point2f>>,
    bbox: BBox3f,
    material: Arc<Material>,
    lighting_profile: Option<Arc<Texture<Texel=RGBSpectrumf>>>,
    pub name: String,
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

    // /// load meshes from an `.obj` file
    // #[inline]
    // pub fn load_from_file<P>(file_name: &P) -> Result<Vec<TriangleMesh>, tobj::LoadError>
    //     where P: AsRef<Path> + ?Sized
    // {
    //     let models = tobj::load_obj(file_name.as_ref())?;
    //     let mut ret = Vec::with_capacity(models.0.len());
    //     for model in models.0 {
    //         ret.push(model.into());
    //     }
    //     Ok(ret)
    // }

    // /// load meshes from an `.obj` file, applying `transform` to it
    // pub fn load_from_file_transformed<P>(file_name: &P, transform: Matrix4f)
    //     -> Result<Vec<TriangleMesh>, tobj::LoadError>
    //     where P: AsRef<Path> + ?Sized
    // {
    //     let models = tobj::load_obj(file_name.as_ref())?;
    //     let mut ret = Vec::with_capacity(models.0.len());
    //     for model in models.0 {
    //         ret.push(TriangleMesh::from_model_transformed(model, transform));
    //     }
    //     Ok(ret)
    // }

    pub fn from_model(
        model: Model, 
        material: Arc<Material>, 
        lighting_profile: Option<Arc<Texture<Texel=RGBSpectrumf>>>
    ) -> TriangleMesh {
        let mut bbox = {
            let p = Point3f::new(
                model.mesh.positions[0] as Float,
                model.mesh.positions[1] as Float,
                model.mesh.positions[2] as Float
            );
            BBox3f::new(p, p)
        };
        let vertices = map_f32s_to_point(&model.mesh.positions, |p| {
            bbox = bbox.extend(p);
            p
        });
        let indices: Vec<_> = model.mesh.indices.into_iter().map(|i| 
            i as usize
        ).collect();
        let normals = if model.mesh.normals.len() > 0 {
            Some(map_f32s_to_vec(&model.mesh.normals, |n| n))
        } else {
            None
        };
        let uvs = if model.mesh.texcoords.len() > 0 {
            Some(map_f32s_to_point2(&model.mesh.texcoords, |uv| uv))
        } else {
            None
        };
        let tangents = None;
        let name = model.name;
        TriangleMesh{
            vertices, indices, tangents, normals, 
            uvs, bbox, name, material, lighting_profile
        }
    }

    pub fn from_model_transformed(
        model: Model,
        transform: Matrix4f,
        material: Arc<Material>, 
        lighting_profile: Option<Arc<Texture<Texel=RGBSpectrumf>>>
    ) -> TriangleMesh {
        let mut bbox = {
            let mut p = Point3f::new(
                model.mesh.positions[0] as Float,
                model.mesh.positions[1] as Float,
                model.mesh.positions[2] as Float
            );
            p = transform.transform_point(p);
            BBox3f::new(p, p)
        };
        let vertices = map_f32s_to_point(&model.mesh.positions, |p| {
            let p = transform.transform_point(p);
            bbox = bbox.extend(p);
            p
        });
        let indices: Vec<_> = model.mesh.indices.into_iter().map(|i| 
            i as usize
        ).collect();
        let normals = if model.mesh.normals.len() > 0 {
            Some(map_f32s_to_vec(&model.mesh.normals, |n| transform.transform_norm(n)))
        } else {
            None
        };
        let uvs = if model.mesh.texcoords.len() > 0 {
            Some(map_f32s_to_point2(&model.mesh.texcoords, |uv| uv))
        } else {
            None
        };
        
        let tangents = None;
        let name = model.name;
        TriangleMesh{
            vertices, indices, tangents, normals, 
            uvs, bbox, name, material, lighting_profile
        }
    }
}

fn map_f32s_to_vec<F>(src: &[f32], mut f: F) -> Vec<Vector3f>
    where F: FnMut(Vector3f) -> Vector3f
{
    let retlen = src.len()/3;
    let mut ret = Vec::with_capacity(retlen);
    for i in 0..retlen {
        let v = unsafe {
            Vector3f::new(
                *src.get_unchecked(3*i) as Float,
                *src.get_unchecked(3*i+1) as Float,
                *src.get_unchecked(3*i+2) as Float
            )
        };
        ret.push(f(v));
    }
    ret
}

fn map_f32s_to_point<F>(src: &[f32], mut f: F) -> Vec<Point3f>
    where F: FnMut(Point3f) -> Point3f
{
    let retlen = src.len()/3;
    let mut ret = Vec::with_capacity(retlen);
    for i in 0..retlen {
        let v = unsafe {
            Point3f::new(
                *src.get_unchecked(3*i) as Float,
                *src.get_unchecked(3*i+1) as Float,
                *src.get_unchecked(3*i+2) as Float
            )
        };
        ret.push(f(v));
    }
    ret
}

fn map_f32s_to_point2<F>(src: &[f32], mut f: F) -> Vec<Point2f>
    where F: FnMut(Point2f) -> Point2f
{
    let retlen = src.len()/2;
    let mut ret = Vec::with_capacity(retlen);
    for i in 0..retlen {
        let v = unsafe {
            Point2f::new(
                *src.get_unchecked(2*i) as Float,
                *src.get_unchecked(2*i+1) as Float
            )
        };
        ret.push(f(v));
    }
    ret
}

impl IntoIterator for TriangleMesh {
    type Item = TriangleInstance;
    type IntoIter = TriangleInstance;

    #[inline]
    fn into_iter(self) -> TriangleInstance {
        TriangleInstance{
            mesh: Arc::new(self),
            idx: 0,
        }
    }
}

impl Iterator for TriangleInstance {
    type Item = TriangleInstance;

    #[inline]
    fn next(&mut self) -> Option<TriangleInstance> {
        if self.idx + 2 < self.mesh.indices.len() {
            let ret = TriangleInstance{
                mesh: Arc::clone(&self.mesh),
                idx: self.idx,
            };
            self.idx += 3;
            Some(ret)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.mesh.indices.len();
        if self.idx >= len {
            (0, Some(0))
        } else {
            let remain = (len - self.idx)/3;
            (remain, Some(remain))
        }
    }
}

/// An instance of triangle from a triangle mesh.
#[derive(Clone)]
pub struct TriangleInstance {
    mesh: Arc<TriangleMesh>,
    /// ith triangle into the `parent` mesh
    idx: usize,
}

impl TriangleInstance {
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
    fn compute_shading_at(&self, b: Vector3f, dpdu: Vector3f) -> DuvInfo
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
            let tbt = normal::get_basis_from(shading_normal);
            shading_tangent = tbt.0;
            shading_bitangent = tbt.1;
        };

        DuvInfo {
            dpdu: shading_tangent,
            dpdv: shading_bitangent,
            dndu: dndu,
            dndv: dndv,
        }
    }
}

impl ops::Index<usize> for TriangleInstance {
    type Output= Point3f;

    #[inline]
    fn index(&self, index: usize) -> &Point3f {
        let idx = self.vidx(index);
        &self.mesh.vertices[idx]
    }
}

impl Shape for TriangleInstance {
    #[inline]
    fn bbox_local(&self) -> BBox3f {
        let bbox = BBox3f::new(self.x(), self.y());
        bbox.extend(self.z())
    }

    #[inline]
    fn intersect_ray(&self, ray: &RawRay) -> Option<(Float, SurfaceInteraction)> {
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

        // conservative intersection
        let maxxt = p0t.x.max(p1t.x).max(p2t.x);
        let maxyt = p0t.y.max(p1t.y).max(p2t.y);
        let maxzt = p0t.z.max(p1t.z).max(p2t.z);
        let maxe = e0.max(e1).max(e2);

        let deltax = maxxt * float::eb_term(5. as Float);
        let deltay = maxyt * float::eb_term(5. as Float);
        let deltaz = maxzt * float::eb_term(3. as Float);

        let delta_err = 2. as Float * (
            float::eb_term(2. as Float) * maxxt * maxyt
             + deltay * maxxt + deltax * maxyt
        );

        let delta_t = 3. as Float * (
            float::eb_term(3. as Float) * maxe * maxzt
             + delta_err * maxzt + deltaz * maxe
        ) * inv_det.abs();

        if t <= delta_t { return None; }

        let uvs = self.uvs();
        let p0 = p0.to_vec();
        let p1 = p1.to_vec();
        let p2 = p2.to_vec();

        let phit = Point3f::from_vec(b0 * p0 + b1 * p1 + b2 * p2);
        let perr = float::eb_term(7. as Float) * Vector3f::new(
            (b0*p0.x).abs() + (b1*p1.x).abs() + (b2*p2.x).abs(),
            (b0*p0.y).abs() + (b1*p1.y).abs() + (b2*p2.y).abs(),
            (b0*p0.z).abs() + (b1*p1.z).abs() + (b2*p2.z).abs()
        );

        let uvhit = Point2f::from_vec(b0 * uvs.0.to_vec() + b1 * uvs.1.to_vec() + b2 * uvs.2.to_vec());

        let (dpdu, dpdv) = TriangleInstance::computedpduv(p0, p1, p2, uvs);


        let mut surface_interaction = SurfaceInteraction::new(
            phit, perr, -ray.direction(), uvhit,
            DuvInfo{
                dpdu: dpdu,
                dpdv: dpdv,
                dndu: Vector3f::zero(),
                dndv: Vector3f::zero(),
            },
            // Some(self.info())
        );
        surface_interaction.set_shading(
            self.compute_shading_at(Vector3f::new(b0, b1, b2), dpdu), true
        );
        Some((t, surface_interaction))
    }

    #[inline]
    fn surface_area(&self) -> Float {
        let a = self.x() - self.z();
        let b = self.x() - self.z();
        (0.5 as Float) * (a.cross(b).magnitude())
    }

    #[inline]
    fn sample(&self, sample: Point2f) -> (Point3f, Vector3f, Float) {
        let barycentrc = sample_uniform_triangle(sample);
        let p = barycentrc.x * self.x().to_vec() + barycentrc.y * self.y().to_vec() + (1. as Float - barycentrc.x - barycentrc.y) * self.z().to_vec();
        let p = Point3f::from_vec(p);
        let n = if let Some(ref norms) = self.mesh.normals {
            (norms[self.vidx(0)] * barycentrc.x + norms[self.vidx(1)] * barycentrc.y + norms[self.vidx(2)] * (1. as Float - barycentrc.x - barycentrc.y))
        } else {
            (self.y() - self.x()).cross(self.z() - self.x())
        };
        (p, n.normalize(), 1. as Float / self.surface_area())
    }
}

impl Composable for TriangleInstance {
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        self.bbox_local()
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        let r = Shape::intersect_ray(self, ray);
        if let Some((t, mut si)) = r {
            ray.set_max_extend(t);
            si.set_primitive(self);
            Some(si)
        } else {
            None
        }
    }

    #[inline]
    fn can_intersect(&self, ray: &RawRay) -> bool {
        Shape::can_intersect(self, ray)
    }

    #[inline]
    fn as_light(&self) -> &Light {
        self
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        3.0 as Float
    }
}

impl Light for TriangleInstance {
    #[inline]
    fn flags(&self) -> LightFlag {
        LIGHT_AREA
    }

    #[inline]
    fn is_delta(&self) -> bool {
        false
    }

    #[inline]
    fn evaluate_path(&self, pos: Point3f, dir: Vector3f) -> RGBSpectrumf {
        if let Some(ref lp) = self.mesh.lighting_profile {
            let p = pos + dir;
            // match `wi` against surface normal
            let ray = RawRay::from_od(p, -dir);
            if let Some((_t, si)) = Shape::intersect_ray(self, &ray) {
                // retrive (u, v)
                let dxy = DxyInfo::from_duv(&si.duv);
                return lp.evaluate(&si, &dxy);
            }
        }
        RGBSpectrumf::black()
    }

    fn evaluate_sampled(
        &self, pos: Point3f, sample: Point2f
    ) -> LightSample {
        let (l_pos, l_norm, l_pdf) = self.sample_wrt(pos, sample);
        let mut ret = LightSample{
            radiance: RGBSpectrumf::black(),
            pdf: l_pdf,
            pfrom: l_pos,
            pto: pos,
        };
        // match against surface normal
        if let Some(ref lp) = self.mesh.lighting_profile {
            let ldir = pos - l_pos;
            if ldir.dot(l_norm) > 0. as Float {
                let ray = RawRay::from_od(pos, -ldir);
                if let Some((_, si)) = Shape::intersect_ray(self, &ray) {
                    let dxy = DxyInfo::from_duv(&si.duv);
                    ret.radiance = lp.evaluate(&si, &dxy);
                }
            }
        }
        ret
    }

    #[inline]
    fn generate_path(&self, samples: SampleInfo) -> PathInfo {
        let (pos, norm, pdfpos) = self.sample(samples.pfilm);
        let (u, v) = normal::get_basis_from(norm);
        let dir = sample::sample_cosw_hemisphere(samples.plens);
        let dir = dir.x * u + dir.y * v + dir.z * norm;
        PathInfo{
            ray: RawRay::from_od(pos, dir),
            normal: norm,
            pdfpos: pdfpos,
            pdfdir: sample::pdf_cosw_hemisphere(dir.z),
            radiance: self.evaluate_path(pos, dir),
        }
    }

    #[inline]
    fn pdf_path(&self, pos: Point3f, dir: Vector3f, norm: Vector3f) -> (Float, Float) {
        (
            Light::pdf(self, pos, norm),
            sample::pdf_cosw_hemisphere(norm.dot(dir).abs())
        )
    }

    fn pdf(&self, pos: Point3f, wi: Vector3f) -> Float {
        self.pdf_wrt(pos, wi)
    }

    fn power(&self) -> RGBSpectrumf {
        if let Some(ref lp) = self.mesh.lighting_profile {
            debug_assert!(self.surface_area() >= 0. as Float);
            lp.mean() * self.surface_area() * float::pi()
        } else {
            RGBSpectrumf::black()
        }
    }
}

impl Primitive for TriangleInstance {
    #[inline]
    fn get_material(&self) -> &Material {
        &*self.mesh.material
    }

    #[inline]
    fn is_emissive(&self) -> bool {
        self.mesh.lighting_profile.is_some()
    }
}
