use crate::acceleration::aabb::AABB;
use crate::acceleration::bvh::Bvh;
use crate::data::Data;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::{IntersectionAlgorithm, Options};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use crate::acceleration::grid::Grid;
use crate::vec3::{Point3, Vec3};
use std::time::Instant;

#[derive(Default, Serialize, Deserialize)]
pub struct HittableList {
    pub objects: Vec<Rc<dyn Hittable>>,
    #[serde(skip)]
    pub algorithm: IntersectionAlgorithm,
    #[serde(skip)]
    pub options: Options,
    #[serde(skip)]
    bvh: Option<Bvh>,
    #[serde(skip)]
    grid: Option<Grid>,
}

impl HittableList {
    pub fn new(object: Rc<dyn Hittable>) -> Self {
        Self { objects: vec![object], algorithm: Default::default(), options: Default::default(), bvh: None, grid: None }
    }

    pub fn init(&mut self) {
        match self.algorithm {
            IntersectionAlgorithm::BVH => {
                let t = Instant::now();
                self.bvh = Some(Bvh::new(self.objects.clone(), &self.options));
                eprintln!("BVH constructed in {:3.2?}", t.elapsed())
            }
            IntersectionAlgorithm::Grid => {
                let t = Instant::now();
                let size = self.options.grid_size;
                self.grid = Some(Grid::new(self.objects.clone(), Vec3::new(size, size, size), Point3::new(-100.0, -100.0, -100.0), Point3::new(100.0, 100.0, 100.0), Point3::new(200.0, 200.0, 200.0)));
                if let Some(grid) = &self.grid {
                    for box_ in &grid.boxes {
                        if box_.objects.len() <= 1 { continue }
                        println!("{:?}: {:?}", box_.aabb, box_.objects);
                    }
                }
                eprintln!("Grid constructed in {:3.2?}", t.elapsed())
            }
            _ => {}
        }
    }

    pub fn clear(&mut self) { self.objects.clear(); }

    pub fn add(&mut self, object: Rc<dyn Hittable>) {
        self.objects.push(object);
    }
}

#[typetag::serde]
impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data) -> bool {
        match self.algorithm {
            IntersectionAlgorithm::Naive => {
                let mut hit_anything = false;
                let mut closest_so_far = ray_t.max;

                for object in &self.objects {
                    data.add_intersection_check();
                    if object.hit(r, Interval::new(ray_t.min, closest_so_far), rec, data) {
                        hit_anything = true;
                        closest_so_far = rec.t;
                    }
                }

                hit_anything
            }
            IntersectionAlgorithm::BVH => {
                if let Some(bvh) = &self.bvh {
                    if let Some(root) = bvh.root() {
                        let root_hit = root.hit_aabb(r, ray_t, rec, data, &self.options).is_some();
                        if root_hit && root.hit(r, ray_t, bvh, rec, data, &self.options) {
                            return true;
                        }
                    }
                    false
                } else {
                    panic!("Please run HittableList.init() first")
                }
            }
            IntersectionAlgorithm::Grid => {
                if let Some(grid) = &self.grid {
                    grid.hit(r, ray_t, rec, data, &self.options)
                } else {
                    panic!("Please run Grid::new first")
                }
            }
        }
    }

    fn to_aabb(&self) -> AABB {
        objects_to_aabb(&self.objects)
    }

    fn centroid(&self) -> Point3 {
        eprintln!("Warning: HittableList.centroid() is slow");
        let aabb = self.to_aabb();
        Point3::new(
            (aabb.min.x() + aabb.max.x()) / 2.0,
            (aabb.min.y() + aabb.max.y()) / 2.0,
            (aabb.min.z() + aabb.max.z()) / 2.0,
        )
    }

    fn surface_area(&self) -> f64 {
        objects_surface_area(&self.objects)
    }
}

pub fn objects_to_aabb(objects: &[Rc<dyn Hittable>]) -> AABB {
    if let Some(first) = objects.first() {
        // Combine all AABBs by folding over the + implementation
        objects[1..].iter()
            .fold(first.to_aabb(),
                  |aabb, object| { aabb + object.to_aabb() },
            )
    } else {
        AABB::default()
    }
}

pub fn objects_surface_area(objects: &[Rc<dyn Hittable>]) -> f64 {
    objects.iter().map(|o| o.surface_area()).sum()
}