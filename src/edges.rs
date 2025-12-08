pub enum AnimEdge {
    Simple(SimpleEdge),
    Output(OutputEdge),
}

// This is used to connect between nodes that don't need any special processing
// IE: From your playback/blend to your l2m job or your state machines/transitions to your final output
pub enum SimpleEdge {}

// This is used mostly to pipe between playbacks and blend jobs.
// Can also be used to pass blend job output to another blend job, or even a state machine's output to a blend job
pub struct OutputEdge {
    pub layer: usize, // used by the graph evaluator on the next node's blend job 
}