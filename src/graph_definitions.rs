// The graph definition is what the user edits (usually via GUI.)
// The graph definition isn't meant to quickly evaluate: Its state is used to construct animgraphs which get evaluated by the game engine.

use crate::{edge_definitions::*, node_definitions::*};

use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{NodeIndex},
};

pub struct AnimGraphDefinition {
    pub graph: SlotMapGraph<StateMachineNodeDefinition, TransitionEdgeDefinition>,
    pub root: Option<NodeIndex>,
    pub params_bool: Vec<bool>,
    pub params_float: Vec<f32>,
    pub params_uint: Vec<usize>,
    pub params_int: Vec<i64>,
    pub params_vec3: Vec<glam::Vec3>,
}

impl AnimGraphDefinition {
    pub fn new() -> Self {
        let graph = SlotMapGraph::<StateMachineNodeDefinition, TransitionEdgeDefinition>::default();
        Self {
            graph,
            root: None,
            params_bool: Vec::new(),
            params_float: Vec::new(),
            params_uint: Vec::new(),
            params_int: Vec::new(),
            params_vec3: Vec::new(),
        }
    }
}
