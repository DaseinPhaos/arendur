// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines microfacet distribution

use super::*;
use super::fresnel::*;

/// A microfacet
pub trait MicrofacetDistribution {
    /// Given surface normal `wh`, returns the differential
    /// area of microfacets
    fn distribution(&self, wh: Vector3f) -> Float;

    /// Given a direction `w`, return masked surface area
    /// per visible surface area
    fn lambda(&self, w: Vector3f) -> Float;

    /// fraction of facets of `wh` visible from `w`:
    #[inline]
    fn visible(&self, w: Vector3f) -> Float {
        1. as Float / (1. as Float + self.lambda(w))
    }

    /// fraction of facets visible from both `w0` and `w1`
    #[inline]
    fn visible_both(&self, w0: Vector3f, w1: Vector3f) -> Float {
        1. as Float / (1. as Float + self.lambda(w0) + self.lambda(w1))
    }

    /// given a uniform sample, sample `wh`
    fn sample_wh(&self, wo: Vector3f, u: Point2f) -> Vector3f;

    /// given `wh` and `wo`, returns the pdf
    fn pdf(&self, wo: Vector3f, wh: Vector3f) -> Float {
        self.distribution(wh) * self.visible(wo) * wo.dot(wh).abs()/normal::cos_theta(wo).abs()
    }
}

/// Transform a perceived `roughness` in $[0,1]$ into an alpha
pub fn roughness_to_alpha(roughness: Float) -> Float {
    let x = roughness.max(1e-3 as Float).ln();
    1.62142 as Float + 0.819955 as Float * x
     + 0.1734 as Float * x * x
     + 0.0171201 as Float * x * x * x 
     + 0.000640711 as Float * x * x * x * x
}

/// A Beckmann microfacet distribution
#[derive(Copy, Clone, Debug)]
pub struct Beckmann {
    /// microfacet oriented perpendicular to `x`-axis
    pub ax: Float,
    /// microfacet oriented perpendicular to `y`-axis
    pub ay: Float,
}

impl MicrofacetDistribution for Beckmann {
    fn distribution(&self, wh: Vector3f) -> Float {
        let cos2_theta = normal::cos2_theta(wh);
        let tan2_theta = normal::tan2_theta(wh);
        let cos2_phi = normal::cos2_phi(wh);
        let sin2_phi = normal::sin2_phi(wh);
        (-tan2_theta*(
            cos2_phi/(self.ax*self.ax) + sin2_phi*(self.ay*self.ay)
        )).exp() / (
            float::pi()*self.ax*self.ay*cos2_theta*cos2_theta
        )
    }

    fn lambda(&self, w: Vector3f) -> Float {
        // polynomial approximation in pbrt
        let tant = normal::tan_theta(w).abs();
        if tant.is_infinite() {
            0. as Float
        } else {
            let alpha = (
                normal::cos2_phi(w) * self.ax * self.ax
                 + normal::sin2_phi(w) * self.ay * self.ay
            ).sqrt();
            let alpha = 1. as Float / (alpha * tant);
            if alpha >= 1.6 as Float {
                0. as Float
            } else {
                (
                    1. as Float - 1.259 as Float * alpha 
                     + 0.396 as Float * alpha * alpha
                ) / (
                    3.535 as Float * alpha + 2.181 as Float * alpha * alpha
                )
            }
        }
    }

    fn sample_wh(&self, wo: Vector3f, u: Point2f) -> Vector3f {
        let mut ln = u.x.ln();
        if ln.is_infinite() { ln = 0. as Float;}
        let (tan2_theta, phi) = if relative_eq!(self.ax, self.ay) {    
            (-ln * self.ax * self.ay, u.y * float::pi() * 2. as Float)
        } else {
            let mut phi = u.y * float::pi() * 2. as Float + float::frac_pi_2();
            phi = (self.ay/self.ax*phi.tan()).atan();
            if u.y > 0.5 as Float { phi += float::pi(); }
            let sp = phi.sin();
            let cp = phi.cos();
            let ax2 = self.ax * self.ax;
            let ay2 = self.ay * self.ay;
            (-ln/(cp*cp/ax2+sp*sp/ay2), phi)
        };
        let ct = 1. as Float / (1. as Float + tan2_theta).sqrt();
        let st = (1. as Float - ct*ct).max(0. as Float).sqrt();
        let wh = Vector3f::new(st*phi.cos(), st*phi.sin(), ct);
        if wo.dot(wh) <= 0. as Float {
            -wh
        } else {
            wh
        }
    }
}

