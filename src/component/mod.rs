// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines renderable components in the world.

use geometry::prelude::*;
use lighting::Light;
use material::Material;

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
pub fn load_obj(path: &Path, transform: Matrix4f) -> Result<Vec<Arc<Composable>>, tobj::LoadError> {
    let (models, mtls) = tobj::load_obj(path)?;
    let mut texturess = HashMap::new();
    let mut materials: Vec<Arc<Material>> = Vec::with_capacity(mtls.len()+1);
    for mtl in mtls {
        let diffuse = ImageTexture::new_as_arc(
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
        let specular = ImageTexture::new_as_arc(
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
            value: (mtl.shininess / 100.).min(1.).max(0.) as Float
        };
        if specular.mean() == RGBSpectrumf::black() || !specular.mean().valid() {
            materials.push(Arc::new(MatteMaterial::new(
                diffuse, Arc::new(ConstantTexture{value: 0. as Float}), 
                None
            )));
        } else {
            // TODO: bump
            materials.push(Arc::new(PlasticMaterial::new(
                diffuse, specular, Arc::new(roughness), None
            )));
        }
        // materials.push(Arc::new(MatteMaterial::new(
        //     diffuse, Arc::new(ConstantTexture{value: 0. as Float}), 
        //     None
        // )));
    }
    materials.push(Arc::new(MatteMaterial::new(
        Arc::new(ConstantTexture{
            value: RGBSpectrumf::new(0.5 as Float, 0.6 as Float, 0.7 as Float)
        }),
        Arc::new(ConstantTexture{value: 0. as Float}), 
        None
    )));
    let mut shapes: Vec<Arc<Composable>> = Vec::new();
    for model in models {
        let mid = model.mesh.material_id.unwrap_or(materials.len()-1);
        // let mid = materials.len()-1;
        let mesh = TriangleMesh::from_model_transformed(model, transform);
        for shape in mesh {
            shapes.push(Arc::new(
                ShapedPrimitive::new(shape, materials[mid].clone(), None))
            );
        }
    }
    Ok(shapes)
}


pub mod shape;
pub mod transformed;
pub mod bvh;
pub mod naive;
pub mod prelude;
