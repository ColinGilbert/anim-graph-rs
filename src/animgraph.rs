use std::sync::*;

use ozz_animation_rs::*;

use crate::edges::*;
use crate::nodes::*;

// This is the root structure for our animation graph.
// 
pub struct AnimGraph {
    pub skeleton: Arc<Skeleton>, // We keep this here as many of the nodes require a skeleton
    pub nodes: Vec<AnimNode>,
    pub edges: Vec<AnimEdge>,
    pub begin_node: Option<usize>,
    pub output_node: Option<usize>,
    pub current_node: Option<usize>,
    pub params_bool: Vec<bool>,
    pub params_float: Vec<f32>,
    pub params_uint: Vec<usize>,
    pub params_int: Vec<i64>,
    pub params_vec3: Vec<glam::Vec3>,
}

impl AnimGraph {
    pub fn new(skeleton: Arc<Skeleton>) -> Self {
        Self {
            skeleton: skeleton.clone(),
            nodes: Vec::new(),
            edges: Vec::new(),
            begin_node: None,
            output_node: None,
            current_node: None,
            params_bool: Vec::new(),
            params_float: Vec::new(),
            params_uint: Vec::new(),
            params_int: Vec::new(),
            params_vec3: Vec::new(),
        }
    }
}
