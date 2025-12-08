use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{EdgeIndex, NodeIndex},
};
use ozz_animation_rs::*;
use std::sync::*;

use crate::edges::*;

pub enum AnimNode {
    Blend(BlendNode),
    Condition(ConditionNode),
    LocalToModel(LocalToModelNode),
    ParamBool(ParamBoolNode),
    ParamFloat(ParamFloatNode),
    ParamInt(ParamIntNode),
    ParamUint(ParamUintNode),
    ParamVec3(ParamVec3Node),
    Sample(SampleNode),
    StateMachine(StateMachineNode),
    Transition(TransitionNode),
}

// This node blends multiple animations together.
// Its inputs can be playback nodes, state machine nodes, or other blend nodes.
// Note: In order to sync animations, they must be added to the node explicitly as part of its parameters.
// This is because trying to figure out what the graph should do when this information is spread out across edges and other nodes is a PITA and I'm time-constrained...
pub struct BlendNode {
    pub blend_job: BlendingJobArc,
    pub samplers: Vec<SamplingJobArc>,
    pub looping: Vec<bool>,
    pub seek: Vec<f32>,
    pub speed: Vec<f32>,
    pub finished_anims: Vec<bool>,
    pub finished_blend: bool,
    pub syncing: bool,
}

impl BlendNode {
    pub fn new(skeleton: Arc<Skeleton>, animations: Vec<Arc<Animation>>) -> Self {
        let mut samplers = Vec::<SamplingJobArc>::new();
        let mut blend_job = BlendingJobArc::default();
        blend_job.set_skeleton(skeleton.clone());

        for a in animations {
            let mut sample_job = SamplingJobArc::default();

            sample_job.set_animation(a.clone());

            sample_job.set_context(SamplingContext::new(a.num_tracks()));

            let sample_out = Arc::new(RwLock::new(vec![
                SoaTransform::default();
                skeleton.num_soa_joints()
            ]));

            sample_job.set_output(sample_out.clone());

            samplers.push(sample_job);

            blend_job
                .layers_mut()
                .push(BlendingLayer::new(sample_out.clone()));
            let i = blend_job.layers().len() - 1;
            blend_job.layers_mut()[i].weight = 1.0;
        }

        let looping = vec![false; samplers.len()];
        let seek = vec![0.0; samplers.len()];
        let speed = vec![1.0; samplers.len()];
        let finished_anims = vec![false; samplers.len()];
        let finished_blend = false;

        Self {
            blend_job,
            samplers,
            looping,
            seek,
            speed,
            finished_anims,
            finished_blend,
            syncing: false,
        }
    }

    pub fn update(&mut self, dt: web_time::Duration) {
        for (i, sampler) in self.samplers.iter_mut().enumerate() {
            if !self.syncing {
                let duration = sampler.animation().unwrap().duration();
                self.seek[i] += dt.as_secs_f32() * self.speed[i];
                if self.looping[i] {
                    self.seek[i] %= duration;
                } else {
                    if !(self.seek[i] < duration) {
                        self.seek[i] = duration;
                        self.finished_anims[i] = true;
                    }
                }
                let ratio = self.seek[i] / duration;
                sampler.set_ratio(ratio);
                sampler.run().unwrap();
            } else {
            }
        }

        let mut finished = true;

        for f in self.finished_anims.clone() {
            if !f {
                finished = false;
                break;
            }
        }

        self.finished_blend = finished;

        self.blend_job.run().unwrap();
    }

    pub fn reset(&mut self) {
        let mut i = 0;
        while i < self.seek.len() {
            self.seek[i] = 0.0;
            i += 1;
        }

        i = 0;
        while i < self.speed.len() {
            self.speed[i] = 1.0;
            i += 1;
        }

        i = 0;
        while i < self.finished_anims.len() {
            self.finished_anims[i] = false;
            i += 1;
        }

        self.finished_blend = false;
    }

    // Returns the layer index. This is used for graph-based inputs
    pub fn set_input(&mut self, input: Arc<RwLock<Vec<SoaTransform>>>) -> usize {
        self.blend_job
            .layers_mut()
            .push(BlendingLayer::new(input.clone()));
        let i = self.blend_job.layers().len() - 1;
        self.blend_job.layers_mut()[i].weight = 1.0;

        i
    }
    // Convenience methods
    pub fn set_output(&mut self, output: Arc<RwLock<Vec<SoaTransform>>>) {
        self.blend_job.set_output(output.clone());
    }

