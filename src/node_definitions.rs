use mapgraph::{aliases::SlotMapGraph, map::slotmap::NodeIndex};

use crate::edge_definitions::AnimEdgeDefinition;

pub enum AnimNodeDefinition {
    Blend(BlendNodeDefinition),
    Condition(ConditionNodeDefinition),
    End,
    LocalToModel,
    Sampler(SamplerNodeDefinition),
    Start,
    StateMachine(StateMachineNodeDefinition),
    // Transition(TransitionNodeDefinition),
}

// This node blends multiple animations together.
// Its inputs can be playback nodes, state machine nodes, or other blend nodes.
// Note: In order to sync animations, they must be added to the node explicitly as part of its parameters.
// This is because trying to figure out what the graph should do when this information is spread out across edges and other nodes is a PITA and I'm time-constrained...
pub struct BlendNodeDefinition {
    pub samplers: Vec<String>,
    pub looping: Vec<bool>,
    pub seek: Vec<f32>,
    pub speed: Vec<f32>,
    pub syncing: bool,
}

impl BlendNodeDefinition {
    pub fn new(animations: &Vec<String>) -> Self {
        let mut samplers: Vec<_> = Vec::new();
        for s in animations {
            samplers.push(s.clone());
        }
        let looping = vec![false; samplers.len()];
        let seek = vec![0.0; samplers.len()];
        let speed = vec![0.0; samplers.len()];

        Self {
            samplers,
            looping,
            seek,
            speed,
            syncing: false,
        }
    }
}

// This is used by the graph evaluator whether or not to evaluate the next node. Indexes into the params_bool vec
pub struct ConditionNodeDefinition {
    pub index: usize,
}

impl ConditionNodeDefinition {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

pub struct ConditionNodeNotDefinition {
    pub index: usize,
}

impl ConditionNodeNotDefinition {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

pub struct SamplerNodeDefinition {
    pub animation: String,
    pub seek: f32,
    pub speed: f32,
    pub looping: bool,
}

// This node samples an animation. This is the simplest node and should be used whenever a single animation will be used, as it is the fastest.
impl SamplerNodeDefinition {
    pub fn new(animation: String) -> Self {
        Self {
            animation,
            seek: 0.0,
            looping: false,
            speed: 1.0,
        }
    }
}

// This is the most complex node type because it manages the current state, does callbacks, and assigns weights to blending jobs.
pub struct StateMachineNodeDefinition {
    pub graph: SlotMapGraph<AnimNodeDefinition, AnimEdgeDefinition>,
    pub start: NodeIndex,
    pub end: NodeIndex,
}

impl StateMachineNodeDefinition {
    pub fn new() -> Self {
        let mut graph = SlotMapGraph::<AnimNodeDefinition, AnimEdgeDefinition>::default();
        let start = graph.add_node(AnimNodeDefinition::Start);
        let end = graph.add_node(AnimNodeDefinition::End);
        Self { graph, start, end }
    }
}

// This is used to do transitions between two state machines.
// Currently uses lerp to blend
// In the future it'll send events.
pub struct TransitionNodeDefinition {
    pub duration: f32,
}

impl TransitionNodeDefinition {
    pub fn new() -> Self {
        Self { duration: 0.2 }
    }
}
