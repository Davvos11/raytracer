use crate::acceleration::aabb::AABB;
use crate::data::Data;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::objects_to_aabb;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::rtweekend::AlgorithmOptions::{BvhNaive, BvhSahPlane};
use crate::rtweekend::Options;
use std::rc::Rc;

/// BVH and AABB from course slides
pub struct Bvh {
    objects: Vec<Rc<dyn Hittable>>,
    nodes: Vec<BvhNode>,
    node_pointer: usize,
}

impl Bvh {
    pub fn new(objects: Vec<Rc<dyn Hittable>>, options: &Options) -> Self {
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

    fn get_split(&self, objects: &mut [Rc<dyn Hittable>], options: &Options) -> Option<(BvhNode, BvhNode)> {
        if options.options.contains(&BvhNaive) {
            if objects.len() < 3 { return None; }
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

        let mut best_heuristic = self.aabb.surface_area() * self.count as f64;
        let mut best_split = None;
        let mut best_axis = 0;
        // Check each axis
        for axis in 0..3 {
            // Sort objects on this axis
            let mut axis_objects = objects.to_vec();
            axis_objects.sort_by(|a, b| {
                let a = a.centroid()[axis];
                let b = b.centroid()[axis];
                f64::total_cmp(&a, &b)
            });

            if options.options.contains(&BvhSahPlane) {
                // TODO check this (or ignore)
                let split = self.count / 2;
                if let Some((heuristic, left, right)) = self.check_split_sah(split, &axis_objects, best_heuristic) {
                    if heuristic < best_heuristic {
                        best_heuristic = heuristic;
                        best_split = Some((left, right));
                        best_axis = axis;
                    }
                }
            } else {
                for split in 1..objects.len() {
                    if let Some((heuristic, left, right)) = self.check_split_sah(split, &axis_objects, best_heuristic) {
                        if heuristic < best_heuristic {
                            best_heuristic = heuristic;
                            best_split = Some((left, right));
                            best_axis = axis;
                        }
                    }
                }
            }
        }
        objects.sort_by(|a, b| {
            let a = a.centroid()[best_axis];
            let b = b.centroid()[best_axis];
            f64::total_cmp(&a, &b)
        });
        best_split
    }

    /// Surface Area Heuristic: a split if only worth it if:
    /// left.surface_area * left.objects.len() + right.surface_area * right.objects.len()
    ///    is less than self.surface_area * self.objects.len()
    /// Returns the new heuristic and the two nodes (or None if the split is not worth it)
    fn check_split_sah(&self, split: usize, objects: &[Rc<dyn Hittable>], current_heuristic: f64)
                       -> Option<(f64, BvhNode, BvhNode)> {
        let left_node = BvhNode::new_leaf(0, self.first, split, objects);
        let right_node = BvhNode::new_leaf(split, self.first, self.count - split, objects);
        let left_heuristic = left_node.aabb.surface_area()  * split as f64;
        let right_heuristic = right_node.aabb.surface_area()  * (self.count - split) as f64;
        let new_heuristic = left_heuristic + right_heuristic;
        if new_heuristic < current_heuristic {
            Some((new_heuristic, left_node, right_node))
        } else {
            None
        }
    }

    pub fn sub_divide(&mut self, bvh: &mut Bvh, options: &Options) {
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

    pub fn hit_aabb(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord, data: &mut Data, options: &Options) -> Option<f64> {
        data.add_intersection_check();
        self.aabb.hit(r, ray_t, rec, options)
    }

    #[allow(clippy::collapsible_if)]
    pub fn hit(&self, r: &Ray, ray_t: Interval, bvh: &Bvh, rec: &mut HitRecord, data: &mut Data, options: &Options) -> bool {
        // if self.hit_aabb(r, ray_t, rec, data).is_none() { return false }

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
            let left = self.left(bvh);
            let left_distance = left.hit_aabb(r, ray_t, rec, data, options);
            let right = self.right(bvh);
            let right_distance = right.hit_aabb(r, ray_t, rec, data, options);

            if let (Some(left_distance), Some(right_distance)) = (left_distance, right_distance) {
                let (close, far) =
                    if left_distance >= right_distance { (right, left) } else { (left, right) };
                let far_distance = left_distance.max(right_distance);
                if close.hit(r, ray_t, bvh, rec, data, options) {
                    hit_anything = true;
                    closest_so_far = rec.t;
                    // If the hitpoint is also in the far aabb, then also check that
                    if far_distance < closest_so_far {
                        data.add_overlapping_aabb();
                        if far.hit(r, Interval::new(ray_t.min, closest_so_far), bvh, rec, data, options) {
                            hit_anything = true;
                        }
                    }
                } else if far.hit(r, ray_t, bvh, rec, data, options) {
                    hit_anything = true;
                }
            } else if left_distance.is_some() && left.hit(r, ray_t, bvh, rec, data, options) {
                hit_anything = true;
            } else if right_distance.is_some() && right.hit(r, ray_t, bvh, rec, data, options) {
                hit_anything = true;
            }
        }

        hit_anything
    }
}