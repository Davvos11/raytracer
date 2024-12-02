use crate::acceleration::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::objects_to_aabb;
use crate::interval::Interval;
use crate::ray::Ray;
use std::rc::Rc;
use crate::data::Data;

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
        let mut root = BvhNode::new_leaf(0, result.objects.len(), &result.objects);
        root.sub_divide(&mut result);
        result.nodes[0] = root;
        result
    }

    pub fn root(&self) -> Option<&BvhNode> {
        self.nodes.first()
    }
}

#[derive(Clone)]
// TODO fit this in one cache line?
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
        Self {
            aabb: Default::default(),
            is_leaf: true,
            left: Default::default(),
            right: Default::default(),
            first: Default::default(),
            count: Default::default(),
        }
    }
}

impl BvhNode {
    pub fn new_node(aabb: AABB, left: usize, right: usize) -> Self {
        Self { aabb, is_leaf: false, left, right, first: 0, count: 0 }
    }

    pub fn new_leaf(first: usize, count: usize, objects: &[Rc<dyn Hittable>]) -> Self {
        Self { aabb: objects_to_aabb(objects), is_leaf: true, first, count, left: 0, right: 0 }
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
    
    fn get_split(&self, objects: &mut [Rc<dyn Hittable>]) -> Option<(BvhNode, BvhNode)> {
        let current_heuristic = self.aabb.surface_area() as usize * self.count;
        // Check each axis
        for axis in 0..3 {
            // Sort objects on this axis
            objects.sort_by(|a, b| {
                let a = a.centroid()[axis];
                let b = b.centroid()[axis];
                f64::total_cmp(&a, &b)
            });
            // Surface Area Heuristic: a split if only worth it if:
            // left.surface_area * left.objects.len() + right.surface_area * right.objects.len()
            //    is less than self.surface_area * self.objects.len()
            // TODO is as usize approximation okay?
            let split = self.count / 2;
            let mut left_node = BvhNode::new_leaf(self.first, split, objects);
            let mut right_node = BvhNode::new_leaf(self.first + split, self.count - split, objects);
            let left_heuristic = left_node.aabb.surface_area() as usize * split;
            let right_heuristic = right_node.aabb.surface_area() as usize * (self.count - split);
            if left_heuristic + right_heuristic < current_heuristic {
                return Some((left_node, right_node));
            }
        }
        None
    }

    pub fn sub_divide(&mut self, bvh: &mut Bvh) {
        assert!(self.is_leaf, "We assume that nodes start out as leaves and are then subdivided");
        let nodes = self.get_split(self.objects_mut(bvh));
        if let Some((mut left_node, mut right_node)) = nodes {
            // Make intermediate node
            self.left = bvh.node_pointer + 1;
            self.right = bvh.node_pointer + 2;
            bvh.node_pointer += 2;

            left_node.sub_divide(bvh);
            bvh.nodes[self.left] = left_node;
            
            right_node.sub_divide(bvh);
            bvh.nodes[self.right] = right_node;

            self.is_leaf = false;
        }
        // If no split found, keep this as a leaf.
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval, bvh: &Bvh, rec: &mut HitRecord, data: &mut Data) -> bool {
        data.add_intersection_check();
        if !self.aabb.hit(r, ray_t) {
            return false;
        }

        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        if self.is_leaf {
            for object in self.objects(bvh) {
                data.add_intersection_check();
                if object.hit(r, Interval::new(ray_t.min, closest_so_far), rec, data) {
                    hit_anything = true;
                    closest_so_far = rec.t;
                }
            }
            return hit_anything;
        } else {
            // TODO check which tree is closer? Or has the least intersections?
            if self.left(bvh).hit(r, ray_t, bvh, rec, data) {
                hit_anything = true;
                closest_so_far = rec.t;
            }
            if self.right(bvh).hit(r, Interval::new(ray_t.min, closest_so_far), bvh, rec, data) {
                hit_anything = true;
            }
        }

        hit_anything
    }
}
