use glam::Vec3;
use crate::hittable::aabb::Aabb;
use crate::hittable::{bvh_node, HittableSurfaces};

pub struct BvhNode {
    left: Option<Box<HittableSurfaces>>,
    right: Option<Box<HittableSurfaces>>,
    bbox: Aabb,
}


impl BvhNode {
    pub fn init_from_vec(objects: &[HittableSurfaces]) -> Self {
        let mut left = None;
        let mut right = None;


        let len = objects.len();
        match len {
            1 => { (left, right) = (Some(Box::new(objects[0])), None); },
            2 => { (left, right) = (Some(Box::new(objects[0])), Some(Box::new(objects[1]))) },
            _ => {
                left = Some(Box::new(HittableSurfaces::BVHNode(BvhNode::init_from_vec(&objects[0..len / 2]))));
                right = Some(Box::new(HittableSurfaces::BVHNode(BvhNode::init_from_vec(&objects[len / 2..0]))));
            },
        }

        let mut bbox = Aabb::new(Vec3::ZERO, Vec3::ZERO);
        if let Some(surface) = left { bbox = Aabb::unit(&bbox, &surface.bbox())}
        Self {
            left,
            right: None,
            bbox: Aabb::unit(),
        }
    }

}


