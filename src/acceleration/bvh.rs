use crate::vec3::Point3;
use std::ops::Add;


#[derive(Default)]
pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

impl AABB {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }
}

impl Add for AABB {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let x_min = f64::min(self.min.x(), rhs.min.x());
        let y_min = f64::min(self.min.y(), rhs.min.y());
        let z_min = f64::min(self.min.z(), rhs.min.z());
        let x_max = f64::max(self.max.x(), rhs.max.x());
        let y_max = f64::max(self.max.y(), rhs.max.y());
        let z_max = f64::max(self.max.z(), rhs.max.z());
        
        Self::new(Point3::new(x_min, y_min, z_min), Point3::new(x_max, y_max, z_max))
    }
}

struct BvhNode {
    pub aabb: AABB,
    pub is_leaf: bool,
    pub left: usize,
    pub right: usize,
    pub first: usize,
    pub count: usize
}

impl BvhNode {
    pub fn new_node(aabb: AABB, left: usize, right: usize) -> Self {
        Self {
            aabb, is_leaf: false, left, right, first: 0, count: 0
        }
    }

    pub fn new_leaf(aabb: AABB, first:  usize, count: usize) -> Self {
        Self {
            aabb, is_leaf: true, first, count, left: 0, right: 0
        }
    }
}
