// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines a perspective camera

use geometry::prelude::*;
use super::{Camera, SampleInfo, ImportanceSample};
use super::projective::ProjCameraInfo;
use super::film::Film;
use spectrum::{RGBSpectrumf, Spectrum};
use sample;
use std;
use serde;
use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeStruct};
use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};

/// A perspective camera
#[derive(Clone)]
pub struct PerspecCam {
    view_parent: Matrix4f,
    parent_view: Matrix4f,
    proj_info: ProjCameraInfo,
    dx: Vector3f,
    dy: Vector3f,
    /// lens_radius, focal_distance; if presented
    lens: Option<(Float, Float)>,
    film: Film,
    area: Float,
    znear: Float,
    zfar: Float,
    fov: Float,
}

impl PerspecCam {
    /// Construction
    pub fn new(
        parent_view: Matrix4f,
        screen: BBox2f,
        znear: Float,
        zfar: Float,
        fov: Float,
        lens: Option<(Float, Float)>,
        film: Film
    ) -> PerspecCam {
        let view_parent = parent_view.inverse_transform().expect("matrix inversion failure");
        let resolution = film.resolutionf();
        let proj_info = ProjCameraInfo::new(
            PerspecCam::perspective_transform(fov, znear, zfar),
            screen, resolution
        );
        
        let mut pview_min = proj_info.raster_view.transform_point(
            Point3f::new(0. as Float, 0. as Float, 0. as Float)
        );
        pview_min /= pview_min.z;
        let mut pview_max = proj_info.raster_view.transform_point(
            Point3f::new(resolution.x, resolution.y, 0. as Float)
        );
        pview_max /= pview_max.z;
        let area = (pview_max.x - pview_min.x)*(pview_max.y - pview_min.y);

        let or2v = proj_info.raster_view.transform_point(
            Point3f::new(1.0 as Float, 0.0 as Float, 0.0 as Float)
        );
        let dx = proj_info.raster_view.transform_point(
            Point3f::new(1.0 as Float, 0.0 as Float, 0.0 as Float)
        ) - or2v;
        let dy = proj_info.raster_view.transform_point(
            Point3f::new(0.0 as Float, 1.0 as Float, 0.0 as Float)
        ) - or2v;
        PerspecCam{
            view_parent,
            parent_view,
            proj_info,
            dx,
            dy,
            lens,
            film,
            area,
            znear,
            zfar,
            fov,
        }
    }

    /// `fov` in radians
    pub fn perspective_transform(fov: Float, znear: Float, zfar: Float) -> Matrix4f {
        assert!(znear < zfar);
        assert!(fov < float::pi());
        let one = Float::one();
        let zero = Float::zero();
        let persp = Matrix4f::new(
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, zfar/(zfar-znear), one,
            zero, zero, -zfar*znear/(zfar-znear), zero
        );

        let inv_tan = one/ ((fov * 0.5 as Float).tan());
        Matrix4f::from_nonuniform_scale(inv_tan, inv_tan, one) * persp     
    }

    pub fn look_from(&mut self, eye: Point3f, to: Point3f, up: Vector3f) {
        let f = (to - eye).normalize();
        let s = up.cross(f).normalize();
        let u = f.cross(s);

        self.parent_view = Matrix4::new(
            s.x.clone(), u.x.clone(), f.x.clone(), Float::zero(),
            s.y.clone(), u.y.clone(), f.y.clone(), Float::zero(),
            s.z.clone(), u.z.clone(), f.z.clone(), Float::zero(),
            -eye.dot(s), -eye.dot(u), -eye.dot(f), Float::one()
        );
        self.view_parent = self.parent_view.inverse_transform().unwrap();
    }
}


