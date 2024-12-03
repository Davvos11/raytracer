use std::rc::Rc;
use crate::acceleration::aabb::AABB;
use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

// http://www.cse.yorku.ca/~amana/research/grid.pdf used for traversal per ray
pub struct Grid {
    pub objects: Vec<Rc<dyn Hittable>>,
    pub boxes: Vec<GridBox>,
    pub box_size: Vec3,
    pub origin: Point3,
    pub total_size: Point3
}

impl Grid {
    pub fn new(objects: Vec<Rc<dyn Hittable>>, box_size: Vec3, origin: Point3, end: Point3, total_size: Point3) -> Self {
        let mut origin_box = GridBox::new(origin, box_size);
        origin_box.try_add_all(&objects);
        let mut boxes = vec![origin_box ; ((total_size.x() / box_size.x()) * (total_size.y() / box_size.y()) * (total_size.z() / box_size.z())) as usize];
        let start_x = origin.x() as i32;
        let start_y = origin.y() as i32;
        let start_z = origin.z() as i32;
        let end_x = end.x() as i32;
        let end_y = end.y() as i32;
        let end_z = end.z() as i32;
        for x in (start_x..end_x).step_by(box_size.x() as usize) {
            for y in (start_y..end_y).step_by(box_size.y() as usize) {
                for z in (start_z..end_z).step_by(box_size.z() as usize) {
                    let min: Point3 = Point3::new(x as f64, y as f64, z as f64);
                    let distance_check = min - origin;
                    if distance_check.length_squared() <= 1.0 {
                        continue // I dont see another way to do this
                    }
                    let mut grid_box = GridBox::new(min, box_size);
                    grid_box.try_add_all(&objects);
                    let final_x = (x as f64 + origin.x()) as i32;
                    let final_y = (y as f64 + origin.y()) as i32;
                    let final_z = (z as f64 + origin.z()) as i32;                    
                    
                    let index = Self::get_index(Point3::new(final_x as f64, final_y as f64, final_z as f64), box_size, total_size);
                    boxes[index] = grid_box;
                }
            }
        }
        
        Self { objects, boxes, box_size, origin, total_size }
    }
    
    pub fn get_index(point: Point3, box_size: Vec3, total_size: Point3) -> usize {
        let x = (point.x() / box_size.x()) as usize;
        let y = (point.y() / box_size.y()) as usize;
        let z = (point.z() / box_size.z()) as usize;
        
        let sizex = (total_size.x() / box_size.x());
        let sizey = (total_size.y() / box_size.y());

        (x as f64 + y as f64 * sizex + z as f64 * sizex * sizey) as usize
    }
    
    /// Finds the grid box that a ray enters
    pub fn get_grid_box_origin_from_ray(&self, ray: Ray) -> Option<&GridBox> {
        let t 
    }
    
    /// Finds the grid box that a given point on a ray is in
    pub fn get_grid_box_from_ray(&self, ray: Ray, t: f64) -> Option<&GridBox> {
        let point = *ray.origin() + *ray.direction() * t;
        
        self.get_grid_box_from_point(point)
    }
    
    /// Calculates the grid box given a point in O(1) time
    pub fn get_grid_box_from_point(&self, point: Point3) -> Option<&GridBox> {
        let x = (point.x() - self.origin.x()) / self.box_size.x();
        let y = (point.y() - self.origin.y()) / self.box_size.y();
        let z = (point.z() - self.origin.z()) / self.box_size.z();
        
        let index = Self::get_index(Point3::new(x.floor(), y.floor(), z.floor()), self.box_size, self.total_size);
        if index >= self.boxes.len() {
            return None; // outside the grid
        }
        
        Some(&self.boxes[index])
    }
}

#[derive(Clone)]
pub struct GridBox {
    pub aabb: AABB,
    pub objects: Vec<usize>
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
            //println!("it should be in there");
            self.objects.push(array_pos);
        }
    }
    
    pub fn try_add_all(&mut self, objects: &Vec<Rc<dyn Hittable>>) {
        for (i, obj) in objects.iter().enumerate() {
            self.try_add(obj.to_aabb(), i);
        }
    }
    
}
