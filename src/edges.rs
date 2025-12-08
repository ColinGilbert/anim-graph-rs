use crate::nodes::*;
use std::sync::*;
use ozz_animation_rs::*;

pub enum AnimEdge {
    Simple(SimpleEdge),
    Output(OutputEdge),
    Transition(TransitionEdge),
}

// This is used to connect between nodes that don't need any special processing
// IE: From your playback/blend to your l2m job or your state machines to your final output
pub enum SimpleEdge {}

// This is used mostly to pipe between playbacks and blend jobs.
// Can also be used to pass blend job output to another blend job, or even a state machine's output to a blend job
pub struct OutputEdge {
    pub weight: f32,
    pub seek: f32,
    pub speed: f32,
    pub layer: usize,
}

// This edge is used to do transitions between two state machines.
// It is the most complex edge type requiring a its own blend job and many parameters.
// In the future it'll send events.
pub struct TransitionEdge {
    pub blend: BlendNode,
    pub seek1: f32,
    pub seek2: f32,
    pub weight1: f32,
    pub weight2: f32,
    pub speed1: f32,
    pub speed2: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl TransitionEdge {
    pub fn new(skeleton: Arc<Skeleton>) -> Self {
        Self {
            blend: BlendNode::new(skeleton),
            seek1: 0.0,
            seek2: 0.0,
            weight1: 1.0,
            weight2: 1.0,
            speed1: 1.0,
            speed2: 1.0,
            duration: 0.2,
            elapsed: 0.0,
        }
    }
}