    pub fn set_layer_weight(&mut self, index: usize, weight: f32) {
        self.blend_job.layers_mut()[index].weight = weight;
    }
}

// This is used by the graph evaluator whether or not to evaluate the next node
pub struct ConditionNode {
    pub index: usize,
}

impl ConditionNode {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

// This node turns local-space bone matrices into model-space matrices.
// It is usually the output node of an animgraph
pub struct LocalToModelNode {
    pub l2m_job: LocalToModelJobArc,
    pub models: Arc<RwLock<Vec<glam::Mat4>>>,
}

impl LocalToModelNode {
    pub fn new(skeleton: Arc<Skeleton>, locals: Arc<RwLock<Vec<SoaTransform>>>) -> Self {
        let mut o = Self {
            l2m_job: LocalToModelJob::default(),
            models: Arc::new(RwLock::new(vec![
                glam::Mat4::default();
                skeleton.num_joints()
            ])),
        };

        o.l2m_job.set_skeleton(skeleton.clone());
        o.l2m_job.set_input(locals.clone());
        o.l2m_job.set_output(o.models.clone());

        o
    }

    pub fn update(&mut self) {
        self.l2m_job.run().unwrap();
    }
}

// These parameter nodes are used during animation graph evaluation to kick off (and forcibly end) transitions
pub struct ParamBoolNode {
    pub idx: usize,
}

impl ParamBoolNode {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamFloatNode {
    pub idx: usize,
}

impl ParamFloatNode {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamIntNode {
    pub idx: usize,
}

impl ParamIntNode {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamUintNode {
    pub idx: usize,
}

impl ParamUintNode {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}
pub struct ParamVec3Node {
    pub idx: usize,
}

impl ParamVec3Node {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

pub struct SampleNode {
    pub sample_job: SamplingJobArc,
    pub seek: f32,
    pub speed: f32,
    pub looping: bool,
    pub finished: bool,
}

// This node samples an animation. This is the simplest node and should be used whenever a single animation will be used, as it is the fastest.
impl SampleNode {
    pub fn new(skeleton: Arc<Skeleton>, animation: Arc<Animation>) -> Self {
        let mut sample_job = SamplingJobArc::default();

        sample_job.set_animation(animation.clone());

        sample_job.set_context(SamplingContext::new(animation.num_tracks()));

        let sample_out = Arc::new(RwLock::new(vec![
            SoaTransform::default();
            skeleton.num_soa_joints()
        ]));

        sample_job.set_output(sample_out.clone());

        Self {
            sample_job,
            seek: 0.0,
            speed: 1.0,
            looping: false,
            finished: false,
        }
    }

    pub fn update(&mut self, dt: web_time::Duration) {
        let duration = self.sample_job.animation().unwrap().duration();
        self.seek += dt.as_secs_f32() * self.speed;
        if self.looping && !self.finished {
            self.seek %= duration;
        } else {
            if !(self.seek < duration) {
                self.seek = 0.0;
                self.finished = true;
            }
        }
        let ratio = self.seek / duration;
        self.sample_job.set_ratio(ratio);
        self.sample_job.run().unwrap();
    }

    pub fn reset(&mut self) {
        self.finished = false;
        self.seek = 0.0;
        self.speed = 1.0;
    }
}

// This is the most complex node type because it manages the current state, does callbacks, and assigns weights to blending jobs.
pub struct StateMachineNode {
    pub graph: SlotMapGraph<AnimNode, AnimEdge>,
    pub start: Option<NodeIndex>,
    pub end: Option<NodeIndex>,
    pub active_node: Option<NodeIndex>,
    pub active_edge: Option<EdgeIndex>,
    pub on_node: bool,
}

impl StateMachineNode {
    pub fn new() -> Self {
        Self {
            graph: SlotMapGraph::<AnimNode, AnimEdge>::default(),
            start: None,
            end: None,
            active_node: None,
            active_edge: None,
            on_node: true,
        }
    }
}

// This is used to do transitions between two state machines.
// Currently uses lerp to blend
// In the future it'll send events.
pub struct TransitionNode {
    pub blend: BlendNode,
    pub weight1: f32,
    pub weight2: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl TransitionNode {
    pub fn new(skeleton: Arc<Skeleton>) -> Self {
        Self {
            blend: BlendNode::new(skeleton, Vec::new()),
            weight1: 1.0,
            weight2: 1.0,
            duration: 0.2,
            elapsed: 0.0,
        }
    }
}
