use std::cmp::{Ordering, PartialOrd};
use crate::intpoint::IntPoint;

#[derive(Default, Eq, PartialEq, Hash)]
pub struct IntBox {
    pub min: IntPoint,
    pub max: IntPoint
}



impl IntBox {
    pub fn new(min: IntPoint, max: IntPoint) -> IntBox {
        IntBox { min, max }
    }
    
    pub fn is_inside(&self, other: IntPoint) -> bool {
        self.min.x() <= other.x() && self.min.y() <= other.y() && self.min.z() <= other.z() 
            && self.max.x() >= other.x() && self.max.y() <= other.y() && self.max.z() <= other.z()
    }
}