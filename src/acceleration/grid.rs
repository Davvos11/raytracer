use std::rc::Rc;
use crate::acceleration::aabb::AABB;
use crate::hittable::Hittable;
use crate::vec3::{Point3, Vec3};

pub struct Grid {
    objects: Vec<Rc<dyn Hittable>>,
    boxes: Vec<GridBox>,
    size: Vec3
}

impl Grid {
    pub fn new(objects: Vec<Rc<dyn Hittable>>, box_size: Vec3, origin: Point3, end: Point3, total_size: Point3) -> Self {
        let mut origin_box = GridBox::new(origin, box_size);
        origin_box.try_add_all(&objects);
        let mut boxes = vec![origin_box ; (total_size.x() * total_size.y() * total_size.z()) as usize];
        let start_x = origin.x() as i32;
        let start_y = origin.y() as i32;
        let start_z = origin.z() as i32;
        let end_x = end.x() as i32;
        let end_y = end.y() as i32;
        let end_z = end.z() as i32;
        for x in (start_x..=end_x).step_by(box_size.x() as usize) {
            for y in (start_y..=end_y).step_by(box_size.y() as usize) {
                for z in (start_z..=end_z).step_by(box_size.z() as usize) {
                    let min: Point3 = Point3::new(x as f64, y as f64, z as f64);
                    let distance_check = min - origin;
                    if distance_check.length_squared() <= 1.0 {
                        continue // I dont see another way to do this
                    }
                    let mut grid_box = GridBox::new(min, box_size);
                    grid_box.try_add_all(&objects);
                    boxes[Self::get_index(Point3::new(x as f64, y as f64, z as f64), box_size, total_size)] = grid_box;
                }
            }
        }
        
        Self { objects, boxes, size: box_size }
    }
    
    pub fn get_index(origin: Point3, box_size: Vec3, total_size: Point3) -> usize {
        let x = (origin.x() / box_size.x()) as usize;
        let y = (origin.y() / box_size.y()) as usize;
        let z = (origin.z() / box_size.z()) as usize;

        (x as f64 + y as f64 * total_size.x() + z as f64 * total_size.x() * total_size.y()) as usize
    }
}

#[derive(Clone)]
pub struct GridBox {
    aabb: AABB,
    objects: Vec<usize>
}

impl GridBox {
    pub fn new(origin: Point3, size: Vec3) -> Self {
        let aabb: AABB = AABB::new(origin, origin + size);
        Self { aabb, objects: Vec::new() }
    }
    
    pub fn try_add(&mut self, object: AABB, array_pos: usize) {
        let aabb = self.aabb.clone();
        let other = object.clone(); // todo: I'm too new to rust to know a better solution lol
        if aabb.inside(other) || object.inside(aabb) {
            self.objects.push(array_pos);
        }
    }
    
    pub fn try_add_all(&mut self, objects: &Vec<Rc<dyn Hittable>>) {
        for (i, obj) in objects.iter().enumerate() {
            self.try_add(obj.to_aabb(), i);
        }
    }
    
}
