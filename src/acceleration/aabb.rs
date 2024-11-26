use std::ops::Add;
use crate::ray::Ray;
use crate::vec3::Point3;

#[derive(Default, Clone)]
pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

impl AABB {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    pub fn inside(&self, ray: &Ray) -> bool {
        todo!()
    }
}

impl Add for AABB {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl Add for &AABB {
    type Output = AABB;

    fn add(self, rhs: Self) -> Self::Output {
        let x_min = f64::min(self.min.x(), rhs.min.x());
        let y_min = f64::min(self.min.y(), rhs.min.y());
        let z_min = f64::min(self.min.z(), rhs.min.z());
        let x_max = f64::max(self.max.x(), rhs.max.x());
        let y_max = f64::max(self.max.y(), rhs.max.y());
        let z_max = f64::max(self.max.z(), rhs.max.z());

        AABB::new(Point3::new(x_min, y_min, z_min), Point3::new(x_max, y_max, z_max))
    }
}

