use std::collections::HashMap;
use crate::hittable_list::HittableList;
use crate::intbox::IntBox;

#[derive(Default)]
pub struct Grid {
    hittable_lists: HashMap<IntBox, HittableList>
}

impl Grid {
    pub fn new() -> Grid {
        Grid { hittable_lists: HashMap::new() }
    }

    pub fn add(&mut self, coordinates: IntBox, list: HittableList) {
        self.hittable_lists.entry(coordinates).or_insert(list);
    }
    
    
}