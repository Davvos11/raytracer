use crate::acceleration::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Point3;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Rc<dyn Material>) -> Self {
        Self { center, radius: f64::max(0.0, radius), mat }
    }
}

#[typetag::serde]
impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let oc = self.center - *r.origin();
        let a = r.direction().length_squared();
        let h = r.direction().dot(&oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrt_discriminant = discriminant.sqrt();

        // Find nearest root within the acceptable range
        let mut root = (h - sqrt_discriminant) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrt_discriminant) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);
        rec.mat = Some(Rc::clone(&self.mat));

        true
    }

    fn to_aabb(&self) -> AABB {
        let x_min = self.center.x() - self.radius;
        let y_min = self.center.y() - self.radius;
        let z_min = self.center.z() - self.radius;
        let x_max = self.center.x() + self.radius;
        let y_max = self.center.y() + self.radius;
        let z_max = self.center.z() + self.radius;

        AABB::new(Point3::new(x_min, y_min, z_min), Point3::new(x_max, y_max, z_max))
    }

    fn centroid(&self) -> Point3 {
        self.center
    }
}
