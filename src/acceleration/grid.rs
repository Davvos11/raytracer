use std::any::Any;
use std::ops::Add;
use std::rc::Rc;
use crate::acceleration::aabb::AABB;
use crate::acceleration::bvh::Bvh;
use crate::data::Data;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::Options;
use crate::vec3::{Point3, Vec3};

// http://www.cse.yorku.ca/~amana/research/grid.pdf used for traversal per ray
pub struct Grid {
    pub objects: Vec<Rc<dyn Hittable>>,
    pub boxes: Vec<GridBox>,
    pub box_size: Vec3,
    pub origin: Point3,
    pub total_size: Point3,
    pub aabb: AABB
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
                    let final_x = (x as f64 - origin.x()) as i32;
                    let final_y = (y as f64 - origin.y()) as i32;
                    let final_z = (z as f64 - origin.z()) as i32;
                    
                    let index = Self::get_index(Point3::new(final_x as f64, final_y as f64, final_z as f64), box_size, total_size);
                    boxes[index] = grid_box;
                }
            }
        }
        
        Self { objects, boxes, box_size, origin, total_size, aabb: AABB::new(origin, end) }
    }
    
    pub fn get_index(point: Point3, box_size: Vec3, total_size: Point3) -> usize {
        let x = (point.x() / box_size.x()) as usize;
        let y = (point.y() / box_size.y()) as usize;
        let z = (point.z() / box_size.z()) as usize;
        
        let size_x = (total_size.x() / box_size.x());
        let size_y = (total_size.y() / box_size.y());

        (x as f64 + y as f64 * size_x + z as f64 * size_x * size_y) as usize
    }
    
    /// Gets value of t for which the ray crosses the first voxel boundary for x, y and z
    pub fn get_tmax(&self, grid_box: &GridBox, ray: &Ray) -> Vec3 {
        // make ray origin be minus the origin
        let x_origin = (ray.origin().x() / self.box_size.x() - (ray.origin().x() / self.box_size.x()).floor()) * self.box_size.x();
        let y_origin = (ray.origin().y() / self.box_size.y() - (ray.origin().y() / self.box_size.y()).floor()) * self.box_size.y();
        let z_origin = (ray.origin().z() / self.box_size.z() - (ray.origin().z() / self.box_size.z()).floor()) * self.box_size.z();
        
        // then look at for what t it is the box width
        let t_x = (self.box_size.x() - x_origin) / ray.direction().x();
        let t_y = (self.box_size.y() - y_origin) / ray.direction().y();
        let t_z = (self.box_size.z() - z_origin) / ray.direction().z();
        
        // then do ray.at(t) - ray.origin to find t_max - maybe not?
        
        
        Vec3::new(t_x, t_y, t_z)
    }
    
    /// Gets the units of t for how far along each axis we need to move to cross a boundary
    pub fn get_tdelta(&self, ray: &Ray) -> Vec3 {
        let x = self.box_size.x() / ray.direction().x().abs();
        let y = self.box_size.y() / ray.direction().y().abs();
        let z = self.box_size.z() / ray.direction().z().abs();
        
        Vec3::new(x, y, z)
    }
    
    /// Gets the first box the ray encounters, can be the box the ray starts inside
    pub fn get_box_enter(&self, ray: &Ray) -> Option<&GridBox> {
        if let Some(grid_box) = self.get_grid_box_from_point(*ray.origin()) {
            // origin is inside a box
            return Some(grid_box)
        }
        if let Some((min, _max)) = self.aabb.enter_and_exit(ray, Interval::new(0.001, f64::INFINITY)) {
            // origin is outside but the ray hits the outer aabb
            return self.get_grid_box_from_ray(ray, min);
        }
        None
    }
    
    /// Finds the grid box that a given point on a ray is in
    pub fn get_grid_box_from_ray(&self, ray: &Ray, t: f64) -> Option<&GridBox> {
        let point = *ray.origin() + *ray.direction() * t;
        
        self.get_grid_box_from_point(point)
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, options: &Options) -> bool {
        self.traverse(r, ray_t, rec, data, options)
    }
    
    /// Traverses the grid until the gridbox containing the object the ray intersects with is found
    pub fn traverse(&self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, options: &Options) -> bool {
        if let Some(grid_box) = self.get_box_enter(ray) {
            if !grid_box.objects.is_empty() { // gotta make sure the starting box doesnt already have the object in it
                let mut hitCopy2 = rec.clone();
                if grid_box.hit(self, ray, ray_t, &mut hitCopy2, data, options) {
                    *rec = hitCopy2.clone();
                    return true;
                }
            }
            let mut xyz: Vec3 = grid_box.aabb.min;
            let step: Vec3 = self.step(ray);
            let mut t_max: Vec3 = self.get_tmax(grid_box, ray);
            let t_delta: Vec3 = self.get_tdelta(ray);
            let mut xyz_hist = Vec::new();
            loop {
                xyz_hist.push(xyz.clone());
                /*let mut rec_copy = rec.clone();
                if self.check(ray, ray_t, &mut rec_copy, data) {
                    eprintln!("raycheck {} {} {} to {} {} {}", ray.origin().x(), ray.origin().y(), ray.origin().z(), ray.direction().x(), ray.direction().y(), ray.direction().z());
                    eprintln!("box enter is {} {} {}", grid_box.aabb.min.x(), grid_box.aabb.min.y(), grid_box.aabb.min.z());
                    eprintln!("{}, {}, {}", xyz.x(), xyz.y(), xyz.z());
                    eprintln!("{}, {}, {}", step.x(), step.y(), step.z());
                    eprintln!("{}, {}, {}", t_max.x(), t_max.y(), t_max.z());
                    eprintln!("{}, {}, {}", t_delta.x(), t_delta.y(), t_delta.z());
                }*/
                if t_max.x() < t_max.y() {
                    if t_max.x() < t_max.z() {
                        xyz += Vec3::new(step.x(), 0.0, 0.0);
                        if self.outside(xyz) {
                            let mut rec_copy = rec.clone();
                            if self.check(ray, ray_t, &mut rec_copy, data) {
                                eprintln!("raycheck {} {} {} to {} {} {}", ray.origin().x(), ray.origin().y(), ray.origin().z(), ray.direction().x(), ray.direction().y(), ray.direction().z());
                                eprintln!("box enter is {} {} {}", grid_box.aabb.min.x(), grid_box.aabb.min.y(), grid_box.aabb.min.z());
                                eprintln!("{}, {}, {}", xyz.x(), xyz.y(), xyz.z());
                                eprintln!("{}, {}, {}", step.x(), step.y(), step.z());
                                eprintln!("{}, {}, {}", t_max.x(), t_max.y(), t_max.z());
                                eprintln!("{}, {}, {}", t_delta.x(), t_delta.y(), t_delta.z());
                                eprintln!("hist: {:?}", xyz_hist);
                            }
                            return false;
                        }
                        t_max += Vec3::new(t_delta.x(), 0.0, 0.0);
                    } else {
                        xyz += Vec3::new(0.0, 0.0, step.z());
                        if self.outside(xyz) {
                            let mut rec_copy = rec.clone();
                            if self.check(ray, ray_t, &mut rec_copy, data) {
                                eprintln!("raycheck {} {} {} to {} {} {}", ray.origin().x(), ray.origin().y(), ray.origin().z(), ray.direction().x(), ray.direction().y(), ray.direction().z());
                                eprintln!("box enter is {} {} {}", grid_box.aabb.min.x(), grid_box.aabb.min.y(), grid_box.aabb.min.z());
                                eprintln!("{}, {}, {}", xyz.x(), xyz.y(), xyz.z());
                                eprintln!("{}, {}, {}", step.x(), step.y(), step.z());
                                eprintln!("{}, {}, {}", t_max.x(), t_max.y(), t_max.z());
                                eprintln!("{}, {}, {}", t_delta.x(), t_delta.y(), t_delta.z());
                                eprintln!("hist: {:?}", xyz_hist);
                            }
                            return false;
                        }
                        t_max += Vec3::new(0.0, 0.0, t_delta.z());
                    }
                } else {
                    if (t_max.y() < t_max.z()) {
                        xyz += Vec3::new(0.0, step.y(), 0.0);
                        if self.outside(xyz) {
                            let mut rec_copy = rec.clone();
                            if self.check(ray, ray_t, &mut rec_copy, data) {
                                eprintln!("raycheck {} {} {} to {} {} {}", ray.origin().x(), ray.origin().y(), ray.origin().z(), ray.direction().x(), ray.direction().y(), ray.direction().z());
                                eprintln!("box enter is {} {} {}", grid_box.aabb.min.x(), grid_box.aabb.min.y(), grid_box.aabb.min.z());
                                eprintln!("{}, {}, {}", xyz.x(), xyz.y(), xyz.z());
                                eprintln!("{}, {}, {}", step.x(), step.y(), step.z());
                                eprintln!("{}, {}, {}", t_max.x(), t_max.y(), t_max.z());
                                eprintln!("{}, {}, {}", t_delta.x(), t_delta.y(), t_delta.z());
                                eprintln!("hist: {:?}", xyz_hist);
                            }
                            return false;
                        }
                        t_max += Vec3::new(0.0, t_delta.y(), 0.0);
                    } else {
                        xyz += Vec3::new(0.0, 0.0, step.z());
                        if self.outside(xyz) {
                            let mut rec_copy = rec.clone();
                            if self.check(ray, ray_t, &mut rec_copy, data) {
                                eprintln!("raycheck {} {} {} to {} {} {}", ray.origin().x(), ray.origin().y(), ray.origin().z(), ray.direction().x(), ray.direction().y(), ray.direction().z());
                                eprintln!("box enter is {} {} {}", grid_box.aabb.min.x(), grid_box.aabb.min.y(), grid_box.aabb.min.z());
                                eprintln!("{}, {}, {}", xyz.x(), xyz.y(), xyz.z());
                                eprintln!("{}, {}, {}", step.x(), step.y(), step.z());
                                eprintln!("{}, {}, {}", t_max.x(), t_max.y(), t_max.z());
                                eprintln!("{}, {}, {}", t_delta.x(), t_delta.y(), t_delta.z());
                                eprintln!("hist: {:?}", xyz_hist);
                            }
                            return false;
                        }
                        t_max += Vec3::new(0.0, 0.0, t_delta.z());
                    }
                }
                if let Some(current_box) = self.get_grid_box_from_xyz(xyz) {
                    if !current_box.objects.is_empty() {
                        let mut hitCopy = rec.clone();
                        if current_box.hit(self, ray, ray_t, &mut hitCopy, data, options) {
                            *rec = hitCopy.clone();
                            return true;
                        }
                    }
                }
                
            }
        }
        false
    }
    
    //TODO: debug function, should go once the bugs are fixed
    pub fn check(&self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data) -> bool {
        let mut hr = rec.clone();
        let mut i = false;
        let mut thing = Vec::new();
        let mut hits = Vec::new();
        
        for (j, h) in self.objects.iter().enumerate() {
            if h.hit(ray, ray_t, &mut hr, data) {
                thing.push((h.to_aabb().min, h.to_aabb().max));
                i = true;
                hits.push(j);
            }
        }
        if i {
            let mut box_list = Vec::new();
            let mut prev = Vec3::new(-1000.0, -1000.0, -1000.0);
            for k in 0..150 {
                let pos = ray.at(k as f64);
                if let Some(test_box) = self.get_grid_box_from_point(pos) {
                    if test_box.aabb.min.x() as i32 != prev.x() as i32
                        && test_box.aabb.min.y() as i32 != prev.y() as i32
                        && test_box.aabb.min.z() as i32 != prev.z() as i32 {
                        box_list.push(test_box.aabb.min);
                        prev = test_box.aabb.min;
                    }
                } else {
                    break;
                }
                
            }
            eprintln!("checking for next ray");
            *rec = hr.clone();
            eprintln!("{:?}", thing);
            eprintln!("{:?}", hits);
            self.print_boxes(hits);
            eprintln!("pathL {:?}", box_list);
        }
        i
    }
    
    pub fn print_boxes(&self, hits: Vec<usize>) {
        for (i, b) in self.boxes.iter().enumerate() {
            if !b.objects.is_empty() && hits.contains(&i){
                eprintln!("{}: {:?} {:?}", i, b.aabb.min, b.objects);
            }
        }
    }
    
    pub fn outside(&self, t_max: Vec3) -> bool {
        let t_max_updated = t_max - self.origin;
        if (!self.aabb.point_inside(t_max_updated)) {
            //eprintln!("{} {} {}", t_max_updated.x(), t_max_updated.y(), t_max_updated.z());
            return true;
        }
        false
    }
    
    pub fn step(&self, ray: &Ray) -> Vec3 {
        let mut stepx = self.box_size.x();
        if ray.direction().x() < 0.0 {
            stepx *= -1.0;
        }
        let mut stepy = self.box_size.y();
        if ray.direction().y() < 0.0 {
            stepy *= -1.0;
        }
        let mut stepz = self.box_size.z();
        if ray.direction().z() < 0.0 {
            stepz *= -1.0;
        }
        Vec3::new(stepx, stepy, stepz)
    }
    
    /// Calculates the grid box given a point in O(1) time
    pub fn get_grid_box_from_point(&self, point: Point3) -> Option<&GridBox> {
        let x = ((point.x().floor() as i32 - self.origin.x() as i32) as f64 / self.box_size.x()).floor() * self.box_size.x();
        let y = ((point.y().floor() as i32 - self.origin.y() as i32) as f64 / self.box_size.y()).floor() * self.box_size.y();
        let z = ((point.z().floor() as i32 - self.origin.z() as i32) as f64 / self.box_size.z()).floor() * self.box_size.z();
                
        let index = Self::get_index(Point3::new(x.floor(), y.floor(), z.floor()), self.box_size, self.total_size);
        if index >= self.boxes.len() {
            return None; // outside the grid
        }
        
        Some(&self.boxes[index])
    }
    
    pub fn get_grid_box_from_xyz(&self, xyz: Vec3) -> Option<&GridBox> {
        let index = Self::get_index(Point3::new(xyz.x().floor(), xyz.y().floor(), xyz.z().floor()), self.box_size, self.total_size);
        if index >= self.boxes.len() {
            return None;
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
    
    pub fn try_add(&mut self, object: &AABB, array_pos: usize) {
        if self.aabb.inside(object) || object.inside(&self.aabb) {
            self.objects.push(array_pos);
        }
    }
    
    pub fn try_add_all(&mut self, objects: &Vec<Rc<dyn Hittable>>) {
        for (i, obj) in objects.iter().enumerate() {
            self.try_add(&obj.to_aabb(), i);
        }
    }
    
    pub fn hit(&self, grid: &Grid, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, _options: &Options) -> bool {
        let mut hit_anything = false;
        data.add_gridbox_intersection_check();
        let mut rec_copy = rec.clone();
        for object in &self.objects[..] {
            let hittable = &grid.objects[*object];
            data.add_intersection_check();
            if hittable.hit(ray, ray_t, &mut rec_copy, data) {
                if let Some(hit_gridbox) = grid.get_grid_box_from_point(ray.at(rec_copy.t)) {
                    //eprintln!("inside grid box");
                    let d = ray.at(rec_copy.t);
                    //eprintln!("rec copy {}", rec_copy.t);
                    //eprintln!("ray at {} {} {}", d.x(), d.y(), d.z());
                   // eprintln!("in gridbox {} {} {}", hit_gridbox.aabb.min.x(), hit_gridbox.aabb.min.y(), hit_gridbox.aabb.min.z());
                    //eprintln!("and self is {} {} {}", self.aabb.min.x(), self.aabb.min.y(), self.aabb.min.z());
                    hit_anything = true;
                    if std::ptr::eq(self, hit_gridbox) {
                        //eprintln!("what");
                        hit_anything = true; // no need for further investigation as the nearest point will definitely be in this box
                    }
                }
            }
        }
        if hit_anything {
            *rec = rec_copy.clone();
        }
        hit_anything
    }
    
}
