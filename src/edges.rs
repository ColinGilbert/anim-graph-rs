
use ozz_animation_rs::Skeleton;
use std::{rc::Rc};
use crate::nodes::BlendNode;

pub enum AnimEdge {
    Simple,
    Blend(BlendEdge),
}

// This is used mostly to pipe between playbacks to blend jobs
// Can also be used to pass blend job output to another blend job
pub struct BlendEdge {
    pub layer: usize, // used by the graph evaluator on the next node's blend job. This info is needed to index into the blend node's layers
}

impl BlendEdge {
    pub fn new(layer: usize) -> Self {
        Self {
            layer,
        }
    }
}

// This is used to do transitions between two state machines
// Currently uses lerp to blend
// In the future it'll send events
pub struct TransitionEdge {
    pub blend: BlendNode,
    pub weight1: f32,
    pub weight2: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl TransitionEdge {
    pub fn new(skeleton: Rc<Skeleton>, duration: f32) -> Self {
        Self {
            blend: BlendNode::new(skeleton, Vec::new()),
            weight1: 1.0,
            weight2: 0.0,
            duration,
            elapsed: 0.0,
        }
    }
}
