// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The commandline interface for arendur.

extern crate arendur;
extern crate clap;
extern crate env_logger;
extern crate serde_json;
extern crate rayon;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate flame;
use arendur::prelude::*;
use clap::{Arg, App};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use std::io::Read;
use std::time::*;
use std::str::FromStr;

fn main() {
    env_logger::init().unwrap();
    let matches = App::new("The Arendur CLI"
    ).version("0.1").author("Luxko<luxko@qq.com>")
    .arg(
        Arg::with_name("INPUT")
            .help("The scene description input file")
            .required(true)
    ).arg(
        Arg::with_name("thread")
            .help("The number of working t")
            .short("t")
            .long("thread")
            .value_name("NUM")
            .takes_value(true)
    ).get_matches();

    let input_filename = matches.value_of("INPUT").unwrap();
    if let Some(threads) = matches.value_of("thread") {
        let threads = usize::from_str(threads.as_ref()).expect("Invalid input: thread needs to be a number");
        rayon::initialize(rayon::Configuration::new().num_threads(threads)).unwrap();
    }

    let (scene, mut renderer) = parse_input(input_filename.as_ref()).expect("");
    println!("Start rendering");
    let sudato = Instant::now();
    renderer.render(&scene);
    
    let duration = sudato.elapsed();
    println!(
        "Done! Time used: {:.4}s", 
        duration.as_secs() as f64 + (duration.subsec_nanos() as f64/1_000_000_000.0f64)
    );
}

#[derive(Debug)]
enum ParsingError {
    IOError(std::io::Error),
    DecodeError(serde_json::error::Error),
}

fn parse_input(filename: &Path) -> Result<(Scene, StdPTRenderer), ParsingError> {
    let buf = {
        let mut file = std::fs::File::open(filename).map_err(|e| 
            ParsingError::IOError(e)
        )?;
        let mut buf = String::new();
        let _ = file.read_to_string(&mut buf).map_err(|e|
            ParsingError::IOError(e)
        )?;
        buf
    };
    let scenedesc: SceneDesc = serde_json::from_str(buf.as_ref()).map_err(|e|
        ParsingError::DecodeError(e)
    )?;

    let mut meshes = HashMap::new();
    let mut primitives: HashMap<_, Arc<Composable>> = HashMap::new();
    // let mut transformed =  HashMap::new();
    // let mut shapes = HashMap::new();
    let mut materials = HashMap::new();
    let mut rgbtextures = HashMap::new();
    let mut graytextures = HashMap::new();
    let mut rgbrefs = HashMap::new();
    let mut grayrefs = HashMap::new();

    let mut lights = Vec::new();

    for light in scenedesc.lights.iter() {
        lights.push(light.to_arc());
    }

    for component in scenedesc.components.iter() {
        let name = component.name.clone();
        if component.value.is_none() {
            println!("ignoring empty component {}", name);
            continue;
        }
        let component = component.value.as_ref().unwrap();
        match *component {
            ComponentDesc::Mesh{
                ref filename, transform
            } => {
                let transform = transform.unwrap_or(Matrix4f::identity());
                if let Ok(ptrs) = arendur::component::load_obj(
                    filename.as_ref(), transform
                ) {
                    meshes.insert(name, ptrs);
                } else {
                    println!("load mesh {} from {} failed.", name, filename);
                }
            },
            ComponentDesc::Shaped{
                ref shape, ref material, ref light,ref transform
            } => {
                let material = material.find_or_insert_with(&mut materials, |m| {
                    m.to_arc(&mut rgbtextures, &mut graytextures, &mut rgbrefs, &mut grayrefs)
                });
                let lt = light.clone().and_then(|l| l.to_arc(&mut rgbtextures, &mut rgbrefs));
                if let Some(material) = material {
                    let sp = match *shape {
                        ShapeDesc::Sphere(ref s) => {
                            ShapedPrimitive::new(
                                s.clone(), material.clone(), lt
                            )
                        }
                    };
                    let sp: Arc<Composable> = if let Some(transform) = *transform {
                        if let Some(inv) = transform.invert() {
                            let sp = Arc::new(TransformedComposable::new(
                                sp, Arc::new(transform), Arc::new(inv)
                            ));
                            if sp.is_emissive() {
                                lights.push(sp.clone());
                            }
                            sp
                        } else {
                            let sp = Arc::new(sp);
                            if sp.is_emissive() {
                                lights.push(sp.clone());
                            }
                            sp
                        }
                    } else {
                        let sp = Arc::new(sp);
                        if sp.is_emissive() {
                            lights.push(sp.clone());
                        }
                        sp
                    };
                    
                    
                    primitives.insert(name, sp);
                } else {
                    println!("load shape {} failed", name);
                }
            },
            ComponentDesc::Transformed{
                transform, ref original
            } => {
                let inv = transform.invert();
                if inv.is_none() {
                    println!("load transformed {} failed, invalid matrix invert", name);
                    continue;
                }
                let inv = inv.unwrap();
                let t = if let Some(orishape) = primitives.get(original) {
                    Arc::new(TransformedComposable::new(
                        orishape.clone(), Arc::new(transform), Arc::new(inv))
                    )
                } else {
                    println!("load transformed {} fialed, original doesn't exists", name);
                    continue;
                };
                primitives.insert(name, t);
            }
        }
    }

    let mut components = Vec::new();
    for mut mesh in meshes {
        components.append(&mut mesh.1);
    }
    for primitive in primitives {
        components.push(primitive.1.into());
    }
    let bvh = BVH::new(&components, BVHStrategy::SAH);

    let scene = Scene::new(lights, Arc::new(bvh));
    let renderer = StdPTRenderer::new(
        scenedesc.sampler, Arc::new(scenedesc.camera),
        &scenedesc.outputfilename, scenedesc.max_depth,
        scenedesc.multithreaded
    );
    Ok((scene, renderer))
}

