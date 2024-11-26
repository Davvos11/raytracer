use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use crate::acceleration::bvh::AABB;
use crate::rtweekend::IntersectionAlgorithm;

#[derive(Default, Serialize, Deserialize)]
pub struct HittableList {
    pub objects: Vec<Rc<dyn Hittable>>,
    #[serde(skip)]
    pub algorithm: IntersectionAlgorithm,
}

impl HittableList {
    pub fn new(object: Rc<dyn Hittable>) -> Self {
        Self { objects: vec![object], algorithm: Default::default() }
    }

    pub fn clear(&mut self) { self.objects.clear(); }

    pub fn add(&mut self, object: Rc<dyn Hittable>) {
        self.objects.push(object);
    }
}

#[typetag::serde]
impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        match self.algorithm {
            IntersectionAlgorithm::Naive => {
                for object in &self.objects {
                    if object.hit(r, Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                        hit_anything = true;
                        closest_so_far = temp_rec.t;
                        *rec = temp_rec.clone();
                    }
                }
            }
            IntersectionAlgorithm::BVH => {
                todo!()
            }
        }

        hit_anything
    }

    /// This function is not very performant and mainly exists to
    /// satisfy the Trait implementation.
    /// TODO: maybe we can use Into<AABB> as a separate trait instead
    fn to_aabb(&self) -> AABB {
        if let Some(first) = self.objects.first() {
            // Combine all AABBs by folding over the + implementation
            self.objects[1..].iter()
                .fold(first.to_aabb(),
                      |aabb, object| { aabb + object.to_aabb() },
                )
        } else {
            AABB::default()
        }
    }
}