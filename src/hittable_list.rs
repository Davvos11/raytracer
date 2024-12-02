use crate::acceleration::aabb::AABB;
use crate::acceleration::bvh::Bvh;
use crate::data::Data;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::{AlgorithmOptions, IntersectionAlgorithm};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::time::Instant;
use crate::vec3::Point3;

#[derive(Default, Serialize, Deserialize)]
pub struct HittableList {
    pub objects: Vec<Rc<dyn Hittable>>,
    #[serde(skip)]
    pub algorithm: IntersectionAlgorithm,
    #[serde(skip)]
    pub options: Vec<AlgorithmOptions>,
    #[serde(skip)]
    bvh: Option<Bvh>,
}

impl HittableList {
    pub fn new(object: Rc<dyn Hittable>) -> Self {
        Self { objects: vec![object], algorithm: Default::default(), options: Vec::new(), bvh: None }
    }

    pub fn init(&mut self) {
        match self.algorithm {
            IntersectionAlgorithm::BVH => {
                let t = Instant::now();
                self.bvh = Some(Bvh::new(self.objects.clone(), &self.options));
                eprintln!("BVH constructed in {:3.2?}", t.elapsed())
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
                        if root.hit(r, ray_t, bvh, rec, data) {
                            return true;
                        }
                    }
                    false
                } else {
                    panic!("Please run HittableList.init() first")
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