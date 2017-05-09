// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! defines some microfacet theory based distributions and some bxdfs
//! derived from them.

use super::*;
use super::fresnel::*;

/// A microfacet distribution description
pub trait MicrofacetDistribution {
    /// Given the macro surface normal `wh`, returns the differential
    /// area of microfacets from that angle
    ///
    /// A physically plausible microfacet distribution $D$ should be
    /// normalized, s.t. $\integral_{H^2(n)}D(\omega_h)cos\theta_h d\omega_h=1$
    fn distribution(&self, wh: Vector3f) -> Float;

    /// Given a direction `w`, return masked prjected surface area
    /// per visible projected surface area along that direction,
    /// assuming this value is independent from `wh`.
    fn lambda(&self, w: Vector3f) -> Float;

    /// fraction of facets visible from `w`.
    /// The masking-shadowing function $G_l$
    /// This interface assumes independence from `wh`
    #[inline]
    fn visible(&self, w: Vector3f) -> Float {
        1. as Float / (1. as Float + self.lambda(w))
    }

    /// fraction of facets visible from both `w0` and `w1`
    ///
    /// This interface assumes independence from `wh`
    #[inline]
    fn visible_both(&self, w0: Vector3f, w1: Vector3f) -> Float {
        1. as Float / (1. as Float + self.lambda(w0) + self.lambda(w1))
    }

    /// given a uniform sample, return a sampled macro normal `wh`
    fn sample_wh(&self, wo: Vector3f, u: Point2f) -> Vector3f;

    /// given `wo` and a sampled `wh`, returns the pdf of this sample
    fn pdf(&self, wo: Vector3f, wh: Vector3f) -> Float {
        self.distribution(wh) * self.visible(wo) * wo.dot(wh).abs()
         /normal::cos_theta(wo).abs()
    }
}

/// Transform a perceived `roughness` in $[0,1]$ into an alpha value
/// which can be used in the `Beckmann` and `Trowbridge` distributions
pub fn roughness_to_alpha(roughness: Float) -> Float {
    let x = roughness.max(1e-3 as Float).ln();
    1.62142 as Float + 0.819955 as Float * x
     + 0.1734 as Float * x * x
     + 0.0171201 as Float * x * x * x 
     + 0.000640711 as Float * x * x * x * x
}

/// A Beckmann microfacet distribution
///
/// With microfacet distribution specified as
/// $D(\omega_h) = \frac{
///    \exp{-tan^2\theta_h(cos^2\phi_h/\alpha_x^2+sin^2\phi_h/\alpha_y^2)}
/// }{
///    \phi*\alpha_x*\alpha_y*cos^4\theta_h
/// }$
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
            cos2_phi/(self.ax*self.ax) + sin2_phi/(self.ay*self.ay)
        )).exp() / (
            float::pi()*self.ax*self.ay*cos2_theta*cos2_theta
        )
    }

    #[inline]
    fn lambda(&self, w: Vector3f) -> Float {
        // polynomial approximation of
        // $\Lambda(\omega)=\frac{
        // erf(a) - 1 + \frac{\exp{-a^2}}{a\sqrt{\pi}}   
        // }{2}$, where $a=1/(\alpha tan\theta)$, and
        // $erf(x)=2/\sqrt{\pi}\integral_0^x\exp{-t^2}dt$
        // this approximation was first introduced by pbrt
        let tant = normal::tan_theta(w).abs();
        if tant.is_infinite() || tant.is_nan() {
            0. as Float
        } else {
            let alpha = (
                normal::cos2_phi(w) * self.ax * self.ax
                 + normal::sin2_phi(w) * self.ay * self.ay
            ).sqrt();
            let a = 1. as Float / (alpha * tant);
            if a >= 1.6 as Float {
                0. as Float
            } else {
                (
                    1. as Float - 1.259 as Float * a
                    + 0.396 as Float * a * a
                ) / (
                    3.535 as Float * a + 2.181 as Float * a * a
                )
            }
        }
    }

    #[inline]
    fn sample_wh(&self, wo: Vector3f, u: Point2f) -> Vector3f {
        _sample_wh_beckmann(wo, u, self.ax, self.ay)
    }
}

