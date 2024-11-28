use crate::acceleration::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::objects_to_aabb;
use crate::interval::Interval;
use crate::ray::Ray;
use std::rc::Rc;

/// BVH and AABB from course slides
pub struct Bvh {
    objects: Vec<Rc<dyn Hittable>>,
    nodes: Vec<BvhNode>,
    node_pointer: usize,
}

impl Bvh {
    pub fn new(objects: Vec<Rc<dyn Hittable>>) -> Self {
        let nodes = vec![BvhNode::default(); objects.len() * 2 - 1];
        let mut result = Self { objects, nodes, node_pointer: 0 };
        // let rc = Rc::new(RefCell::new(result));
        let mut root = BvhNode::new_leaf(AABB::default(), 0, result.objects.len());
        root.sub_divide(&mut result);
        result.nodes[0] = root;
        result
    }

    pub fn root(&self) -> Option<&BvhNode> {
        self.nodes.first()
    }
}

#[derive(Clone)]
pub struct BvhNode {
    pub aabb: AABB,
    pub is_leaf: bool,
    pub left: usize,
    pub right: usize,
    pub first: usize,
    pub count: usize,
}

impl Default for BvhNode {
    fn default() -> Self {
        Self::new_leaf(AABB::default(), 0, 0)
    }
}

impl BvhNode {
    pub fn new_node(aabb: AABB, left: usize, right: usize) -> Self {
        Self { aabb, is_leaf: false, left, right, first: 0, count: 0 }
    }

    pub fn new_leaf(aabb: AABB, first: usize, count: usize) -> Self {
        Self { aabb, is_leaf: true, first, count, left: 0, right: 0 }
    }

    pub fn objects<'a>(&'a self, bvh: &'a Bvh) -> &'a [Rc<dyn Hittable>] {
        if !self.is_leaf { panic!("Cannot get objects for intermediate node") }
        &bvh.objects[self.first..(self.first + self.count)]
    }

    pub fn objects_mut<'a>(&'a self, bvh: &'a mut Bvh) -> &'a mut [Rc<dyn Hittable>] {
        if !self.is_leaf { panic!("Cannot get objects for intermediate node") }
        &mut bvh.objects[self.first..(self.first + self.count)]
    }

    pub fn left<'a>(&'a self, bvh: &'a Bvh) -> &'a BvhNode {
        if self.is_leaf { panic!("Cannot get child tree for leaf node") }
        &bvh.nodes[self.left]
    }

    pub fn right<'a>(&'a self, bvh: &'a Bvh) -> &'a BvhNode {
        if self.is_leaf { panic!("Cannot get child tree for leaf node") }
        &bvh.nodes[self.right]
    }

    pub fn sub_divide(&mut self, bvh: &mut Bvh) {
        assert!(self.is_leaf, "We assume that nodes start out as leaves and are then subdivided");
        let split = {
            // let mut bvh_b = bvh.borrow_mut();
            let objects = self.objects_mut(bvh);

            if objects.len() < 3 {
                // Set / keep as leaf
                self.aabb = objects_to_aabb(objects);
                return;
            }

            // For now we choose the x-axis always
            // TODO choose axis, also don't calculate full AABB, but use centroid
            objects.sort_by(|a, b| {
                let a = a.to_aabb().max.x();
                let b = b.to_aabb().min.x();
                f64::total_cmp(&a, &b)
            });
            // Return split index in this block
            // We use this block so `objects` goes out of scope since it has a
            // mutable borrow on bvh and we want to have a new mutable borrow in `sub_divide`
            objects.len() / 2
        };

        self.left = bvh.node_pointer + 1;
        self.right = bvh.node_pointer + 2;
        bvh.node_pointer += 2;

        let mut left_node = BvhNode::new_leaf(AABB::default(), self.first, split);
        left_node.sub_divide(bvh);
        self.aabb = left_node.aabb.clone();
        bvh.nodes[self.left] = left_node;

        let mut right_node = BvhNode::new_leaf(AABB::default(), self.first + split, self.count - split);
        right_node.sub_divide(bvh);
        self.aabb = &self.aabb + &right_node.aabb.clone();
        bvh.nodes[self.right] = right_node;

        self.is_leaf = false;
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval, bvh: &Bvh, hit_record: &mut HitRecord) -> bool {
        if !self.aabb.hit(r, ray_t) {
            return false;
        }

        if self.is_leaf {
            for object in self.objects(bvh) {
                if object.hit(r, ray_t, hit_record) {
                    return true;
                }
            }
        } else {
            if self.left(bvh).hit(r, ray_t, bvh, hit_record) {
                return true;
            }
            if self.right(bvh).hit(r, ray_t, bvh, hit_record) {
                return true;
            }
        }

        false
    }
}
