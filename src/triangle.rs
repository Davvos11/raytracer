use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Point3;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use crate::acceleration::aabb::AABB;

#[derive(Serialize, Deserialize)]
pub struct Triangle {
    v0: Point3,
    v1: Point3,
    v2: Point3,
    mat: Rc<dyn Material>,
}

impl Triangle {
    pub fn new(v0: Point3, v1: Point3, v2: Point3, mat: Rc<dyn Material>) -> Self {
        Self { v0, v1, v2, mat }
    }

    pub fn a(&self) -> Point3 { self.v0 }
    pub fn b(&self) -> Point3 { self.v1 }
    pub fn c(&self) -> Point3 { self.v2 }
}

#[typetag::serde]
impl Hittable for Triangle {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // Calculate the normal by the cross product of AB and AC
        let v0v1 = self.v1 - self.v0; // AB
        let v0v2 = self.v2 - self.v0; // AC
        let n = v0v1.cross(&v0v2);

        // Check if the ray and plane are parallel
        let n_dot_dir = n.dot(r.direction());
        if !ray_t.surrounds(n_dot_dir) {
            return false;
        }

        // Get the distance from the origin to the plane
        let d = -n.dot(&self.v0);
        // Get the distance along the ray
        rec.t = -(n.dot(r.origin()) + d) / n_dot_dir;

        // The triangle is not visible if it is behind the camera
        if rec.t < 0.0 {
            return false;
        }
        // Get the intersection point
        rec.p = r.at(rec.t);
        // Check if the plane intersection is inside the triangle
        // (inside-outside test)
        let v0p = rec.p - self.v0;
        if n.dot(&v0v1.cross(&v0p)) <= 0.0 {
            return false;
        }
        let v1v2 = self.v2 - self.v1;
        let v1p = rec.p - self.v1;
        if n.dot(&v1v2.cross(&v1p)) <= 0.0 {
            return false;
        }
        let v2v0 = self.v0 - self.v2;
        let v2p = rec.p - self.v2;
        if n.dot(&v2v0.cross(&v2p)) <= 0.0 {
            return false;
        }

        rec.set_face_normal(r, n);
        rec.mat = Some(Rc::clone(&self.mat));

        true
    }

    fn to_aabb(&self) -> AABB {
        let x_min = self.a().x().min(self.b().x()).min(self.c().x());
        let y_min = self.a().y().min(self.b().y()).min(self.c().y());
        let z_min = self.a().z().min(self.b().z()).min(self.c().z());
        let x_max = self.a().x().max(self.b().x()).max(self.c().x());
        let y_max = self.a().y().max(self.b().y()).max(self.c().y());
        let z_max = self.a().z().max(self.b().z()).max(self.c().z());

        AABB::new(Point3::new(x_min, y_min, z_min), Point3::new(x_max, y_max, z_max))
    }
}