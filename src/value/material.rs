use serde::{Deserialize, Serialize};
use crate::value::color::Color;
use crate::hittable::HitRecord;
use crate::value::ray::Ray;
use crate::utils::rtweekend::random_double;
use crate::value::vec3::Vec3;

#[derive(Debug, Eq, PartialEq)]
pub enum MaterialType {
    Lambertian,
    Metal,
    Dielectric,
}

#[typetag::serde(tag = "type")]
pub trait Material {
    fn scatter(&self, r_in: &Ray, hit_record: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool;

    fn get_type(&self) -> MaterialType;
}

#[derive(Serialize, Deserialize)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

#[typetag::serde]
impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let mut scatter_direction = rec.normal + Vec3::random_unit();

        // Catch degenerate scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo;
        true
    }

    fn get_type(&self) -> MaterialType {
        MaterialType::Lambertian
    }
}

#[derive(Serialize, Deserialize)]
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz: if fuzz < 1.0 { fuzz } else { 1.0 } }
    }
}

#[typetag::serde]
impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let reflected = Vec3::reflect(r_in.direction(), &rec.normal);
        let reflected = reflected.unit() + (self.fuzz * Vec3::random_unit());
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;

        scattered.direction().dot(&rec.normal) > 0.0
    }

    fn get_type(&self) -> MaterialType {
        MaterialType::Metal
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }
}

#[typetag::serde]
impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face { 1.0 / self.refraction_index } else { self.refraction_index };

        let unit_direction = Vec3::unit(r_in.direction());
        let cos_theta = f64::min((-unit_direction).dot(&rec.normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let direction =
            if ri * sin_theta > 1.0 || reflectance(cos_theta, ri) > random_double() {
                // Cannot refract, must reflect
                Vec3::reflect(&unit_direction, &rec.normal)
            } else {
                // Can refract
                Vec3::refract(&unit_direction, &rec.normal, ri)
            };

        *scattered = Ray::new(rec.p, direction);
        true
    }

    fn get_type(&self) -> MaterialType {
        MaterialType::Dielectric
    }
}

fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
    let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
    
