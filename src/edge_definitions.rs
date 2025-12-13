pub enum AnimEdgeDefinition {
    Simple(SimpleEdgeDefinition),
    Output(BlendEdgeDefinition),
}

// This is used to connect between nodes that don't need any special processing
// IE: From your playback/blend to your l2m job or your state machines to your final output
pub enum SimpleEdgeDefinition {}

// This is used mostly to pipe between playbacks and blend jobs.
// Can also be used to pass blend job output to another blend job, or even a state machine's output to a blend job
pub struct BlendEdgeDefinition {
    pub layer: usize,
}

// This is used to do transitions between two state machines.
// Currently uses lerp to blend
// In the future it'll send events.
pub struct TransitionEdgeDefinition {
    pub duration: f32,
}

impl TransitionEdgeDefinition {
    pub fn new() -> Self {
        Self {
            duration: 0.2,
        }
    }
}
