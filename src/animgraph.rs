use std::sync::*;

use ozz_animation_rs::*;

use crate::edges::*;
use crate::nodes::*;

// This is the structure for our animation graph.
pub struct AnimGraph {
    pub skeleton: Arc<Skeleton>, // We keep this here, as many of the nodes require a skeleton
    pub nodes: Vec<AnimNode>,
    pub edges: Vec<AnimEdge>,
    pub start_node: usize,
    pub output_node: usize,
    pub current_node: usize,
    pub params_bool: Vec<bool>,
    pub params_float: Vec<f32>,
    pub params_uint: Vec<usize>,
    pub params_int: Vec<i64>,
    pub params_vec3: Vec<glam::Vec3>,
}

impl AnimGraph {
    pub fn new(skeleton: Arc<Skeleton>, node: AnimNode) -> Self {
        let mut nodes = Vec::new();
        nodes.push(node);
        Self {
            skeleton: skeleton.clone(),
            nodes,
            edges: Vec::new(),
            start_node: 0,
            output_node: 0,
            current_node: 0,
            params_bool: Vec::new(),
            params_float: Vec::new(),
            params_uint: Vec::new(),
            params_int: Vec::new(),
            params_vec3: Vec::new(),
        }
    }
}
