use ozz_animation_rs::*;
use std::sync::*;
use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{EdgeIndex, NodeIndex},
};

use crate::edges::*;

pub enum AnimNode {
    Blend(BlendNode),
    LocalToModel(LocalToModelNode),
    ParamBool(ParamBoolNode),
    ParamFloat(ParamFloatNode),
    ParamInt(ParamIntNode),
    ParamUint(ParamUintNode),
    ParamVec3(ParamVec3Node),
    Sample(SampleNode),
    StateMachine(StateMachineNode),
}

pub struct BlendNode {
    pub blend_job: BlendingJobArc,
}

impl BlendNode {
    pub fn new(skeleton: Arc<Skeleton>) -> Self {
        let mut blend_job = BlendingJobArc::default();
        blend_job.set_skeleton(skeleton.clone());

        Self { blend_job }
    }

    pub fn update(&mut self) {
        self.blend_job.run().unwrap();
    }

    // Returns the layer index
    pub fn set_input(&mut self, input: Arc<RwLock<Vec<SoaTransform>>>) -> usize {
        self.blend_job
            .layers_mut()
            .push(BlendingLayer::new(input.clone()));
        let i = self.blend_job.layers().len() - 1;
        self.blend_job.layers_mut()[i].weight = 1.0;

        i
    }

    pub fn set_output(&mut self, output: Arc<RwLock<Vec<SoaTransform>>>) {
        self.blend_job.set_output(output.clone());
    }
}

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
}

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
            looping: false,
            speed: 1.0,
        }
    }

    pub fn update(&mut self, dt: web_time::Duration) {
        let duration = self.sample_job.animation().unwrap().duration();
        self.seek += dt.as_secs_f32() * self.speed;
        if self.looping {
            self.seek %= duration;
        } else {
            if !(self.seek < duration) {
                self.seek = 0.0;
            }
        }
        let ratio = self.seek / duration;
        self.sample_job.set_ratio(ratio);
        self.sample_job.run().unwrap();
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

