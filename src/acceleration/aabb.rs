use std::ops::Add;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::Point3;

/// BVH and AABB from course slides
#[derive(Default, Clone)]
pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

impl AABB {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    fn axis_interval(&self, axis: u32) -> Interval {
        Interval::new(self.min[axis], self.max[axis])
    }

    /// From https://raytracing.github.io/books/RayTracingTheNextWeek.html#boundingvolumehierarchies/rayintersectionwithanaabb
    pub fn hit(&self, ray: &Ray, ray_t: Interval) -> bool {
        // Make a copy for local use
        let mut ray_t = ray_t;
        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let ad_inverse = 1.0 / ray.direction()[axis];

            let t0 = (ax.min - ray.origin()[axis]) * ad_inverse;
            let t1 = (ax.max - ray.origin()[axis]) * ad_inverse;

            if t0 < t1 {
                if t0 > ray_t.min { ray_t.min = t0; }
                if t1 < ray_t.max { ray_t.max = t1; }
            } else {
                if t1 > ray_t.min { ray_t.min = t1; }
                if t0 < ray_t.max { ray_t.max = t0; }
            }

            // If the interval is now empty, we missed the AABB on this axis
            if ray_t.max <= ray_t.min { return false; }
        }

        true
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