impl Serialize for PerspecCam {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut state = s.serialize_struct("PerspecCam", 7)?;
        state.serialize_field("transform", &self.parent_view)?;
        state.serialize_field("screen", &self.proj_info.screen)?;
        state.serialize_field("znear", &self.znear)?;
        state.serialize_field("zfar", &self.zfar)?;
        state.serialize_field("fov", &self.fov)?;
        state.serialize_field("lens", &self.lens)?;
        state.serialize_field("film", &self.film)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PerspecCam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Transform, Screen, Znear, Zfar, Fov, Lens, Film }

        struct SamplerVisitor;
        impl<'de> Visitor<'de> for SamplerVisitor {
            type Value = PerspecCam;
            fn expecting(&self, fmter: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmter.write_str("struct PerspecCam")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                where V: SeqAccess<'de>
            {
                let transform = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let screen = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let znear = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let zfar = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
                let fov = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
                let lens = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(5, &self))?;
                let film = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(6, &self))?;
                Ok(PerspecCam::new(transform, screen, znear, zfar, fov, lens, film))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                where V: MapAccess<'de>
            {
                let mut transform = None;
                let mut screen = None;
                let mut znear = None;
                let mut zfar = None;
                let mut fov = None;
                let mut lens = None;
                let mut film = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Transform => {
                            if transform.is_some() {
                                return Err(serde::de::Error::duplicate_field("transform"));
                            }
                            transform = Some(map.next_value()?);
                        }
                        Field::Screen => {
                            if screen.is_some() {
                                return Err(serde::de::Error::duplicate_field("screen"));
                            }
                            screen = Some(map.next_value()?);
                        }
                        Field::Znear => {
                            if znear.is_some() {
                                return Err(serde::de::Error::duplicate_field("znear"));
                            }
                            znear = Some(map.next_value()?);
                        }
                        Field::Zfar => {
                            if zfar.is_some() {
                                return Err(serde::de::Error::duplicate_field("zfar"));
                            }
                            zfar = Some(map.next_value()?);
                        }
                        Field::Fov => {
                            if fov.is_some() {
                                return Err(serde::de::Error::duplicate_field("fov"));
                            }
                            fov = Some(map.next_value()?);
                        }
                        Field::Lens => {
                            if lens.is_some() {
                                return Err(serde::de::Error::duplicate_field("lens"));
                            }
                            lens = Some(map.next_value()?);
                        }
                        Field::Film => {
                            if film.is_some() {
                                return Err(serde::de::Error::duplicate_field("film"));
                            }
                            film = Some(map.next_value()?);
                        }
                    }
                }
                let transform = transform.ok_or_else(|| 
                    serde::de::Error::missing_field("transform")
                )?;
                let screen = screen.ok_or_else(|| 
                    serde::de::Error::missing_field("screen")
                )?;
                let znear = znear.ok_or_else(|| 
                    serde::de::Error::missing_field("znear")
                )?;
                let zfar = zfar.ok_or_else(|| 
                    serde::de::Error::missing_field("zfar")
                )?;
                let fov = fov.ok_or_else(|| 
                    serde::de::Error::missing_field("fov")
                )?;
                let lens = lens.ok_or_else(|| 
                    serde::de::Error::missing_field("lens")
                )?;
                let film = film.ok_or_else(|| 
                    serde::de::Error::missing_field("film")
                )?;

                Ok(PerspecCam::new(
                    transform, screen, znear, zfar, fov, lens, film
                ))
            }
        }
        const FIELDS: &[&str] = &["transform", "screen", "znear", "zfar", "fov", "lens", "film"];
        deserializer.deserialize_struct("PerspecCam", FIELDS, SamplerVisitor)
    }
}

impl Camera for PerspecCam {
    fn parent_to_view(&self) -> Matrix4f {
        self.parent_view
    }

    fn view_to_parent(&self) -> Matrix4f {
        self.view_parent
    }

