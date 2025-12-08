use std::sync::*;

use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{EdgeIndex, NodeIndex},
};

use ozz_animation_rs::*;

use crate::edges::*;
use crate::nodes::*;

// This is the root structure for our animation graph.
// 
pub struct AnimGraph {
    pub skeleton: Arc<Skeleton>, // We keep this here as many of the nodes require a skeleton
    pub graph: SlotMapGraph<AnimNode, AnimEdge>,
    pub begin: Option<NodeIndex>,
    pub output: Option<NodeIndex>,
    pub current_node: Option<NodeIndex>,
    pub current_edge: Option<EdgeIndex>,
    pub on_node: bool,
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
            graph: SlotMapGraph::<AnimNode, AnimEdge>::default(),
            begin: None,
            output: None,
            current_node: None,
            current_edge: None,
            on_node: true,
            params_bool: Vec::new(),
            params_float: Vec::new(),
            params_uint: Vec::new(),
            params_int: Vec::new(),
            params_vec3: Vec::new(),
        }
    }
}
