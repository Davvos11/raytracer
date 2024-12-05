use std::ops::Add;
use crate::hittable::HitRecord;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::{AlgorithmOptions, Options};
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
    /// With an extension to return the intersection point (on the ray)
    pub fn hit(&self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, options: &Options) -> Option<f64> {
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
            if ray_t.max <= ray_t.min { return None; }
        }

        let hit_point = *ray.origin() + ray_t.min * *ray.direction();
        if options.draw_boxes && self.is_edge(hit_point) {
            rec.hits_aabb_edge = true;
        }
        Some(ray_t.min)
    }

    fn is_edge(&self, point: Point3) -> bool {
        (0..3)
            .filter(|&axis| {
                point[axis] <= self.min[axis] + 0.01 && point[axis] >= self.min[axis] - 0.01 
                    || point[axis] <= self.max[axis] + 0.01 && point[axis] >= self.max[axis] - 0.01 
            })
            .count() >= 2
    }
    
    pub fn contains(&self, point: Point3) -> bool {
        (0..3).all(|axis| self.min[axis] <= point[axis] && self.max[axis] >= point[axis])
    }

    // TODO maybe the 2.0 can be removed since it is only a comparator
    pub fn surface_area(&self) -> f64 {
        2.0 * (
            (self.max.x() - self.min.x()) * (self.max.y() - self.min.y())
                + (self.max.x() - self.min.x()) * (self.max.z() - self.min.z())
                + (self.max.y() - self.min.y()) * (self.max.z() - self.min.z())
        )
    }
    
    /// Checks if self is inside other, https://math.stackexchange.com/questions/1472049/check-if-a-point-is-inside-a-rectangular-shaped-area-3d
    /// TODO: probably won't work properly
    /// TODO: trying smth else
    pub fn inside(&self, other: AABB) -> bool {
        /*let p1 = other.min;
        let p2 = Point3::new(p1.x(), p1.y(), other.max.z());
        let p4 = Point3::new(other.max.x(), p1.y(), p1.z());
        let p5 = Point3::new(p1.x(), other.max.y(), p1.z());
        let u = p1 - p2;
        let v = p1 - p4;
        let w = p1 - p5;
        for vec in [self.min, self.max] {
            let dot1 = u.dot(&vec);
            let dot2 = v.dot(&vec);
            let dot3 = w.dot(&vec);
            if u.dot(&p1) <= dot1 && dot1 <= u.dot(&p2)
                && v.dot(&p1) <= dot2 && dot2 <= v.dot(&p4)
                && w.dot(&p1) <= dot3 && dot3 <= w.dot(&p5) {
                return true;
            }
        }*/

        self.max.x() >= other.min.x() && self.min.x() <= other.max.x()
            && self.max.y() >= other.min.y() && self.min.y() <= other.max.y()
            && self.max.z() >= other.min.z() && self.min.z() <= other.max.z()
    }
    
    pub fn point_inside(&self, point: Point3) -> bool {
        self.min.x() <= point.x() && self.max.x() >= point.x()
            && self.min.y() <= point.y() && self.max.y() >= point.y()
            && self.min.z() <= point.z() && self.max.z() >= point.z()
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

