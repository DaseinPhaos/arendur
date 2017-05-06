// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines renderable components in the world.

use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use tobj;
use lighting::Light;
use geometry::prelude::*;
use material::prelude::*;
use shape::prelude::*;
use texturing::prelude::*;
use spectrum::prelude::*;

/// A renderable composable component.
pub trait Composable: Sync + Send {
    /// returns bounding box in parent frame.
    fn bbox_parent(&self) -> BBox3f;

    /// test for intersection. Note that its guarantees are from `Shape`'s:
    /// - `ray` is specified in parent frame,
    /// - if hit, returns surface interaction data in *parent* frame.
    /// - if hit, `ray`'s `tmax` would be updated to the hitting `t`.
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction>;

    /// test if an intersection can occur. Might be more efficient
    #[inline]
    fn can_intersect(&self, ray: &RawRay) -> bool {
        let mut ray = ray.clone();
        self.intersect_ray(&mut ray).is_some()
    }

    fn as_light(&self) -> &Light {
        unimplemented!();
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        1.0 as Float
    }
}

// /// An aggregated renderable entity
// pub trait Aggregate: Composable {

// }

/// A renderable primitive
pub trait Primitive: Composable + Light {
    /// return if the primitive can emit lights
    fn is_emissive(&self) -> bool;

    /// return the material associated with this primitive
    fn get_material(&self) -> &Material;
}

/// Load an `.obj` file into a vector
pub fn load_obj(path: &Path, transform: Matrix4f) -> Result<Vec<ComponentPointer>, tobj::LoadError> {
    let (models, mtls) = tobj::load_obj(path)?;
    let mut texturess = HashMap::new();
    let mut bumps = HashMap::new();
    let mut materials: Vec<Arc<Material>> = Vec::with_capacity(mtls.len()+1);
    for mtl in mtls {
        // println!("{:?}", mtl);
        let diffuse = RGBImageTexture::new_as_arc(
            ImageInfo{
                name: mtl.diffuse_texture,
                trilinear: false,
                max_aniso: 16. as Float,
                wrapping: ImageWrapMode::Repeat,
                gamma: false,
                scale: 1. as Float,
            },
            UVMapping{
                scaling: Vector2f::new(1. as Float, 1. as Float),
                shifting: Vector2f::zero(),
            },
            &mut texturess
        ).unwrap_or(
            Arc::new(ConstantTexture{value: RGBSpectrum::new(
                mtl.diffuse[0], mtl.diffuse[1], mtl.diffuse[2]
            ) })
        );
        let specular = RGBImageTexture::new_as_arc(
            ImageInfo{
                name: mtl.specular_texture,
                trilinear: false,
                max_aniso: 16. as Float,
                wrapping: ImageWrapMode::Repeat,
                gamma: false,
                scale: 1. as Float,
            },
            UVMapping{
                scaling: Vector2f::new(1. as Float, 1. as Float),
                shifting: Vector2f::zero(),
            },
            &mut texturess
        ).unwrap_or(
            Arc::new(ConstantTexture{value: RGBSpectrum::new(
                mtl.specular[0], mtl.specular[1], mtl.specular[2]
            ) })
        );

        let roughness = ConstantTexture{
            value: ((1000. - mtl.shininess) / 1000.).min(1.).max(0.) as Float
        };

        let bump = LumaImageTexture::new_as_arc(
            ImageInfo{
                name: mtl.unknown_param.get("map_bump").map_or_else(|| String::new(), |r| r.to_owned()),
                trilinear: false,
                max_aniso: 16. as Float,
                wrapping: ImageWrapMode::Repeat,
                gamma: false,
                scale: 1. as Float,
            },
            UVMapping{
                scaling: Vector2f::new(1. as Float, 1. as Float),
                shifting: Vector2f::zero(),
            },
            &mut bumps
        );
        let illum = mtl.unknown_param.get("illum").map(|a| a.as_ref()).unwrap_or("2");
        let dissolve = mtl.dissolve as Float;
        // if illum == "4" {
        if illum.contains("4") {
            // specular transmittance
            materials.push(Arc::new(GlassMaterial::new(
                diffuse, specular, dissolve.min(1. as Float).max(0. as Float),
                mtl.optical_density, bump
            )));
        } else if !relative_eq!(dissolve, 1.0 as Float) {
            // glossy transmitance
            materials.push(Arc::new(TranslucentMaterial::new(
                diffuse, specular, Arc::new(roughness), dissolve, bump
            )));
        } else if specular.mean() == RGBSpectrumf::black() || !specular.mean().valid() {
            // diffuse reflection
            materials.push(Arc::new(MatteMaterial::new(
                diffuse, Arc::new(ConstantTexture{value: 0. as Float}), 
                bump
            )));
        } else {
            // glossy reflection
            materials.push(Arc::new(PlasticMaterial::new(
                diffuse, specular, Arc::new(roughness), bump
            )));
        }
    }
    materials.push(Arc::new(MatteMaterial::new(
        Arc::new(ConstantTexture{
            value: RGBSpectrumf::new(0.5 as Float, 0.6 as Float, 0.7 as Float)
        }),
        Arc::new(ConstantTexture{value: 0. as Float}), 
        None
    )));
    let mut shapes: Vec<ComponentPointer> = Vec::new();
    for model in models {
        let mid = model.mesh.material_id.unwrap_or(materials.len()-1);
        // let mid = materials.len()-1;
        let mesh = TriangleMesh::from_model_transformed(model, transform, materials[mid].clone(), None);
        for shape in mesh {
            shapes.push(
                shape.into()
            );
        }
    }
    Ok(shapes)
}