#[derive(Serialize, Deserialize, Clone)]
struct SceneDesc {
    lights: Vec<LightDesc>,
    components: Vec<Named<ComponentDesc>>,
    sampler: StdStrataSampler,
    camera: PerspecCam,
    multithreaded: bool,
    max_depth: usize,
    outputfilename: String,
}

#[derive(Serialize, Deserialize, Clone)]
enum ComponentDesc {
    Mesh{
        filename: String,
        transform: Option<Matrix4f>,
    },
    Shaped{
        shape: ShapeDesc,
        material: Named<MaterialDesc>,
        light: Option<Named<RGBTextureDesc>>,
        transform: Option<Matrix4f>,
    },
    Transformed{
        transform: Matrix4f,
        original: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
struct Named<T> {
    name: String,
    value: Option<T>,
}

impl<T> Named<T> {
    fn find_or_insert_with<'a, V, F>(
        &self, map: &'a mut HashMap<String, V>, f: F
    ) -> Option<&'a V>
        where V: 'a,
              F: FnOnce(&T) -> Option<V>
    {
        if let Some(ref t) = self.value {
            if let Some(v) = f(t) {
                map.insert(self.name.clone(), v);
            }
        }
        map.get(&self.name)
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum ShapeDesc {
    Sphere(Sphere),
}

#[derive(Serialize, Deserialize, Clone)]
enum MaterialDesc {
    Matte{
        kd: Named<RGBTextureDesc>,
        sigma: Named<GrayTextureDesc>,
        bump: Option<Named<GrayTextureDesc>>,
    },
    Glass{
        diffuse: Named<RGBTextureDesc>,
        specular: Named<RGBTextureDesc>,
        roughness: Named<GrayTextureDesc>,
        bump: Option<Named<GrayTextureDesc>>,
        eta: Float,
    },
    Plastic{
        diffuse: Named<RGBTextureDesc>,
        specular: Named<RGBTextureDesc>,
        roughness: Named<GrayTextureDesc>,
        bump: Option<Named<GrayTextureDesc>>,
    },
    Translucent{
        diffuse: Named<RGBTextureDesc>,
        specular: Named<RGBTextureDesc>,
        roughness: Named<GrayTextureDesc>,
        bump: Option<Named<GrayTextureDesc>>,
        dissolve: Float,
    }
}

impl MaterialDesc {
    fn to_arc(
        &self, 
        rgbs: &mut HashMap<String, Arc<Texture<Texel=RGBSpectrumf>>>,
        grays: &mut HashMap<String, Arc<Texture<Texel=Float>>>,
        rgb_refs: &mut RGBMipMapHashTable<Float>,
        gray_refs: &mut LumaMipMapHashTable<Float>
    ) -> Option<Arc<Material>> {
        match *self {
            MaterialDesc::Matte{
                ref kd, ref sigma, ref bump
            } => {
                let kdt = kd.to_arc(rgbs, rgb_refs);
                let sigmat = sigma.to_arc(grays, gray_refs);
                let bumpt = bump.clone().and_then(|bn| {
                    bn.to_arc(grays, gray_refs)
                });
                if kdt.is_some() && sigmat.is_some() {
                    Some(Arc::new(MatteMaterial::new(
                        kdt.unwrap(), sigmat.unwrap(), bumpt
                    )))
                } else {
                    None
                }
            },
            MaterialDesc::Glass{
                ref diffuse, ref specular, ref roughness, ref bump, eta
            } => {
                let diffuse = diffuse.to_arc(rgbs, rgb_refs);
                let specular = specular.to_arc(rgbs, rgb_refs);
                let roughness = roughness.to_arc(grays, gray_refs);
                let bump = bump.clone().and_then(
                    |b| b.to_arc(grays, gray_refs)
                );
                if diffuse.is_some() && specular.is_some() && roughness.is_some() {
                    Some(Arc::new(GlassMaterial::new(
                        diffuse.unwrap(), specular.unwrap(), 
                        roughness.unwrap(), eta, bump
                    )))
                } else {
                    None
                }
            },
            MaterialDesc::Plastic{
                ref diffuse, ref specular, ref roughness, ref bump,
            } => {
                let diffuse = diffuse.to_arc(rgbs, rgb_refs);
                let specular = specular.to_arc(rgbs, rgb_refs);
                let roughness = roughness.to_arc(grays, gray_refs);
                let bump = bump.clone().and_then(
                    |b| b.to_arc(grays, gray_refs)
                );
                if diffuse.is_some() && specular.is_some() && roughness.is_some() {
                    Some(Arc::new(PlasticMaterial::new(
                        diffuse.unwrap(), specular.unwrap(), 
                        roughness.unwrap(), bump
                    )))
                } else {
                    None
                }
            },
            MaterialDesc::Translucent{
                ref diffuse, ref specular, ref roughness, ref bump, dissolve
            } => {
                let diffuse = diffuse.to_arc(rgbs, rgb_refs);
                let specular = specular.to_arc(rgbs, rgb_refs);
                let roughness = roughness.to_arc(grays, gray_refs);
                let bump = bump.clone().and_then(
                    |b| b.to_arc(grays, gray_refs)
                );
                if diffuse.is_some() && specular.is_some() && roughness.is_some() {
                    Some(Arc::new(TranslucentMaterial::new(
                        diffuse.unwrap(), specular.unwrap(), 
                        roughness.unwrap(), dissolve, bump
                    )))
                } else {
                    None
                }
            },
        }
        
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum RGBTextureDesc{
    Image{
        info: ImageInfo,
        mapping: UVMapping,
    },
    Constant{
        value: RGBSpectrumf,
    },
    Product{
        ta: String,
        tb: String,
    },
}

impl Named<RGBTextureDesc> {
    fn to_arc(
        &self,
        textures: &mut HashMap<String, Arc<Texture<Texel=RGBSpectrumf>>>,
        refs: &mut RGBMipMapHashTable<Float>
    ) -> Option<Arc<Texture<Texel=RGBSpectrumf>>> {
        let value = if let Some(t) = textures.get(&self.name) {
            return Some(t.clone());
        } else {
            if self.value.is_none() { return None; }
            self.value.as_ref().unwrap()
        };
        match *value {
            RGBTextureDesc::Image{
                ref info, ref mapping
            } => {
                RGBImageTexture::new_as_arc(info.clone(), mapping.clone(), refs)
            }
            RGBTextureDesc::Constant{
                value
            } => {
                Some(Arc::new(ConstantTexture{value}))
            }
            RGBTextureDesc::Product{
                ref ta, ref tb
            } => {
                let ta = textures.get(ta);
                let tb = textures.get(tb);
                if ta.is_some() && tb.is_some() {
                    Some(Arc::new(ProductTexture{
                        t0: ta.unwrap().clone(),
                        t1: tb.unwrap().clone(),
                    }))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum GrayTextureDesc{
    Image{
        info: ImageInfo,
        mapping: UVMapping,
    },
    Constant{
        value: Float,
    },
    Product{
        ta: String,
        tb: String,
    },
}

impl Named<GrayTextureDesc> {
    fn to_arc(
        &self,
        textures: &mut HashMap<String, Arc<Texture<Texel=Float>>>,
        refs: &mut LumaMipMapHashTable<Float>
    ) -> Option<Arc<Texture<Texel=Float>>> {
        let value = if let Some(t) = textures.get(&self.name) {
            return Some(t.clone());
        } else {
            if self.value.is_none() { return None; }
            self.value.as_ref().unwrap()
        };
        match *value {
            GrayTextureDesc::Image{
                ref info, ref mapping
            } => {
                LumaImageTexture::new_as_arc(info.clone(), mapping.clone(), refs)
            }
            GrayTextureDesc::Constant{
                value
            } => {
                Some(Arc::new(ConstantTexture{value}))
            }
            GrayTextureDesc::Product{
                ref ta, ref tb
            } => {
                let ta = textures.get(ta);
                let tb = textures.get(tb);
                if ta.is_some() && tb.is_some() {
                    Some(Arc::new(ProductTexture{
                        t0: ta.unwrap().clone(),
                        t1: tb.unwrap().clone(),
                    }))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum LightDesc {
    Point(PointLight),
    Spot(SpotLight),
    Distant(DistantLight),
    // Area(String),
}

impl LightDesc {
    fn to_arc(&self) -> Arc<Light> {
        match *self {
            LightDesc::Point(p) => {
                Arc::new(p)
            },
            LightDesc::Spot(p) => {
                Arc::new(p)
            },
            LightDesc::Distant(d) => {
                Arc::new(d)
            }
        }
    }
}