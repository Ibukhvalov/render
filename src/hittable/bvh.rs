use super::{aabb, HitRecord, Hittable, HittableSurfaces};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util;
use aabb::Aabb;

enum Node {
    Branch { left: Box<BVH>, right: Box<BVH> },
    Leaf(Box<HittableSurfaces>),
}

pub struct BVH {
    pub node: Node,
    pub bbox: Aabb,
}

impl BVH {
    pub fn init_from_vec(mut hittable_list: Vec<HittableSurfaces>) -> Self {
        let len = hittable_list.len();

        let mut axis_ranges: Vec<(usize, f32)> = (0..3)
            .map(|axis| (axis, util::axis_range(&hittable_list, axis)))
            .collect();

        axis_ranges.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let longest_axis = axis_ranges[0].0;

        hittable_list.sort_unstable_by(util::box_compare(longest_axis));

        match len {
            0 => {
                panic!["empty initial vector!"]
            }
            1 => {
                let surface = hittable_list.pop().unwrap();
                if let Some(bbox) = surface.get_bbox() {
                    BVH {
                        node: Node::Leaf(Box::new(surface)),
                        bbox,
                    }
                } else {
                    panic!("no bbox in surface")
                }
            }
            _ => {
                let right = BVH::init_from_vec(hittable_list.drain(len / 2..).collect());
                let left = BVH::init_from_vec(hittable_list);
                let bbox = Aabb::unit(&left.get_bbox().unwrap(), &right.get_bbox().unwrap());
                BVH {
                    node: Node::Branch {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    bbox,
                }
            }
        }
    }
}

impl Hittable for BVH {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        if self.bbox.hit(ray, &interval) {
            match &self.node {
                Node::Leaf(surface) => surface.hit(ray, interval),
                Node::Branch { left, right } => {
                    let left = left.hit(ray, interval);
                    if let Some(l) = &left {
                        interval.max = l.t.min;
                    }
                    let right = right.hit(ray, interval);
                    if right.is_some() {
                        right
                    } else {
                        left
                    }
                }
            }
        } else {
            None
        }
    }

    fn get_bbox(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}
