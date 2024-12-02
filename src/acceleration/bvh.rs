use crate::acceleration::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::objects_to_aabb;
use crate::interval::Interval;
use crate::ray::Ray;
use std::rc::Rc;
use crate::data::Data;
use crate::rtweekend::AlgorithmOptions;
use crate::rtweekend::AlgorithmOptions::{BvhNaive, BvhSahPlane};

/// BVH and AABB from course slides
pub struct Bvh {
    objects: Vec<Rc<dyn Hittable>>,
    nodes: Vec<BvhNode>,
    node_pointer: usize,
}

impl Bvh {
    pub fn new(objects: Vec<Rc<dyn Hittable>>, options: &[AlgorithmOptions]) -> Self {
        let nodes = vec![BvhNode::default(); objects.len() * 4];
        let mut result = Self { objects, nodes, node_pointer: 0 };
        let mut root = BvhNode::new_leaf(0, 0, result.objects.len(), &result.objects);
        root.sub_divide(&mut result, options);
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

    /// Create a new leaf node
    /// `first` is relative to the provided `objects` array
    /// `offset + first` will be the index into the corresponding `Bvh.objects`
    pub fn new_leaf(first: usize, offset: usize, count: usize, objects: &[Rc<dyn Hittable>]) -> Self {
        let aabb = objects_to_aabb(&objects[first..(first + count)]);
        Self { aabb, is_leaf: true, first: offset + first, count, left: 0, right: 0 }
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

    fn get_split(&self, objects: &mut [Rc<dyn Hittable>], options: &[AlgorithmOptions]) -> Option<(BvhNode, BvhNode)> {
        if objects.len() < 3 { return None; }
        
        if options.contains(&BvhNaive) {
            // Sort objects on this axis
            objects.sort_by(|a, b| {
                let a = a.centroid().x();
                let b = b.centroid().x();
                f64::total_cmp(&a, &b)
            });
            let split = self.count / 2;
            let left_node = BvhNode::new_leaf(0, self.first, split, objects);
            let right_node = BvhNode::new_leaf(split, self.first, self.count - split, objects);
            return Some((left_node, right_node));
        }

        let current_heuristic = self.aabb.surface_area() as usize * self.count;
        // Check each axis
        for axis in 0..3 {
            // Sort objects on this axis
            objects.sort_by(|a, b| {
                let a = a.centroid()[axis];
                let b = b.centroid()[axis];
                f64::total_cmp(&a, &b)
            });

            if options.contains(&BvhSahPlane) {
                let split = self.count / 2;
                if let result @ Some(_) = self.check_split_sah(split, objects, current_heuristic) {
                    return result;
                }
            } else {
                for split in 0..objects.len() {
                    if let result @ Some(_) = self.check_split_sah(split, objects, current_heuristic) {
                        return result;
                    }
                }
            }
        }
        None
    }

    /// Surface Area Heuristic: a split if only worth it if:
    /// left.surface_area * left.objects.len() + right.surface_area * right.objects.len()
    ///    is less than self.surface_area * self.objects.len()
    fn check_split_sah(&self, split: usize, objects: &[Rc<dyn Hittable>], current_heuristic: usize) -> Option<(BvhNode, BvhNode)> {
        let left_node = BvhNode::new_leaf(0, self.first, split, objects);
        let right_node = BvhNode::new_leaf(split, self.first, self.count - split, objects);
        let left_heuristic = left_node.aabb.surface_area() as usize * split;
        let right_heuristic = right_node.aabb.surface_area() as usize * (self.count - split);
        if left_heuristic + right_heuristic < current_heuristic {
            Some((left_node, right_node))
        } else {
            None
        }
    }

    pub fn sub_divide(&mut self, bvh: &mut Bvh, options: &[AlgorithmOptions]) {
        assert!(self.is_leaf, "We assume that nodes start out as leaves and are then subdivided");
        let nodes = self.get_split(self.objects_mut(bvh), options);
        if let Some((mut left_node, mut right_node)) = nodes {
            // Make intermediate node
            self.left = bvh.node_pointer + 1;
            self.right = bvh.node_pointer + 2;
            bvh.node_pointer += 2;

            left_node.sub_divide(bvh, options);
            bvh.nodes[self.left] = left_node;

            right_node.sub_divide(bvh, options);
            bvh.nodes[self.right] = right_node;

            self.is_leaf = false;
        }
        // If no split found, keep this as a leaf.
    }

    pub fn hit_aabb(&self, r: &Ray, ray_t: Interval, data: &mut Data) -> Option<f64> {
        data.add_intersection_check();
        self.aabb.hit(r, ray_t)
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval, bvh: &Bvh, rec: &mut HitRecord, data: &mut Data) -> bool {
        if self.hit_aabb(r, ray_t, data).is_none() { return false }
        
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