    fn generate_path(&self, sample_info: SampleInfo) -> RawRay {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float), pview.to_vec().normalize());

        if let Some((r, d)) = self.lens {
            debug_assert!(r>0.0 as Float);
            debug_assert!(d>0.0 as Float);
            let plens = r * sample::sample_concentric_disk(sample_info.plens);
            let ft = d/ray.direction().z;
            let pfocus = ray.evaluate(ft);
            let new_origin = Point3f::new(plens.x, plens.y, 0.0 as Float);
            ray = RawRay::from_od(
                new_origin,
                (pfocus - new_origin).normalize()
            );
        }
        // TODO: update ray medium
        self.view_parent.transform_ray(&ray)
    }

    fn generate_path_differential(&self, sample_info: SampleInfo) -> RayDifferential {
        let pfilm = Point3f::new(sample_info.pfilm.x, sample_info.pfilm.y, 0.0 as Float);
        let pview = self.proj_info.raster_view.transform_point(pfilm);
        let mut ray = RawRay::from_od(
            Point3f::new(0.0 as Float, 0.0 as Float, 0.0 as Float), 
            pview.to_vec().normalize()
        );

        if let Some((r, d)) = self.lens {
            debug_assert!(r>0.0 as Float);
            debug_assert!(d>0.0 as Float);
            let plens = r * sample::sample_concentric_disk(sample_info.plens);
            let ft = d/ray.direction().z;
            let pfocus = ray.evaluate(ft);
            let new_origin = Point3f::new(plens.x, plens.y, 0.0 as Float);
            ray = RawRay::from_od(
                new_origin,
                (pfocus - new_origin).normalize()
            );
        }
        // TODO: account for lens
        let rx = RawRay::from_od(ray.origin(), (pview.to_vec()+self.dx).normalize());
        let ry = RawRay::from_od(ray.origin(), (pview.to_vec()+self.dy).normalize());
        let ret = RayDifferential{
            ray: ray,
            diffs: Some((rx, ry)),
        };
        self.view_parent.transform_ray_differential(&ret)
    }

    #[inline]
    fn get_film(&self) -> &Film {
        &self.film
    }

    #[inline]
    fn get_film_mut(&mut self) -> &mut Film {
        &mut self.film
    }

    fn evaluate_importance(
        &self, pos: Point3f, dir: Vector3f
    ) -> Option<(RGBSpectrumf, Point2f)> {
        let p2v = self.parent_view;
        let dir_view = p2v.transform_vector(dir);
        let costheta = dir_view.z;
        if costheta <= 0. as Float { return None; }
        let focus_t = if let Some(lens) = self.lens {
            lens.1 / costheta
        } else {
            1. as Float / costheta
        };
        let pos_view = p2v.transform_point(pos);
        let focus_view = pos_view + dir_view * focus_t;
        let p_raster = (
            self.proj_info.screen_raster*self.proj_info.view_screen
        ).transform_point(focus_view);
        let p_raster = Point2::new(p_raster.x, p_raster.y);
        
        let bound: BBox2<isize> = BBox2::new(Point2::new(0, 0), self.film.resolution().cast());
        if !bound.contain_lb(p_raster.cast()) { return None; }

        let costheta2 = costheta * costheta;
        let lens_area = if let Some(lens) = self.lens {
            float::pi() * lens.0 * lens.0
        } else {
            1. as Float
        };
        let importance = 1. as Float / (self.area * lens_area * costheta2 * costheta2);
        Some((
            RGBSpectrumf::new(importance, importance, importance),
            p_raster
        ))
    }

    fn evaluate_importance_sampled(
        &self, posw: Point3f, sample: Point2f
    ) -> (ImportanceSample, Point2f) {
        let plens = if let Some((r, _)) = self.lens {
            r* sample::sample_concentric_disk(sample)
        } else {
            Point2f::new(0. as Float, 0. as Float)
        };
        let pfrom = self.view_parent.transform_point(
            Point3f::new(plens.x, plens.y, 0. as Float)
        );
        let pto = posw;
        let mut dir = pfrom - pto;
        let dist2 = dir.magnitude2();
        dir /= dist2.sqrt();
        let (importance, praster) = if let Some((i, pr)) = self.evaluate_importance(pto, -dir) {
            (i, pr)
        } else {
            (RGBSpectrumf::black(), Point2f::new(0. as Float, 0. as Float))
        };
        let pdf = if let Some((r, _)) = self.lens {
            let norm = self.view_parent.transform_vector(
                Vector3f::new(0. as Float, 0. as Float, 1. as Float)
            );
            dist2 / (dir.dot(norm).abs()*r*r*float::pi())
        } else {
            1. as Float
        };
        (ImportanceSample{
            radiance: importance,
            pdf: pdf,
            pfrom: pfrom,
            pto: posw,
        }, praster)
    }

    fn pdf(&self, pos: Point3f, dir: Vector3f) -> (Float, Float) {
        let ret = (0. as Float, 0. as Float);
        let p2v = self.parent_view;
        let dir_view = p2v.transform_vector(dir);
        let costheta = dir_view.z;
        if costheta <= 0. as Float { return ret; }
        let focus_t = if let Some(lens) = self.lens {
            lens.1 / costheta
        } else {
            1. as Float / costheta
        };
        let pos_view = p2v.transform_point(pos);
        let focus_view = pos_view + dir_view * focus_t;
        let p_raster = (
            self.proj_info.screen_raster*self.proj_info.view_screen
        ).transform_point(focus_view);
        let p_raster = Point2::new(p_raster.x, p_raster.y);
        
        let bound: BBox2<isize> = BBox2::new(Point2::new(0, 0), self.film.resolution().cast());
        if !bound.contain_lb(p_raster.cast()) { return ret; }

        let lens_area = if let Some(lens) = self.lens {
            float::pi() * lens.0 * lens.0
        } else {
            1. as Float
        };

        (
            1. as Float/lens_area, // pdfpos
            1. as Float/(self.area * costheta * costheta * costheta) // pdfdir
        )
    }
}
