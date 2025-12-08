use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{NodeIndex},
};

use crate::{edge_definitions::AnimEdgeDefinition};

pub enum AnimNodeDefinition {
    Blend(BlendNodeDefinition),
    Condition(ConditionNodeDefinition),
    LocalToModel(LocalToModelNodeDefinition),
    ParamBool(ParamBoolNodeDefinition),
    ParamFloat(ParamFloatNodeDefinition),
    ParamInt(ParamIntNodeDefinition),
    ParamUint(ParamUintNodeDefinition),
    ParamVec3(ParamVec3NodeDefinition),
    Sample(SampleNodeDefinition),
    StateMachine(StateMachineNodeDefinition),
    Transition(TransitionNodeDefinition),
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

// This is used by the graph evaluator whether or not to evaluate the next node. uses the params_bool vec
pub struct ConditionNodeDefinition {
    pub index: usize,
}

impl ConditionNodeDefinition {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

// This node turns local-space bone matrices into model-space matrices.
// It is usually the output node of an animgraph
pub struct LocalToModelNodeDefinition {}

impl LocalToModelNodeDefinition {
    pub fn new() -> Self {
        Self {}
    }
}

// These parameter nodes are used during animation graph evaluation to kick off (and forcibly end) transitions
pub struct ParamBoolNodeDefinition {
    pub idx: usize,
}

impl ParamBoolNodeDefinition {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamFloatNodeDefinition {
    pub idx: usize,
}

impl ParamFloatNodeDefinition {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamIntNodeDefinition {
    pub idx: usize,
}

impl ParamIntNodeDefinition {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamUintNodeDefinition {
    pub idx: usize,
}

impl ParamUintNodeDefinition {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamVec3NodeDefinition {
    pub idx: usize,
}

impl ParamVec3NodeDefinition {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

pub struct SampleNodeDefinition {
    pub seek: f32,
    pub speed: f32,
    pub looping: bool,
}

// This node samples an animation. This is the simplest node and should be used whenever a single animation will be used, as it is the fastest.
impl SampleNodeDefinition {
    pub fn new() -> Self {
        Self {
            seek: 0.0,
            looping: false,
            speed: 1.0,
        }
    }
}

// This is the most complex node type because it manages the current state, does callbacks, and assigns weights to blending jobs.
pub struct StateMachineNodeDefinition {
    pub graph: SlotMapGraph<AnimNodeDefinition, AnimEdgeDefinition>,
    pub start: Option<NodeIndex>,
    pub end: Option<NodeIndex>,
}

impl StateMachineNodeDefinition {
    pub fn new() -> Self {
        Self {
            graph: SlotMapGraph::<AnimNodeDefinition, AnimEdgeDefinition>::default(),
            start: None,
            end: None,
        }
    }
}

// This is used to do transitions between two state machines.
// Currently uses lerp to blend
// In the future it'll send events.
pub struct TransitionNodeDefinition {
    pub weight1: f32,
    pub weight2: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl TransitionNodeDefinition {
    pub fn new() -> Self {
        Self {
            weight1: 1.0,
            weight2: 1.0,
            duration: 0.2,
            elapsed: 0.0,
        }
    }
}
