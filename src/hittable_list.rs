use crate::acceleration::aabb::AABB;
use crate::acceleration::bvh::Bvh;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::IntersectionAlgorithm;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Default, Serialize, Deserialize)]
pub struct HittableList {
    pub objects: Vec<Rc<dyn Hittable>>,
    #[serde(skip)]
    pub algorithm: IntersectionAlgorithm,
    #[serde(skip)]
    bvh: Option<Bvh>,
}

impl HittableList {
    pub fn new(object: Rc<dyn Hittable>) -> Self {
        Self { objects: vec![object], algorithm: Default::default(), bvh: None }
    }

    pub fn init(&mut self) {
        match self.algorithm {
            IntersectionAlgorithm::BVH => {
                self.bvh = Some(Bvh::new(self.objects.clone()));
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
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        match self.algorithm {
            IntersectionAlgorithm::Naive => {
                let mut hit_anything = false;
                let mut closest_so_far = ray_t.max;

                for object in &self.objects {
                    if object.hit(r, Interval::new(ray_t.min, closest_so_far), rec) {
                        hit_anything = true;
                        closest_so_far = rec.t;
                    }
                }

                hit_anything
            }
            IntersectionAlgorithm::BVH => {
                if let Some(bvh) = &self.bvh {
                    if let Some(root) = bvh.root() {
                        if root.hit(r, ray_t, bvh, rec) {
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