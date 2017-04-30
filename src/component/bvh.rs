// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Bounding volume hierarchy

use super::*;
use std::mem;
use std::sync::Arc;
use super::naive::Naive;
use copy_arena::{Arena, Allocator};

#[derive(Copy, Clone)]
struct ComponentInfo {
    bound: BBox3f,
    centroid: Point3f,
    cost: Float,
    idx: usize,
}

impl ComponentInfo {
    fn new(components: &[Arc<Composable>]) -> Vec<ComponentInfo> {
        let mut ret = Vec::with_capacity(components.len());
        for (idx, c) in components.iter().enumerate() {
            let bound = c.bbox_parent();
            let centroid = (bound.pmin + bound.pmax.to_vec())/2.0 as Float;
            ret.push(ComponentInfo{
                bound, centroid, idx,
                cost: c.intersection_cost(),
            });
        }
        ret
    }
}


/// BVH construction strategy
#[derive(Copy, Clone)]
pub enum BVHStrategy {
    /// splitting by surface area heuristics
    SAH,
    /// splitting by middle count
    MiddleCount,
    /// splitting by midpoint of the centroid bound
    MidPoint,
}

/// Bounding volume hierarchy used for intersection acceleration
pub struct BVH {
    components: Vec<Arc<Composable>>,
    nodes: Vec<LinearNode>,
}

impl BVH {
    /// construction from a `Compoable` slice, with `strategy`
    pub fn new(
        components: &[Arc<Composable>], 
        strategy: BVHStrategy
    ) -> BVH {
        let mut arena = Arena::new();
        let mut alloc = arena.allocator();
        let mut cinfo = ComponentInfo::new(&components);
        let mut ordered = cinfo.clone();
        let mut node_count = 0;
        let root = recursive_build(
            &mut alloc, &mut cinfo, 0, &mut node_count,
            &mut ordered, strategy
        );
        let nodes = root.flatten(node_count);
        let mut sorted = Vec::with_capacity(components.len());
        for info in ordered {
            sorted.push(Arc::clone(&components[info.idx]));
        }
        BVH{
            components: sorted, nodes
        }
    }
}

impl Composable for BVH {
    fn bbox_parent(&self) -> BBox3f {
        self.nodes[0].bound
    }

    fn intersect_ray(&self, ray: &mut RawRay) -> Option<SurfaceInteraction> {
        let mut stack = vec![0];
        let mut final_ret = None;
        // (origin, inv_dir, dir_is_neg, max_extend)
        let mut ray_cache = BBox3f::construct_ray_cache(ray);
        while let Some(idx) = stack.pop() {
            assert!(idx<self.nodes.len());
            let node = unsafe {self.nodes.get_unchecked(idx)};
            if node.bound.intersect_ray_cached(&ray_cache).is_none() { continue; }
            if node.len > 0 {
                for element in &self.components[node.offset..node.offset+node.len] {
                    let mut iray = ray.clone();
                    let ret = element.intersect_ray(&mut iray);
                    if ray.max_extend() > iray.max_extend() {
                        *ray = iray;
                        ray_cache.3 = ray.max_extend();
                        final_ret = ret;
                    }
                }
            } else {
                assert!(idx+node.offset < self.nodes.len());
                if ray_cache.2[node.split_axis] {
                    stack.push(idx+1);
                    stack.push(idx+node.offset);
                } else {
                    stack.push(idx+node.offset);
                    stack.push(idx+1);
                }
            }
        }
        final_ret
    }

    fn intersection_cost(&self) -> Float {
        // FIXME: this is silly
        (self.nodes.len() as Float).log2()
    }
}

impl From<Naive> for BVH {
    fn from(naive: Naive) -> BVH {
        BVH::new(&naive.elements, BVHStrategy::SAH)
    }
}

#[derive(Copy, Clone)]
struct LinearNode {
    bound: BBox3f,
    /// `len==0` means leaf node
    len: usize,
    /// if leaf, means offset into the components array
    /// if interior, means offset to the second child
    offset: usize,
    split_axis: usize,
}

use std::fmt::{Debug, Formatter, Result};
impl Debug for LinearNode {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let t = if self.len == 0 {
            "Interior"
        } else {
            "Leaf"
        };
        writeln!(
            fmt, 
            "\n{}{{\n\tbound:{:?},\n\tlen:{}, offset:{}, split:{}\n}}", 
            t, self.bound, self.len, self.offset, self.split_axis
        )
    }
}

#[derive(Clone, Copy)]
struct BuildNode<'a> {
    bound: BBox3f,
    childs: Option<(&'a BuildNode<'a>, &'a BuildNode<'a>, usize)>,
    /// if leaf, means offset into the components array
    /// if interior, means offset to the second child in node array
    offset: usize,
    /// if leaf, means length into the components array
    /// if interior, means length in node array
    len: usize,
}

impl<'a> Default for BuildNode<'a> {
    fn default() -> Self {
        unsafe {
            mem::uninitialized()
        }
    }
}

