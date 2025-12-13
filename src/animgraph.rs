use std::collections::HashMap;
use std::rc::Rc;

use ozz_animation_rs::*;

use crate::edges::*;
use crate::nodes::*;

use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{NodeIndex, EdgeIndex},
};

// This is the structure for our animation graph.
pub struct AnimGraph {
    pub skeleton: Rc<Skeleton>, // We keep this here, as many of the nodes inside will require a skeleton
    pub graph: SlotMapGraph<StateMachineNode, TransitionEdge>,
    pub root: Option<NodeIndex>, // This should be your character's idle state
    pub current_state_machine: Option<NodeIndex>,
    pub current_transition: Option<EdgeIndex>,
    // The following are the parameters this graph stores.
    pub bools: Vec<bool>,
    pub floats: Vec<f32>,
    pub uints: Vec<usize>,
    pub ints: Vec<i64>,
    pub vec3s: Vec<glam::Vec3>,
    // The following are for the client programmer to build a map of enums to parameters indices.
    pub bool_names: HashMap<String, usize>,
    pub float_names: HashMap<String, usize>,
    pub uint_names: HashMap<String, usize>,
    pub int_names: HashMap<String, usize>,
    pub vec_names: HashMap<String, usize>
}

impl AnimGraph {
    pub fn new(skeleton: Rc<Skeleton>) -> Self {
        let graph = SlotMapGraph::<StateMachineNode, TransitionEdge>::default();
        Self {
            skeleton: skeleton.clone(),
            graph,
            root: None,
            current_state_machine: None,
            current_transition: None,
            bools: Vec::new(),
            floats: Vec::new(),
            uints: Vec::new(),
            ints: Vec::new(),
            vec3s: Vec::new(),
            bool_names: HashMap::new(),
            float_names: HashMap::new(),
            uint_names: HashMap::new(),
            int_names: HashMap::new(),
            vec_names: HashMap::new(),
        }
    }



}