/// A Trowbridege-Reitz microfacet distribution
///
/// with microfacet distribution function specified as
/// $D(\omega_h)=\frac{1}{\pi\alpha_x\alpha_ycos^4\theta_h(
///    1+tan^2\theta_h(cos^2\phi_h/\alpha_x^2+sin^2\phi_h/\alpha_y^2)
/// )^2}
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
        if tan2_theta.is_infinite() { return 0. as Float; }
        let cos2_phi = normal::cos2_phi(wh);
        let sin2_phi = normal::sin2_phi(wh);
        let last_term = 1. as Float + tan2_theta*(
            cos2_phi/(self.ax*self.ax) + sin2_phi/(self.ay*self.ay)
        );
        1. as Float / (
            float::pi() * self.ax * self.ay
             * cos2_theta * cos2_theta * last_term * last_term
        )
    }

    #[inline]
    fn lambda(&self, w: Vector3f) -> Float {
        let tabs = normal::tan_theta(w).abs();
        if tabs.is_infinite() { return 0. as Float; }
        let alpha = (
            normal::cos2_phi(w)*self.ax*self.ax
             + normal::sin2_phi(w)*self.ay*self.ay
        ).sqrt();
        let term = alpha * tabs;
        (-1. as Float + (1. as Float + term*term).sqrt()) * 0.5 as Float
    }

    #[inline]
    fn sample_wh(&self, wo: Vector3f, u: Point2f) -> Vector3f {
        let won = if wo.z < 0. as Float { -wo } else { wo };
        let wh = _sample_wh_trowbridge(won, u, self.ax, self.ay);
        let ret = if wo.z < 0. as Float { -wh } else { wh };
        ret
    }
}

fn _sample_wh_beckmann(wo: Vector3f, u: Point2f, ax: Float, ay: Float) -> Vector3f {
    // let mut ln = u.x.ln();
    // if ln.is_infinite() { ln = 0. as Float;}
    // let (tan2_theta, phi) = if relative_eq!(ax, ay) {    
    //     (-ln * ax * ay, u.y * float::pi() * 2. as Float)
    // } else {
    //     let mut phi = u.y * float::pi() * 2. as Float + float::frac_pi_2();
    //     phi = (ay/ax*phi.tan()).atan();
    //     if u.y > 0.5 as Float { phi += float::pi(); }
    //     let sp = phi.sin();
    //     let cp = phi.cos();
    //     let ax2 = ax * ax;
    //     let ay2 = ay * ay;
    //     (-ln/(cp*cp/ax2+sp*sp/ay2), phi)
    // };
    // let ct = 1. as Float / (1. as Float + tan2_theta).sqrt();
    // let st = (1. as Float - ct*ct).max(0. as Float).sqrt();
    // let wh = Vector3f::new(st*phi.cos(), st*phi.sin(), ct);
    // if wo.dot(wh) <= 0. as Float {
    //     -wh
    // } else {
    //     wh
    // }
    let wo_stretched = Vector3f::new(ax*wo.x, ay*wo.y, wo.z).normalize();
    let cos_theta = normal::cos_theta(wo_stretched).abs();
    let (mut sx, mut sy) = if cos_theta > 0.9999 as Float {
        let r = (-u.x.ln()).sqrt();
        let phi = 2.0 as Float * u.y * float::pi();
        (r*phi.cos(), r*phi.sin())
    } else {
        let sin_theta = (1.0 as Float - cos_theta*cos_theta).max(0. as Float).sqrt();
        let tan_theta = sin_theta/cos_theta;
        let cot_theta = cos_theta/sin_theta;
        let mut a = -1.0 as Float;
        let mut c = erf(cot_theta);
        let ux = u.x.max(1e-6 as Float);
        let theta = cos_theta.acos();
        let fit = 1.0 as Float + theta * (
            -0.876 as Float + theta * (
                0.4265 as Float - 0.0594 as Float * theta
            )
        );
        let mut b = c - (1.0 as Float + c) * (1. as Float - ux).powf(fit);
        let sqrt_pi_inv = 1. as Float / float::pi().sqrt();
        let norm = 1.0 as Float / (
            1.0 as Float + c + sqrt_pi_inv * tan_theta * (-cot_theta*cot_theta).exp()
        );
        for _it in 1..10 {
            if b<a || b>c { b = 0.5 as Float * (a+c); }
            let inv = erf_inv(b);
            let value = norm * (
                1.0 as Float + b + sqrt_pi_inv * tan_theta * (-inv*inv).exp()
            ) - ux;
            
            if value.abs() < 1e-5 as Float { break; }

            let derivation = norm * (1.0 as Float - inv*tan_theta);
            
            if value > 0. as Float {
                c = b;
            } else {
                a = b;
            }
            b -= value / derivation;
        }
        (erf_inv(b), erf_inv(
            2.0 as Float * (u.y).max(1e-6 as Float) - 1.0 as Float
        ))
    };
    let cos_phi = normal::cos_phi(wo_stretched);
    let sin_phi = normal::sin_phi(wo_stretched);
    let rotation_tmp =  cos_phi* sx - sin_phi*sy;
    sy = sin_phi*sx + cos_phi*sy;
    sx = rotation_tmp;
    sx *= ax;
    sy *= ay;
    Vector3f::new(-sx, -sy, 1. as Float).normalize() * wo.z.signum()
}

