use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(Default, Eq, PartialEq, Hash)]
pub struct IntPoint {
    e: [i32; 3]
}

impl IntPoint {
    pub fn new(x: i32, y: i32, z: i32) -> IntPoint {
        IntPoint { e: [x, y, z] }
    }
    
    pub fn x(&self) -> i32 {
        self.e[0]
    }
    
    pub fn y(&self) -> i32 {
        self.e[1]
    }
    
    pub fn z(&self) -> i32 {
        self.e[2]
    }
}
