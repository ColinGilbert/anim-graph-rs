pub enum AnimEdgeDefinition {
    Simple(SimpleEdgeDefinition),
    Output(OutputEdgeDefinition),
}

// This is used to connect between nodes that don't need any special processing
// IE: From your playback/blend to your l2m job or your state machines to your final output
pub enum SimpleEdgeDefinition {}

// This is used mostly to pipe between playbacks and blend jobs.
// Can also be used to pass blend job output to another blend job, or even a state machine's output to a blend job
pub struct OutputEdgeDefinition {
    pub weight: f32,
    pub layer: usize,
    // if duration <= 0.0, then we treat it as permanently on
}