fn _sample_wh_trowbridge(wo: Vector3f, u: Point2f, ax: Float, ay: Float) -> Vector3f {
    let wo_stretched = Vector3f::new(ax*wo.x, ay*wo.y, wo.z).normalize();
    let cos_theta = normal::cos_theta(wo_stretched).abs();
    let (mut sx, mut sy) = if cos_theta > 0.9999 as Float {
        let r = (u.x/(1.0 as Float - u.x)).sqrt();
        let phi = 2.0 as Float * u.y * float::pi();
        (r*phi.cos(), r*phi.sin())
    } else {
        let sin_theta = (1.0 as Float - cos_theta*cos_theta).max(0. as Float).sqrt();
        let tan_theta = sin_theta/cos_theta;
        let cot_theta = cos_theta/sin_theta;
        let g1 = 2.0 as Float / (1.0 as Float + (
            1.0 as Float + 1.0 as Float / (cot_theta*cot_theta)
        ).sqrt());
        let a = 2.0 as Float * u.y / g1 - 1.0 as Float;
        let tmp = (1.0 as Float / (a*a - 1.0 as Float)).min(1e10 as Float);
        let d = (tan_theta*tan_theta*tmp*tmp - (
            a * a - tan_theta * tan_theta
        )*tmp).max(0. as Float).sqrt();
        let sx1 = tan_theta*tmp - d;
        let sx2 = tan_theta*tmp + d;

        let sx = if a < 0. as Float || sx2 > cot_theta {
            sx1
        } else {
            sx2
        };
        
        let (s, uy) = if u.y > 0.5 as Float {
            (1. as Float, 2. as Float * (u.y - 0.5 as Float))
        } else {
            (-1. as Float, 2. as Float * (0.5 as Float - u.y))
        };
        let z = (uy*(uy*(
            uy * 0.27385 as Float - 0.73369 as Float
        ) + 0.46341 as Float)) / (uy*(uy*(
            uy * 0.093073 as Float + 0.309420 as Float
        ) - 1.000000 as Float) + 0.597999 as Float);
        let sy = s * z * (1. as Float + sx*sx);
        (sx, sy)
    };
    let cos_phi = normal::cos_phi(wo_stretched);
    let sin_phi = normal::sin_phi(wo_stretched);
    let rotation_tmp =  cos_phi* sx - sin_phi*sy;
    sy = sin_phi*sx + cos_phi*sy;
    sx = rotation_tmp;
    sx *= ax;
    sy *= ay;
    Vector3f::new(-sx, -sy, 1. as Float).normalize()
}

