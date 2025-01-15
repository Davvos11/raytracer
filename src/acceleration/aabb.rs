use crate::acceleration::grid::Grid;
use crate::hittable::HitRecord;
use crate::value::interval::Interval;
use crate::value::ray::Ray;
use crate::utils::rtweekend::Options;
use crate::value::vec3::Point3;
use std::ops::Add;

#[allow(clippy::upper_case_acronyms)]
/// BVH and AABB from course slides
#[derive(Default, Clone, Debug)]
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
    pub fn hit(&self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, options: &Options) -> Option<(Interval, Exit)> {
        // Make a copy for local use
        let mut ray_t = ray_t;
        let mut exit = Exit::default();
        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let ad_inverse = 1.0 / ray.direction()[axis];

            let t0 = (ax.min - ray.origin()[axis]) * ad_inverse;
            let t1 = (ax.max - ray.origin()[axis]) * ad_inverse;

            if t0 < t1 {
                if t0 > ray_t.min { ray_t.min = t0; }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                    exit.set(axis, 1)
                }
            } else {
                if t1 > ray_t.min { ray_t.min = t1; }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                    exit.set(axis, -1)
                }
            }

            // If the interval is now empty, we missed the AABB on this axis
            if ray_t.max < ray_t.min { return None; }
        }

        let hit_point = *ray.origin() + ray_t.min * *ray.direction();
        if options.draw_boxes && self.is_edge(hit_point) {
            rec.hits_aabb_edge = true;
        }
        Some((ray_t, exit))
    }

    // Duplicate of the function above to get the exit point as well
    pub fn enter_and_exit(&self, ray: &Ray, ray_t: Interval) -> Option<(f64, f64)> {
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

        Some((ray_t.min, ray_t.max))
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

    /// Checks if self is inside other
    pub fn inside(&self, other: &AABB) -> bool {
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

#[derive(Default, Debug)]
pub struct Exit {
    pub axis: u32,
    pub direction: i8,
}

impl Exit {
    pub fn set(&mut self, axis: u32, direction: i8) {
        self.axis = axis;
        self.direction = direction;
    }
    
    pub fn get_step(&self, grid: &Grid) -> Point3 {
        let mut result = Point3::default();
        result[self.axis] = self.direction as f64 * grid.box_size[self.axis];
        result
    }
}