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