/// polynomial approximation of $Erf^{-1}(x)$, introduced by pbrt
#[inline]
fn erf_inv(x: Float) -> Float {
    let x = x.max(-0.99999 as Float).min(0.99999 as Float);
    let mut w = -((1.0 as Float - x) * (1.0 as Float + x)).ln();
    let mut p;
    if w < 5.0 as Float {
        w = w - 2.5 as Float;
        p = 2.81022636e-08 as Float;
        p = 3.43273939e-07 as Float + p * w;
        p = -3.5233877e-06 as Float + p * w;
        p = -4.39150654e-06 as Float + p * w;
        p = 0.00021858087 as Float + p * w;
        p = -0.00125372503 as Float + p * w;
        p = -0.00417768164 as Float + p * w;
        p = 0.246640727 as Float + p * w;
        p = 1.50140941 as Float + p * w;
    } else {
        w = w.sqrt() - 3.0 as Float;
        p = -0.000200214257 as Float;
        p = 0.000100950558 as Float + p * w;
        p = 0.00134934322 as Float + p * w;
        p = -0.00367342844 as Float + p * w;
        p = 0.00573950773 as Float + p * w;
        p = -0.0076224613 as Float + p * w;
        p = 0.00943887047 as Float + p * w;
        p = 1.00167406 as Float + p * w;
        p = 2.83297682 as Float + p * w;
    }
    p * x
}

/// polynomial approximation of $Erf(x)$, introduced by pbrt
#[inline]
fn erf(x: Float) -> Float {
    // constants
    const A1: Float = 0.254829592 as Float;
    const A2: Float = -0.28449673 as Float;
    const A3: Float = 1.421413741 as Float;
    const A4: Float = -1.453152027 as Float;
    const A5: Float = 1.061405429 as Float;
    const P: Float = 0.3275911 as Float;

    // Save the sign of x
    let sign = x.signum();
    let x = x*sign;

    // A&S formula 7.1.26
    let t = 1.0 as Float / (1.0 as Float + P * x);
    let y =
        1.0 as Float -
        (((((A5 * t + A4) * t) + A3) * t + A2) * t + A1) * t * (-x * x).exp();

    sign * y
}

/// a Torrance-Sparrow bxdf, with bxdf given by
/// $f(\omega_o, \omega_i)=\frac{
///    D(\omege_h)G(\omega_o,\omega_i)F_r(\omega_o)
/// }{4cos\theta_o cos\theta_i}
#[derive(Copy, Clone, Debug)]
pub struct TorranceSparrowRBxdf<M, F> {
    /// reflectance factor
    pub reflectance: RGBSpectrumf,
    /// microfacet distribution for `D` and `G`
    pub distribution: M,
    /// fresnel factor `Fr`
    pub fresnel: F
}

impl<M, F> TorranceSparrowRBxdf<M, F> {
    #[inline]
    pub fn new(reflectance: RGBSpectrumf, distribution: M, fresnel: F) -> Self {
        TorranceSparrowRBxdf{
            reflectance, distribution, fresnel
        }
    }
}

