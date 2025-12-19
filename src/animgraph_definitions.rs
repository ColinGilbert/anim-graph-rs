// The graph definition is what the user edits (usually via GUI.)
// The graph definition isn't meant to quickly evaluate: Its state is used to construct animgraphs which get evaluated by the game engine.

use std::collections::HashMap;

use crate::node_definitions::*;

use mapgraph::{aliases::SlotMapGraph, map::slotmap::NodeIndex};

pub struct AnimGraphDefinition {
    pub graph: SlotMapGraph<StateMachineNodeDefinition, TransitionNodeDefinition>,
    pub root: Option<NodeIndex>,

    pub bools: Vec<bool>,
    pub floats: Vec<f32>,
    pub uints: Vec<usize>,
    pub ints: Vec<i64>,
    pub vec3s: Vec<glam::Vec3>,

    pub bool_names: HashMap<String, usize>,
    pub float_names: HashMap<String, usize>,
    pub uint_names: HashMap<String, usize>,
    pub int_names: HashMap<String, usize>,
    pub vec3_names: HashMap<String, usize>,

    // pub sampler_nodes: Vec<SamplerNodeDefinition>,
    // pub blend_nodes: Vec<BlendNodeDefinition>,
    // pub state_machine_nodes: Vec<StateMachineNodeDefinition>,
}

impl AnimGraphDefinition {
    pub fn new() -> Self {
        let graph = SlotMapGraph::<StateMachineNodeDefinition, TransitionNodeDefinition>::default();
        Self {
            graph,
            root: None,
            bools: Vec::new(),
            floats: Vec::new(),
            uints: Vec::new(),
            ints: Vec::new(),
            vec3s: Vec::new(),

            bool_names: HashMap::new(),
            float_names: HashMap::new(),
            uint_names: HashMap::new(),
            int_names: HashMap::new(),
            vec3_names: HashMap::new(),

            // sampler_nodes: Vec::new(),
            // blend_nodes: Vec::new(),
            // state_machine_nodes: Vec::new(),
        }
    }
}
