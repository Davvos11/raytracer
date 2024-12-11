use std::rc::Rc;
use crate::acceleration::aabb::AABB;
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
    pub aabb: AABB,
}

impl Grid {
    
    /// Creates and fills a Grid according to the settings provided in the arguments
    pub fn new(objects: Vec<Rc<dyn Hittable>>, box_size: Vec3, origin: Point3, end: Point3, total_size: Point3) -> Self {
        let mut origin_box = GridBox::new(origin, box_size);
        origin_box.try_add_all(&objects);
        
        let mut boxes = vec![origin_box; ((total_size.x() / box_size.x()) * (total_size.y() / box_size.y()) * (total_size.z() / box_size.z())) as usize];
        
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
                    
                    // Don't add the origin box twice
                    let distance_check = min - origin;
                    if distance_check.length_squared() <= 1.0 {
                        continue;
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

    /// Calculates the index for a 1D array
    pub fn get_index(point: Point3, box_size: Vec3, total_size: Point3) -> usize {
        let x = (point.x() / box_size.x()) as usize;
        let y = (point.y() / box_size.y()) as usize;
        let z = (point.z() / box_size.z()) as usize;

        let size_x = total_size.x() / box_size.x();
        let size_y = total_size.y() / box_size.y();

        (x as f64 + y as f64 * size_x + z as f64 * size_x * size_y) as usize
    }

    /// Safe extension to get the index as to not fall outside the grid
    fn get_index_(&self, point: Point3) -> Option<usize> {
        let idx = Self::get_index(point - self.origin, self.box_size, self.total_size);
        if idx < self.boxes.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// Returns the distance of each dimension to the nearest grid border on the ray's direction
    pub fn get_tmax(&self, grid_box: &GridBox, ray: &Ray) -> Vec3 {
        let mut x_nearest = grid_box.aabb.min.x();
        if ray.direction().x() >= 0.0 {
            x_nearest = grid_box.aabb.max.x();
        }
        
        let mut y_nearest = grid_box.aabb.min.y();
        if ray.direction().y() >= 0.0 {
            y_nearest = grid_box.aabb.max.y();
        }
        
        let mut z_nearest = grid_box.aabb.min.z();
        if ray.direction().z() >= 0.0 {
            z_nearest = grid_box.aabb.max.z();
        }
        
        let t_x = (x_nearest - ray.origin().x()).abs() / ray.direction().x().abs();
        let t_y = (y_nearest - ray.origin().y()).abs() / ray.direction().y().abs();
        let t_z = (z_nearest - ray.origin().z()).abs() / ray.direction().z().abs();

        Vec3::new(t_x, t_y, t_z)
    }

    /// Returns the units of t for how far along each axis we need to move to cross a boundary
    pub fn get_tdelta(&self, ray: &Ray) -> Vec3 {
        let x = self.box_size.x() / ray.direction().x().abs();
        let y = self.box_size.y() / ray.direction().y().abs();
        let z = self.box_size.z() / ray.direction().z().abs();

        Vec3::new(x, y, z)
    }

    /// Returns the first box the ray encounters, which can be the box the ray starts inside
    pub fn get_box_enter(&self, ray: &Ray) -> Option<&GridBox> {
        if let Some(grid_box) = self.get_grid_box_from_point(*ray.origin()) {
            // origin is inside a box
            return Some(grid_box);
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

    /// Returns true if the ray hits an object, rec and data are updated.
    pub fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, options: &Options) -> bool {
        // The equation of the ray is →u + t →v for t ≥ 0. 
        //  The initialization phase begins by identifying the voxel in which the ray origin, →u, is found.
        if let Some(initial_box) = self.get_box_enter(r) {
            self.traverse(initial_box, r, ray_t, rec, data, options, 0)
        } else {
            // TODO this should not happen
            // panic!("Initial box not found");
            false
        }
    }

    /// Traverses the grid until the gridbox containing the object the ray intersects with is found
    fn traverse(&self, grid_box: &GridBox, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, options: &Options, depth: u32) -> bool {
        // println!("{:?}", &grid_box);
        // TODO this should probably not happen
        if depth > 10 { return false; }
        data.add_traversal_step();
        // Check for primitive intersections in this box
        if grid_box.hit(self, ray, ray_t, rec, data ,options) {
            // println!("Hit item! {:?}", rec.p);
            return true;
        }
        
        // The integer variables X and Y are initialized to the starting voxel coordinates.
        let xyz: Vec3 = grid_box.aabb.min;
        // Determine the point where we will leave this aabb
        data.add_intersection_check();
        let t_hit = grid_box.aabb.hit(ray, Interval::universe(), rec, options)
            // .expect(&format!("We do not hit the box we are in: {:?} with {:?}", grid_box, ray))
        ;
        // TODO this should also not happen
        if t_hit.is_none() { /*println!("sip...");*/ return false; }
        let t_hit = t_hit.unwrap();
        let step = t_hit.1.get_step(self);

        let next_xyz = xyz + step;
        // println!("{:?} + {:?} = {:?}", xyz, step, next_xyz);
        if self.outside(next_xyz) { return false; }
        if let Some(next_idx) = self.get_index_(next_xyz) {
            let next_box = &self.boxes[next_idx];
            // println!("Next box: {:?}", next_box.aabb);
            // Traverse next box
            self.traverse(next_box, ray, ray_t, rec, data, options, depth + 1)
        } else {
            false
        }
    }

    // TODO: could be removed
    pub fn traverse_2(&self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, options: &Options) -> bool {
        if let Some(cellBox) = self.get_grid_box_from_point(*ray.origin()) {
            if !cellBox.objects.is_empty() {
                let mut hit_copy = rec.clone();
                if cellBox.hit(self, ray, ray_t, &mut hit_copy, data, options) {
                    *rec = hit_copy.clone();
                    return true;
                }
            }

            let mut cell_index = cellBox.aabb.min;
            let mut delta_t = Vec3::default();
            let mut next_crossing_t = Vec3::default();
            let ray_origin_grid = *ray.origin() - self.origin;
            if ray.direction().x() < 0.0 {
                delta_t.set_x(-1.0 * self.box_size.x() / ray.direction().x());
                next_crossing_t.set_x(((ray_origin_grid.x() / self.box_size.x()).floor() * self.box_size.x() - ray_origin_grid.x()) / ray.direction().x());
            } else {
                delta_t.set_x(self.box_size.x() / ray.direction().x());
                next_crossing_t.set_x((((ray_origin_grid.x() / self.box_size.x()).floor() + 1.0) * self.box_size.x() - ray_origin_grid.x()) / ray.direction().x());
            }
            if ray.direction().y() < 0.0 {
                delta_t.set_y(-1.0 * self.box_size.y() / ray.direction().y());
                next_crossing_t.set_y(((ray_origin_grid.y() / self.box_size.y()).floor() * self.box_size.y() - ray_origin_grid.y()) / ray.direction().y());
            } else {
                delta_t.set_y(self.box_size.y() / ray.direction().y());
                next_crossing_t.set_y((((ray_origin_grid.y() / self.box_size.y()).floor() + 1.0) * self.box_size.y() - ray_origin_grid.y()) / ray.direction().y());
            }
            if ray.direction().z() < 0.0 {
                delta_t.set_z(-1.0 * self.box_size.z() / ray.direction().z());
                next_crossing_t.set_z(((ray_origin_grid.z() / self.box_size.z()).floor() * self.box_size.z() - ray_origin_grid.z()) / ray.direction().z());
            } else {
                delta_t.set_z(self.box_size.z() / ray.direction().z());
                next_crossing_t.set_z((((ray_origin_grid.z() / self.box_size.z()).floor() + 1.0) * self.box_size.z() - ray_origin_grid.z()) / ray.direction().z());
            }

            let mut t = 0.0;
            loop {
                if next_crossing_t.x() < next_crossing_t.y() {
                    if next_crossing_t.x() < next_crossing_t.z() {
                        t = next_crossing_t.x();
                        next_crossing_t.set_x(next_crossing_t.x() + delta_t.x());
                        if (ray.direction().x() < 0.0) {
                            cell_index.set_x(cell_index.x() - self.box_size.x());
                        } else {
                            cell_index.set_x(cell_index.x() + self.box_size.x());
                        }
                    } else {
                        t = next_crossing_t.z();
                        next_crossing_t.set_z(next_crossing_t.z() + delta_t.z());
                        if ray.direction().z() < 0.0 {
                            cell_index.set_z(cell_index.z() - self.box_size.z());
                        } else {
                            cell_index.set_z(cell_index.z() + self.box_size.z());
                        }
                    }
                } else {
                    if (next_crossing_t.y() < next_crossing_t.z()) {
                        t = next_crossing_t.y();
                        next_crossing_t.set_y(next_crossing_t.y() + delta_t.y());
                        if ray.direction().y() < 0.0 {
                            cell_index.set_y(cell_index.y() - self.box_size.y());
                        } else {
                            cell_index.set_y(cell_index.y() + self.box_size.y());
                        }
                    } else {
                        t = next_crossing_t.z();
                        next_crossing_t.set_z(next_crossing_t.z() + delta_t.z());
                        if ray.direction().z() < 0.0 {
                            cell_index.set_z(cell_index.z() - self.box_size.z());
                        } else {
                            cell_index.set_z(cell_index.z() + self.box_size.z());
                        }
                    }
                    if self.outside(cell_index) {
                        // todo: put debug stuff here if no workey
                        break;
                    }
                    if let Some(current_box) = self.get_grid_box_from_xyz(cell_index) {
                        if !current_box.objects.is_empty() {
                            let mut hit_copy = rec.clone();
                            if current_box.hit(self, ray, ray_t, &mut hit_copy, data, options) {
                                *rec = hit_copy.clone();
                                return true;
                            }
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

    
    /// TODO: debug should be removed later
    pub fn print_boxes(&self, hits: Vec<usize>) {
        for (i, b) in self.boxes.iter().enumerate() {
            if !b.objects.is_empty() && hits.contains(&i) {
                eprintln!("{}: {:?} {:?}", i, b.aabb.min, b.objects);
            }
        }
    }

    /// Returns true if the point is outside the grid
    pub fn outside(&self, t_max: Vec3) -> bool {
        let t_max_updated = t_max; // - self.origin;
        if (!self.aabb.point_inside(t_max_updated)) {
            //eprintln!("{} {} {}", t_max_updated.x(), t_max_updated.y(), t_max_updated.z());
            return true;
        }
        false
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

    /// Calculates the grid box from an xyz vector, which represents the minimum corner of a gridbox
    pub fn get_grid_box_from_xyz(&self, xyz: Vec3) -> Option<&GridBox> {
        let index = Self::get_index(Point3::new(xyz.x().floor(), xyz.y().floor(), xyz.z().floor()), self.box_size, self.total_size);
        if index >= self.boxes.len() {
            return None;
        }

        Some(&self.boxes[index])
    }

    #[allow(clippy::collapsible_if)] /// `point` should be a point on one of the planes
    pub fn step_from_point(&self, point: Point3, aabb: &AABB) -> Option<Vec3> {
        // println!("{:?} on {:?}", &point, aabb);
        let mut closest_axis = None;
        let mut closest = f64::INFINITY;
        let mut direction = 0;
        let epsilon = Interval::new(-1e-3, 1e-3);
        for axis in 0..3 {
            let min_diff = (point[axis] - aabb.min[axis]).abs();
            if epsilon.contains(min_diff) {
                if min_diff < closest {
                    closest_axis = Some(axis);
                    closest = min_diff;
                    direction = -1;
                }
            }
            let max_diff = (point[axis] - aabb.max[axis]).abs();
            if epsilon.contains(point[axis] - aabb.max[axis]) {
                if max_diff < closest {
                    closest_axis = Some(axis);
                    closest = min_diff;
                    direction = 1;
                }
            }
        }

        if let Some(axis) = closest_axis {
            let mut result = Point3::default();
            result[axis] = direction as f64 * self.box_size[axis];
            return Some(result);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct GridBox {
    pub aabb: AABB,
    pub objects: Vec<usize>,
}

impl GridBox {
    
    pub fn new(origin: Point3, size: Vec3) -> Self {
        let aabb: AABB = AABB::new(origin, origin + size);
        Self { aabb, objects: Vec::new() }
    }

    /// Adds an object into the grid box if (part of) it is inside the box's AABB
    pub fn try_add(&mut self, object: &AABB, array_pos: usize) {
        if self.aabb.inside(object) || object.inside(&self.aabb) {
            self.objects.push(array_pos);
        }
    }

    /// Checks for all objects if they should be in the grid box
    pub fn try_add_all(&mut self, objects: &Vec<Rc<dyn Hittable>>) {
        for (i, obj) in objects.iter().enumerate() {
            self.try_add(&obj.to_aabb(), i);
        }
    }

    /// Loops through all objects in the gridbox to see if the ray hits anything.
    pub fn hit(&self, grid: &Grid, ray: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, _options: &Options) -> bool {
        data.add_gridbox_intersection_check();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;
        let objects: Vec<_> = self.objects.iter().map(|&i| &grid.objects[i]).collect();
        for object in objects {
            data.add_intersection_check();
            if object.hit(ray, Interval::new(ray_t.min, closest_so_far), rec, data) {
                hit_anything = true;
                closest_so_far = rec.t;
            }
        }
        hit_anything
    }
}