impl<M: MicrofacetDistribution, F: Fresnel> Bxdf for TorranceSparrowRBxdf<M, F> {
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
             * self.fresnel.evaluate(wi.dot(wh))
             / (4. as Float * wo.z.abs() * wi.z.abs())
        }
    }

    fn evaluate_sampled(&self, wo: Vector3f, u: Point2f
    ) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        let wh = self.distribution.sample_wh(wo, u);
        let pdf = self.distribution.pdf(wo, wh)/(4. as Float * wo.dot(wh));
        let wi = (2. as Float * wh * wo.dot(wh)- wo).normalize();
        if wo.z * wi.z <= 0. as Float {
            trace!("not samehemisphere for TSR, blacking");
            (RGBSpectrumf::black(), wi, pdf, self.kind())
        } else {
            let ret = (self.evaluate(wo, wi), wi, pdf, self.kind());
            trace!("samehemisphere for TSR, {:?}", ret);
            ret
        }
    }

    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        if wo.z *wi.z <= 0. as Float { return 0. as Float; }
        let wh = (wo + wi).normalize();
        let pdf = self.distribution.pdf(wo, wh)/(4. as Float * wo.dot(wh));
        // pdf.max(0. as Float)
        pdf
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TorranceSparrowTBxdf<M> {
    /// transmittance factor
    pub transmittance: RGBSpectrumf,
    /// fresnel factor `Fr`
    pub fresnel: Dielectric,
    /// microfacet distribution for `D` and `G`
    pub distribution: M,
}

impl<M> TorranceSparrowTBxdf<M> {
    #[inline]
    pub fn new(
        transmittance: RGBSpectrumf, distribution: M, eta0: Float, eta1: Float
    ) -> Self {
        TorranceSparrowTBxdf{
            transmittance, distribution, fresnel: Dielectric::new(eta0, eta1)
        }
    }
}

impl<M: MicrofacetDistribution> Bxdf for TorranceSparrowTBxdf<M> {
    #[inline]
    fn kind(&self) -> BxdfType {
        BXDF_TRANSMISSION | BXDF_GLOSSY
    }

    fn evaluate(&self, wo: Vector3f, wi: Vector3f) -> RGBSpectrumf {
        // reject reflectance
        if wo.z * wi.z > 0. as Float { return RGBSpectrumf::black(); }
        let eta = if wo.z > 0. as Float {
            self.fresnel.eta1 / self.fresnel.eta0
        } else {
            self.fresnel.eta0 / self.fresnel.eta1
        };
        let mut wh = (wo+wi*eta).normalize();
        if wh.x.is_infinite() || wh.y.is_infinite() || wh.z.is_infinite()
         || wh.x.is_nan() || wh.y.is_nan() || wh.z.is_nan() {
            trace!("handling eta==1");
            return RGBSpectrumf::grey_scale(1. as Float);
        }
        if wh.z < 0. as Float { wh = -wh; }
        let cosoh = wo.dot(wh);
        let f = self.fresnel.evaluate(cosoh);
        let cosih = wi.dot(wh);
        let sqrt_denom = cosoh + eta * cosih;

        let ret = self.transmittance * self.distribution.distribution(wh)
            * self.distribution.visible_both(wo, wi)
            * (RGBSpectrumf::grey_scale(1. as Float) - f)
            * cosih.abs() * cosoh.abs()//  * 2.5 as Float
            / (normal::cos_theta(wo).abs() * normal::cos_theta(wi).abs()*sqrt_denom*sqrt_denom);
        if ret.r() < 0. as Float {
            warn!("negative f:");
            warn!("\tdis:{}, v:{}, cih: {}, coh:{}, ",self.distribution.distribution(wh), self.distribution.visible_both(wo, wi), cosih.abs(), cosoh.abs());
            warn!("\tctwo:{}, ctwi:{}, denom:{}", wo.z, wi.z, sqrt_denom);
        }
        ret
    }