impl<'a> BuildNode<'a> {
    #[inline]
    fn is_leaf(&self) -> bool {
        self.childs.is_none()
    }

    #[inline]
    fn node_length(&self) -> usize {
        if self.is_leaf() {
            1
        } else {
            self.len
        }
    }

    #[inline]
    fn to_leaf(&mut self, offset: usize, len: usize, bound: BBox3f) {
        self.bound = bound;
        self.offset = offset;
        self.len = len;
        self.childs = None;
    }

    #[inline]
    fn to_interior(&mut self,
        child0: &'a BuildNode<'a>, 
        child1: &'a BuildNode<'a>, 
        axis: usize,
    ) {
        self.bound = child0.bound.union(&child1.bound);
        self.childs = Some((child0, child1, axis));
        self.offset = child0.node_length() + 1;
        self.len = child0.node_length() + child1.node_length() + 1;
    }

    fn flatten(&self, total_nodes: usize) -> Vec<LinearNode> {
        let mut ret = Vec::with_capacity(total_nodes);
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if let Some((child0, child1, split)) = node.childs {
                stack.push(child1);
                stack.push(child0);
                ret.push(LinearNode{
                    bound: node.bound,
                    len: 0,
                    offset: node.offset,
                    split_axis: split,
                });
            } else {
                ret.push(LinearNode{
                    bound: node.bound,
                    len: node.len,
                    offset: node.offset,
                    split_axis: 4,
                });
            }
        }
        // println!("ret={:?}", ret);
        ret
    }
}

fn recursive_build<'a>(
    alloc: &mut Allocator<'a>, components: &mut [ComponentInfo], offset: usize,
    node_count: &mut usize, ordered: &mut [ComponentInfo], strategy: BVHStrategy
) -> &'a mut BuildNode<'a> {
    assert!(components.len()==ordered.len());
    assert!(components.len()!=0);
    *node_count += 1;
    let mut ret: &'a mut BuildNode<'a> = alloc.alloc_default();
    if components.len() == 1 { unsafe {
        ret.to_leaf(offset, 1, components.get_unchecked(0).bound);
        *ordered.get_unchecked_mut(0) = *components.get_unchecked(0);
    }} else {
        let (bound, centroid_bound) = {
            let mut b = unsafe {components.get_unchecked(0).bound};
            let mut cb = unsafe {
                BBox3f::new(
                    components.get_unchecked(0).centroid,
                    components.get_unchecked(0).centroid
                )
            };
            for c in &components[1..] {
                b = b.union(&c.bound);
                cb = cb.extend(c.centroid);
            }
            (b, cb)
        };
        let split_axis = centroid_bound.max_extent();
        if centroid_bound.pmin[split_axis] == centroid_bound.pmax[split_axis] {
            ret.to_leaf(offset, components.len(), bound);
        }
        else {
            match strategy {
                BVHStrategy::SAH => {
                    if components.len() <= 4 {
                        ret = recursive_build(
                            alloc, components, offset, 
                            node_count, ordered, BVHStrategy::MidPoint
                        );
                    } else {
                        let inv_area = 1.0 as Float / bound.surface_area();
                        let midpoint = sah_midpoint(
                            components, split_axis, centroid_bound, inv_area
                        );
                        sort_mid(
                            alloc, components, offset, node_count, ordered,
                            strategy, midpoint[split_axis], split_axis, &mut ret, bound
                        );
                    }
                },
                BVHStrategy::MiddleCount => {
                    let mid_idx = components.len() >> 1;
                    handle_tails(
                        alloc, components, offset, node_count, ordered,
                        strategy, mid_idx, split_axis, ret, bound
                    );
                },
                BVHStrategy::MidPoint => {
                    let mid = (
                        centroid_bound.pmax[split_axis]
                         + centroid_bound.pmin[split_axis]
                    )/2.0 as Float;
                    sort_mid(
                        alloc, components, offset, node_count, ordered,
                        strategy, mid, split_axis, &mut ret, bound
                    );
                }
            }
        }
    }
    ret
}

#[derive(Copy, Clone)]
struct Bucket {
    count: usize,
    cost: Float,
    bound: BBox3f,
    initialized: bool,
}

#[inline]
fn partition(
    buckets: &mut [Bucket], component: &ComponentInfo, 
    centroid_lb: Point3f, diagonal: Vector3f, axis: usize
) {
    let dif = component.centroid - centroid_lb;
    let mut idx = (dif[axis]/diagonal[axis]*buckets.len() as Float) as usize;
    if idx == buckets.len() { idx -= 1; }
    let bucket = unsafe { buckets.get_unchecked_mut(idx)};
    if !bucket.initialized {
        bucket.count = 1;
        bucket.cost = component.cost;
        bucket.bound = component.bound;
        bucket.initialized = true;
    } else {
        bucket.count += 1;
        bucket.cost += component.cost;
        bucket.bound = bucket.bound.union(&component.bound);
    }
}

