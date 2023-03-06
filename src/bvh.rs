use std::collections::HashSet;

use crate::{bbox::AABB, ray::Ray, vec::f64x3};

pub struct BVHPrimitive {
    pub bbox: AABB,
    pub primitive: usize
}

struct BVHNode {
    bbox: AABB,
    left_or_prim: u32,
    right_or_count: u32
}

impl BVHNode {
    pub fn new(bbox: AABB, is_leaf: bool, left_or_prim: u32, right_or_count: u32) -> BVHNode {
        if left_or_prim & 0x80000000 != 0 {
            panic!("Only 31-bit are used to index BVH nodes. 32 bit is used to distinguish leaf nodes.")
        }
        let mut left = left_or_prim;
        if is_leaf {
            left = left | 0x80000000
        }
        BVHNode { bbox, left_or_prim: left, right_or_count }
    }
    pub fn is_leaf(&self) -> bool {
        self.left_or_prim & 0x80000000 != 0
    }

    pub fn left_child(&self) -> usize {
        (self.left_or_prim & 0x7FFFFFFF) as usize
    }

    pub fn right_child(&self) -> usize {
        self.right_or_count as usize
    }

    pub fn primitive(&self) -> usize {
        (self.left_or_prim & 0x7FFFFFFF) as usize
    }

    pub fn count(&self) -> usize {
        self.right_or_count as usize
    }

    pub fn bbox(&self) -> &AABB {
        &self.bbox
    }
}

pub struct BVH {
    nodes: Vec<BVHNode>
}

pub struct IsectInfo {
    pub primitive: usize,
    pub t: f64
}

impl BVH {
    pub fn intersection(&self, ray: &Ray, tmax: f32,
        isect: &dyn Fn(usize, f64x3, f64x3, f64) -> Option<f64>) -> Option<IsectInfo> {

        let root_node = self.nodes.len() - 1;
        let mut cur_t = tmax as f64;
        let mut cur_primitive = 0;
        let mut stack = [0usize; 50];
        stack[0] = root_node;
        let mut stack_pointer: usize = 1;

        let origin = f64x3::from(ray.origin);
        let direction = f64x3::from(ray.direction);

        loop {
            if stack_pointer == 0 {
                break
            }

            let node = &self.nodes[stack[stack_pointer - 1]];
            stack_pointer -= 1;

            if node.bbox.intersection(ray) {
                if node.is_leaf() {
                    let prim = node.primitive();
                    if let Some(t) = isect(prim, origin, direction, cur_t) {
                        cur_t = t;
                        cur_primitive = prim;
                    }
                } else {
                    stack[stack_pointer] = node.left_child();
                    stack_pointer += 1;
                    stack[stack_pointer] = node.right_child();
                    stack_pointer += 1;
                }
            }
        }
        if cur_t != tmax as f64 {
            return Some(IsectInfo{primitive: cur_primitive, t: cur_t})
        }
        None
    }

    pub fn visible(&self, ray: &Ray, tmax: f32,
        isect: &dyn Fn(usize, f64x3, f64x3, f64) -> Option<f64>) -> bool {

        let root_node = self.nodes.len() - 1;
        let cur_t = tmax as f64;
        let mut stack = [0usize; 50];
        stack[0] = root_node;
        let mut stack_pointer: usize = 1;

        let origin = f64x3::from(ray.origin);
        let direction = f64x3::from(ray.direction);

        loop {
            if stack_pointer == 0 {
                break
            }

            let node = &self.nodes[stack[stack_pointer - 1]];
            stack_pointer -= 1;

            if node.bbox.intersection(ray) {
                if node.is_leaf() {
                    let prim = node.primitive();
                    if let Some(_t) = isect(prim, origin, direction, cur_t) {
                        return false
                    }
                } else {
                    stack[stack_pointer] = node.left_child();
                    stack_pointer += 1;
                    stack[stack_pointer] = node.right_child();
                    stack_pointer += 1;
                }
            }
        }
        return true
    }
}


pub fn build_bottom_up_bvh(primitives: &Vec<BVHPrimitive>) -> BVH {
    let mut nodes = Vec::with_capacity(primitives.len()*2);
    let mut active_nodes = HashSet::new();
    for prim in primitives {
        let bbox = AABB::new(prim.bbox.min, prim.bbox.max);
        let node = BVHNode::new(bbox, true, prim.primitive as u32, 1);
        active_nodes.insert(nodes.len());
        nodes.push(node);
    }

    loop {
        if active_nodes.len() == 1 {
            break
        }
        let mut min_area = f32::INFINITY;
        let mut l1 = 0;
        let mut l2 = 0;
        for n1 in active_nodes.iter() {
            for n2 in active_nodes.iter() {
                if n1 != n2 {
                    let area = &nodes[*n1].bbox().merge(&nodes[*n2].bbox()).area();
                    if *area < min_area {
                        l1 = *n1;
                        l2 = *n2;
                        min_area = *area;
                    }
                }
            }
        }

        if l1 == l2 {
            break
        }

        let bbox = &nodes[l1].bbox().merge(&nodes[l2].bbox());
        let node = BVHNode::new(*bbox, false, l1 as u32, l2 as u32);
        active_nodes.remove(&l1);
        active_nodes.remove(&l2);
        active_nodes.insert(nodes.len());
        nodes.push(node);
    }

    //println!("Number of nodes! {} {}", primitives.len(), nodes.len());
    // for node in &nodes {
    //     if node.is_leaf() {
    //         println!("{}", node.primitive());
    //     } else {
    //         println!("{} {} {:?} {:?}", node.left_child(), node.right_child(), node.bbox.min, node.bbox.max);
    //     }
    // }
    return BVH{nodes}

}


#[cfg(test)]
mod tests {

    use crate::vec::f32x3;

    use super::*;
    use std::mem;

    # [test]
    fn bvh_test() {
        print!("Bvh node size {}\n", mem::size_of::<BVHNode>());
        let bbox = AABB::new(f32x3(0.0, 0.5, 0.3), f32x3(0.99, 2.2, 3.3));
        let bvh = BVHNode::new(bbox, true, 2000000000, 0);
    }
}