/// A Trowbridege-Reitz microfacet distribution
#[derive(Copy, Clone, Debug)]
pub struct Trowbridge {
    /// microfacet oriented perpendicular to `x`-axis
    pub ax: Float,
    /// microfacet oriented perpendicular to `y`-axis
    pub ay: Float,
}

impl MicrofacetDistribution for Trowbridge {
    fn distribution(&self, wh: Vector3f) -> Float {
        let cos2_theta = normal::cos2_theta(wh);
        let tan2_theta = normal::tan2_theta(wh);
        let cos2_phi = normal::cos2_phi(wh);
        let sin2_phi = normal::sin2_phi(wh);
        let last_term = 1. as Float + tan2_theta*(
            cos2_phi/(self.ax*self.ax) + sin2_phi*(self.ay*self.ay)
        );
        1. as Float / (
            float::pi() * self.ax * self.ay
             * cos2_theta * cos2_theta * last_term * last_term
        )
    }

    fn lambda(&self, w: Vector3f) -> Float {
        // polynomial approximation in pbrt
        let tan2t = normal::tan2_theta(w);
        if tan2t.is_infinite() {
            0. as Float
        } else {
            let alpha = (
                normal::cos2_phi(w) * self.ax * self.ax
                 + normal::sin2_phi(w) * self.ay * self.ay
            ).sqrt();
            ((1. as Float + alpha*alpha*tan2t).sqrt() - 1. as Float)*0.5 as Float
        }
    }

    fn sample_wh(&self, wo: Vector3f, u: Point2f) -> Vector3f {
        let mut ln = u.x.ln();
        if ln.is_infinite() { ln = 0. as Float;}
        let (tan2_theta, phi) = if relative_eq!(self.ax, self.ay) {    
            (-ln * self.ax * self.ay, u.y * float::pi() * 2. as Float)
        } else {
            let mut phi = u.y * float::pi() * 2. as Float + float::frac_pi_2();
            phi = (self.ay/self.ax*phi.tan()).atan();
            if u.y > 0.5 as Float { phi += float::pi(); }
            let sp = phi.sin();
            let cp = phi.cos();
            let ax2 = self.ax * self.ax;
            let ay2 = self.ay * self.ay;
            (-ln/(cp*cp/ax2+sp*sp/ay2), phi)
        };
        let ct = 1. as Float / (1. as Float + tan2_theta).sqrt();
        let st = (1. as Float - ct*ct).max(0. as Float).sqrt();
        let wh = Vector3f::new(st*phi.cos(), st*phi.sin(), ct);
        if wo.dot(wh) <= 0. as Float {
            -wh
        } else {
            wh
        }
    }
}

/// a Torrance-Sparrow bxdf
#[derive(Copy, Clone, Debug)]
pub struct TorranceSparrowBxdf<M, F> {
    /// reflectance factor
    pub reflectance: RGBSpectrumf,
    /// microfacet distribution
    pub distribution: M,
    /// fresnel factor
    pub fresnel: F
}