    fn evaluate_sampled(&self, wo: Vector3f, u: Point2f
    ) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        let wh = self.distribution.sample_wh(wo, u);
        let eta = if wo.z > 0. as Float {
            self.fresnel.eta0 / self.fresnel.eta1
        } else {
            self.fresnel.eta1 / self.fresnel.eta0
        };
        if let Some(wi) = normal::refract(wo, wh, eta) {
            let pdf = self.pdf(wo, wi);
            let f = self.evaluate(wo, wi);
            let ret = (f, wi, pdf, self.kind());
            trace!("refraction found {:?}", ret);
            ret
        } else {
            trace!("total reflection, no refraction");
            (RGBSpectrumf::black(), Vector3f::zero(), 0. as Float, self.kind())
        }
    }

    #[inline]
    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        if wo.z * wi.z > 0. as Float { return 0. as Float; }
        let eta = if wo.z > 0. as Float {
            self.fresnel.eta1 / self.fresnel.eta0
        } else {
            self.fresnel.eta0 / self.fresnel.eta1
        };
        let wh = (wo + wi*eta).normalize();
        if wh.x.is_infinite() || wh.y.is_infinite() || wh.z.is_infinite()
         || wh.x.is_nan() || wh.y.is_nan() || wh.z.is_nan() {
            trace!("handling eta==1");
            return 1. as Float;
        }
        let sqrt_denom = wo.dot(wh) + eta*wi.dot(wh);
        let dhdi = eta*eta*wi.dot(wh).abs() / (sqrt_denom*sqrt_denom);
        trace!("wo: {:?}, wi: {:?}, wh: {:?}, sqrtdenom: {}", wo, wi, wh, sqrt_denom);
        let pdf = self.distribution.pdf(wo, wh) * dhdi;
        // pdf.max(0. as Float)
        pdf
    }
}

/// A Ashikhmin-Shirley Bxdf, modelling a glossy specular
/// surface above a diffuse one
#[derive(Copy, Clone, Debug)]
pub struct AshikhminShirleyBxdf<M> {
    /// The diffuse term
    pub diffuse: RGBSpectrumf,
    /// The specular term, with fresnel term given by
    /// $F_r(cos\theta)=R+(1-R)(1-cos\theta)^5$
    pub specular: RGBSpectrumf,
    /// distribution for the specular term
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
                1. as Float - (
                    1. as Float - 0.5 as Float * normal::cos_theta(w).abs()
                ).powi(5)
            };
            let diffuse = (28. as Float / (23. as Float * float::pi()))
             * self.diffuse * (RGBSpectrumf::grey_scale(1. as Float) - self.specular)
             * term(wo) * term(wi);
            let specular = self.distribution.distribution(wh)
             * schlick_fresnel(wi.dot(wh), self.specular)
             / (
                 4. as Float * wi.dot(wh).abs()
                  * normal::cos_theta(wi).abs().max(normal::cos_theta(wo).abs())
             );
            diffuse + specular
        }
    }

    fn evaluate_sampled(&self, wo: Vector3f, mut u: Point2f
    ) -> (RGBSpectrumf, Vector3f, Float, BxdfType) {
        // sample according to specular distribution
        // or according to the diffuse term, both with
        // probability of 1/2
        let wi = if u.x < 0.5 as Float {
            u.x *= 2. as Float;
            let wh = self.distribution.sample_wh(wo, u);
            let wi = (2. as Float * wh * wo.dot(wh)- wo).normalize();
            if wo.z * wi.z <= 0. as Float {
                return (RGBSpectrumf::black(), wi, self.pdf(wo, wi), self.kind());
            } else {
                wi
            }
        } else {
            u.x = (1. as Float - u.x) * 2. as Float;
            let mut wi = sample::sample_cosw_hemisphere(u);
            if wi.z < 0.0 as Float {wi.z = -wi.z;}
            wi
        };
        (self.evaluate(wo, wi), wi, self.pdf(wo, wi), self.kind())
    }

    fn pdf(&self, wo: Vector3f, wi: Vector3f) -> Float {
        if wo.z * wi.z < 0. as Float { return 0. as Float; }
        let wh = (wo + wi).normalize();
        let pdf = 0.5 as Float * (
            self.distribution.pdf(wo, wh)/(4. as Float * wo.dot(wh))
             + normal::cos_theta(wi).abs() * float::frac_1_pi()
        );
        // pdf.max(0. as Float)
        pdf
    }
}

#[inline]
fn schlick_fresnel(cost: Float, s: RGBSpectrumf) -> RGBSpectrumf {
    s + (1. as Float - cost).powi(5) * (RGBSpectrumf::grey_scale(1. as Float) - s)
}
