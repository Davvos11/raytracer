use std::rc::Rc;
use crate::acceleration::aabb::AABB;
use crate::value::data::Data;
use crate::value::interval::Interval;
use crate::value::material::{Material, MaterialType};
use crate::value::ray::Ray;
use crate::value::vec3::{Point3, Vec3};

pub mod sphere;
pub mod hittable_list;
pub mod triangle;

#[derive(Default, Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Option<Rc<dyn Material>>,
    pub t: f64,
    pub front_face: bool,
    pub hits_aabb_edge: bool,
}

impl HitRecord {
    /// Sets the hit record normal vector.
    /// Note: the parameter `outward_normal` is assumed to have unit length.
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = r.direction().dot(&outward_normal) < 0.0;
        self.normal = if self.front_face { outward_normal } else { -outward_normal };
    }
}

#[typetag::serde(tag = "type")]
pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: Interval, hit_record: &mut HitRecord, data: &mut Data) -> bool;

    fn to_aabb(&self) -> AABB;

    fn centroid(&self) -> Point3;
    
    fn surface_area(&self) -> f64;
    
    fn material_type(&self) -> Option<MaterialType> {
        None
    }
}