impl<M: MicrofacetDistribution, F: Fresnel> Bxdf for TorranceSparrowBxdf<M, F> {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_REFLECTION | BXDF_GLOSSY
    }

    fn evaluate(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf {
        let wh = (wo+wi).normalize();
        if wh.x.is_nan() || wh.y.is_nan() || wh.z.is_nan() {
            RGBSpectrumf::black()
        } else {
            self.reflectance * self.distribution.distribution(wh)
             * self.distribution.visible_both(wo, wi)
             * self.fresnel.evaluate(wo.dot(wi))
             / (4. as Float * normal::cos_theta(wo) * normal::cos_theta(wi))
        }
    }

    fn evaluate_sampled(&self, wo: Vector3f, u: Point2f) -> (RGBSpectrumf, Vector3f, Float) {
        let wh = self.distribution.sample_wh(wo, u);
        let pdf = self.distribution.pdf(wo, wh)/(4. as Float * wo.dot(wh));
        let wi = (2. as Float * wh * wo.dot(wh)- wo).normalize();
        if wo.dot(wi) <= 0. as Float {
            (RGBSpectrumf::black(), wi, pdf)
        } else {
            (self.evaluate(wo, wi), wi, pdf)
        }
    }

    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        let wh = (wo + wi).normalize();
        self.distribution.pdf(wo, wh)/(4. as Float * wo.dot(wh))
    }
}

/// A Ashikhmin-Shirley Bxdf model
#[derive(Copy, Clone, Debug)]
pub struct AshikhminShirleyBxdf<M> {
    pub diffuse: RGBSpectrumf,
    pub specular: RGBSpectrumf,
    pub distribution: M
}

impl<M> AshikhminShirleyBxdf<M> {
    #[inline]
    pub fn new(
        diffuse: RGBSpectrumf, specular: RGBSpectrumf, distribution: M
        ) -> AshikhminShirleyBxdf<M> {
        AshikhminShirleyBxdf{
            diffuse: diffuse.clamp(0. as Float, 1. as Float),
            specular: specular.clamp(0. as Float, 1. as Float),
            distribution
        }
    }
}

impl<M: MicrofacetDistribution> Bxdf for AshikhminShirleyBxdf<M> {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_REFLECTION | BXDF_GLOSSY
    }

    fn evaluate(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf {
        let wh = wo+wi;
        if relative_eq!(wh.magnitude2(), 0. as Float) {
            RGBSpectrumf::black()
        } else {
            let wh = wh.normalize();
            let term = |w| {
                1. as Float - (1. as Float - 0.5 as Float * normal::cos_theta(w).abs()).powi(5)
            };
            let diffuse = (28. as Float / (23. as Float * float::pi()))
             * self.diffuse * (RGBSpectrumf::grey_scale(1. as Float) - self.specular)
             * term(wo) * term(wi);
            let specular = self.distribution.distribution(wh)
             * schlick(wi.dot(wh), self.specular)
             / (
                 4. as Float * wi.dot(wh).abs()
                  * normal::cos_theta(wi).abs().max(normal::cos_theta(wo).abs())
             );
            diffuse + specular
        }
    }

    fn evaluate_sampled(&self, wo: Vector3f, mut u: Point2f) -> (RGBSpectrumf, Vector3f, Float) {
        let wi = if u.x < 0.5 as Float {
            u.x *= 2. as Float;
            let wh = self.distribution.sample_wh(wo, u);
            let pdf = self.distribution.pdf(wo, wh)
             /(4. as Float * wo.dot(wh))*0.5 as Float;
            let wi = (2. as Float * wh * wo.dot(wh)- wo).normalize();
            if wo.dot(wi) <= 0. as Float {
                return (RGBSpectrumf::black(), wi, pdf);
            } else {
                wi
            }
        } else {
            u.x = (1. as Float - u.x) * 2. as Float;
            let mut wi = sample::sample_cosw_hemisphere(u);
            if wi.z < 0.0 as Float {wi.z = -wi.z;}
            wi
        };
        (self.evaluate(wo, wi), wi, self.pdf(wo, wi))
    }

    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        if wo.dot(wi) < 0. as Float { return 0. as Float; }
        let wh = (wo + wi).normalize();
        0.5 as Float * (
            self.distribution.pdf(wo, wh)/(4. as Float * wo.dot(wh))
             + normal::cos_theta(wi).abs() * float::frac_1_pi()
        )
    }
}

#[inline]
fn schlick(cost: Float, s: RGBSpectrumf) -> RGBSpectrumf {
    s + (1. as Float - cost).powi(5) * (RGBSpectrumf::grey_scale(1. as Float) - s)
}