/// A thread-safe pointer to a composable component
/// We introduce this to increase data locality of the
/// widely used triangle components
#[derive(Clone)]
pub enum ComponentPointer {
    Triangle(TriangleInstance),
    Arc(Arc<Composable>),
}

impl Composable for ComponentPointer {
    /// returns bounding box in parent frame.
    #[inline]
    fn bbox_parent(&self) -> BBox3f {
        match *self {
            ComponentPointer::Arc(ref arc) => arc.bbox_parent(),
            ComponentPointer::Triangle(ref t) => Composable::bbox_parent(t),
        }
    }

    #[inline]
    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        match *self {
            ComponentPointer::Arc(ref arc) => arc.intersect_ray(ray),
            ComponentPointer::Triangle(ref t) => Composable::intersect_ray(t, ray),
        }
    }

    /// test if an intersection can occur. Might be more efficient
    #[inline]
    fn can_intersect(&self, ray: &RawRay) -> bool {
        match *self {
            ComponentPointer::Arc(ref arc) => arc.can_intersect(ray),
            ComponentPointer::Triangle(ref t) => Composable::can_intersect(t, ray),
        }
    }

    #[inline]
    fn as_light(&self) -> &Light {
        match *self {
            ComponentPointer::Arc(ref arc) => arc.as_light(),
            ComponentPointer::Triangle(ref t) => t.as_light(),
        }
    }

    #[inline]
    fn intersection_cost(&self) -> Float {
        match *self {
            ComponentPointer::Arc(ref arc) => arc.intersection_cost(),
            ComponentPointer::Triangle(ref t) => t.intersection_cost(),
        }
    }
}

impl From<Arc<Composable>> for ComponentPointer {
    #[inline]
    fn from(arc: Arc<Composable>) -> ComponentPointer {
        ComponentPointer::Arc(arc)
    }
}

impl From<TriangleInstance> for ComponentPointer {
    #[inline]
    fn from(t: TriangleInstance) -> ComponentPointer {
        ComponentPointer::Triangle(t)
    }
}

pub mod shape;
pub mod transformed;
pub mod bvh;
pub mod naive;
pub mod prelude;