impl Default for Bucket {
    #[inline]
    fn default() -> Bucket {
        Bucket{
            count: 0,
            cost: 0.0 as Float,
            bound: unsafe{mem::uninitialized()},
            initialized: false
        }
    }
}

impl Bucket {
    #[inline]
    fn union(&self, bucket: &Bucket) -> Bucket {
        if !self.initialized {
            *bucket
        } else if !bucket.initialized {
            *self
        } else {
            Bucket{
                count: self.count + bucket.count,
                cost: self.cost + bucket.cost,
                bound: self.bound.union(&bucket.bound),
                initialized: true,
            }
        }
    }
}

fn sah_midpoint(
    components: &[ComponentInfo], split_axis: usize, cb: BBox3f, inv_area: Float
) -> Point3f {
    const BUCKETS: usize = 32;
    let mut buckets = [Bucket::default(); BUCKETS];
    let diagonal = cb.diagonal();
    for component in components.iter() {
        partition(
            &mut buckets, component, 
            cb.pmin, diagonal, split_axis
        );
    }
    let mut accum = [Default::default(); BUCKETS];
    accum[0] = buckets[0];
    let mut accum_rev = [Default::default(); BUCKETS];
    accum_rev[BUCKETS-1] = buckets[BUCKETS-1];
    for i in 1..BUCKETS-1 {
        accum[i] = accum[i-1].union(&buckets[i]);
        accum_rev[BUCKETS-1-i] = accum_rev[BUCKETS-i].union(&buckets[BUCKETS-i]);
    }
    accum_rev[0] = accum_rev[1].union(&buckets[0]);
    let mut boundary_idx = BUCKETS-1;
    let mut min_cost = accum_rev[0].cost;
    for i in 0..BUCKETS-1 {
        let cost = 0.125 as Float + (
            accum[i].cost * accum[i].bound.surface_area()
            + accum_rev[i+1].cost * accum_rev[i+1].bound.surface_area()
        ) * inv_area;
        if cost < min_cost {
            boundary_idx = i;
            min_cost = cost;
        }
    }
    cb.pmin+cb.diagonal()*(
        (boundary_idx+1) as Float / BUCKETS as Float
    )
}

fn sort_mid<'a>(
    alloc: &mut Allocator<'a>, components: &mut [ComponentInfo], offset: usize,
    node_count: &mut usize, ordered: &mut [ComponentInfo], strategy: BVHStrategy,
    mid: Float, split_axis: usize, ret: &mut BuildNode<'a>, bound: BBox3f
) {
    assert!(components.len()==ordered.len());
    // println!("Sorting... mid={}", mid);
    // println!("Before:");
    // print!("\tComponents: {{\n\t\t");
    // for c in components.iter() {
    //     print!("{}, ", c.centroid[split_axis]);
    // }
    // print!("\n}}\tOrdered: {{\n\t\t");
    // for c in ordered.iter() {
    //     print!("{}, ", c.centroid[split_axis]);
    // }
    // println!("\n}}");
    let mut j = ordered.len();
    let mut i = 0;
    unsafe {
        for component in components.iter() {
            if component.centroid[split_axis] < mid {
                *ordered.get_unchecked_mut(i) = *component;
                i += 1;
            } else {
                j -= 1;
                *ordered.get_unchecked_mut(j) = *component;
            }
        }
        assert!(j == i);
    }
    components.copy_from_slice(ordered);
    handle_tails(
        alloc, components, offset, node_count, ordered,
        strategy, i, split_axis, ret, bound
    );
}

#[inline]
fn handle_tails<'a>(
    alloc: &mut Allocator<'a>, components: &mut [ComponentInfo], offset: usize,
    node_count: &mut usize, ordered: &mut [ComponentInfo], strategy: BVHStrategy,
    i: usize, split_axis: usize, ret: &mut BuildNode<'a>, bound: BBox3f
) {
    // if i==0 {
    //     // print!("0 ");
    //     *node_count -= 1;
    //     *ret = *recursive_build(
    //         alloc, &mut components[i..], offset+i,
    //         node_count, &mut ordered[i..], strategy
    //     );
    // } else if i == components.len() {
    //     // print!("1 ");
    //     *node_count -= 1;
    //     *ret = *recursive_build(
    //         alloc, &mut components[0..i], offset,
    //         node_count, &mut ordered[0..i], strategy
    //     );
    if i == 0 || i == components.len() {
        ret.to_leaf(offset, components.len(), bound);
    } else {
        // print!("2 ");
        let child0 = recursive_build(
            alloc, &mut components[0..i], offset,
            node_count, &mut ordered[0..i], strategy
        );
        let child1 = recursive_build(
            alloc, &mut components[i..], offset+i,
            node_count, &mut ordered[i..], strategy
        );
        ret.to_interior(
            child0, child1, split_axis
        );
    }
}