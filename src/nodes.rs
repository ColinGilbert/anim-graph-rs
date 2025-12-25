use mapgraph::aliases::SlotMapGraph;
use mapgraph::map::slotmap::NodeIndex;
use ozz_animation_rs::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::*;

use crate::edges::AnimEdge;

#[derive(Copy, Clone)]
pub enum AnimNode {
    Blend(usize),
    Condition(usize),
    ConditionNot(usize),
    End(usize),
    //LocalToModel(usize),
    Sampler(usize),
    Start,
    // StateMachine(StateMachineNode),
    // Transition(TransitionNode),
}

pub enum AnimGraphNode {
    StateMachine(usize),
    Transition(usize),
}

// This node blends multiple animations together.
// Inputs can be its embedded animations, sampler nodes, state machine nodes, or other blend nodes.
// Note: In order to sync animations, they must be added to the node explicitly as part of its parameters.
// This is because a sampler node that gets independently evaluated can't be synced with this node.
pub struct BlendNode {
    // The ozz blend job that does the heavy lifting
    pub blend_job: BlendingJobRc,
    // These samplers are for animations that'll be synced together. Optional.
    pub samplers: Vec<SamplingJobRc>,
    // Whether or not a given animation is looping. Optional.
    pub looping: Vec<bool>,
    // The seek points of the animations in the samplers. Optional.
    pub seek: Vec<f32>,
    // The various speeds at which the animations in the samplers are playing. Optional.
    pub speed: Vec<f32>,
    // These parameters are there for when you want to drive the animations yourself and update it in code. Optional.
    // The first generic usize argument refers to the samplers indices.
    // The second one refers to the parameter indices stored in the animgraph. Both are floats.
    pub speed_params: HashMap<usize, usize>,
    pub weight_params: HashMap<usize, usize>,
    // These are here to tell whether or not we should be continuing to blend.
    pub finished_anims: Vec<bool>,
    pub finished_blend: bool,
    // These are for synchronizing between the samplers. Optional.
    pub syncing: bool,
    pub sync_driver: usize,
}

impl BlendNode {
    pub fn new(skeleton: Rc<Skeleton>, animations: Vec<Rc<Animation>>) -> Self {
        let mut samplers = Vec::<SamplingJobRc>::new();
        let mut blend_job = BlendingJobRc::default();
        blend_job.set_skeleton(skeleton.clone());

        for a in animations {
            let mut sample_job = SamplingJobRc::default();

            sample_job.set_animation(a.clone());

            sample_job.set_context(SamplingContext::new(a.num_tracks()));

            let sample_out = Rc::new(RefCell::new(vec![
                SoaTransform::default();
                skeleton.num_soa_joints()
            ]));

            sample_job.set_output(sample_out.clone());

            samplers.push(sample_job);

            blend_job
                .layers_mut()
                .push(BlendingLayer::new(sample_out.clone()));

            let layers_idx = blend_job.layers().len() - 1;

            blend_job.layers_mut()[layers_idx].weight = 1.0;
        }

        let looping = vec![false; samplers.len()];
        let seek = vec![0.0; samplers.len()];
        let speed = vec![1.0; samplers.len()];
        let speed_params = HashMap::<usize, usize>::new();
        let weight_params = HashMap::<usize, usize>::new();
        let finished_anims = vec![false; samplers.len()];
        let finished_blend = false;

        Self {
            blend_job,
            samplers,
            looping,
            seek,
            speed,
            speed_params,
            weight_params,
            finished_anims,
            finished_blend,
            syncing: false,
            sync_driver: 0,
        }
    }

    pub fn update(&mut self, dt: web_time::Duration) {
        if !self.syncing && !self.finished_blend {
            // Not syncing between anims, blend job hasn't finished yet
            for (i, sampler) in self.samplers.iter_mut().enumerate() {
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
            }
        } else if !self.finished_blend {
            // Syncing between anims, blend job hasn't finished yet
            let driver_duration = self.samplers[self.sync_driver]
                .animation()
                .unwrap()
                .duration();
            for (i, sampler) in self.samplers.iter_mut().enumerate() {
                let anim_duration = sampler.animation().unwrap().duration();
                let driver_ratio = driver_duration / anim_duration;

                self.seek[i] += dt.as_secs_f32() * self.speed[i] * driver_ratio;

                if self.looping[i] {
                    self.seek[i] %= anim_duration;
                } else {
                    if !(self.seek[i] < anim_duration) {
                        self.seek[i] = anim_duration;
                        self.finished_anims[i] = true;
                    }
                }

                let ratio = self.seek[i] / anim_duration;

                sampler.set_ratio(ratio);
                sampler.run().unwrap();
            }
        } else {
        } // Nothing to do.

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

    // Returns the input's index. This is useful for graph-based inputs
    // The return value is used to index into the blend job layers, and the following vectors: looping, seek, speed, and finished_anims
    pub fn set_input(&mut self, input: Rc<RefCell<Vec<SoaTransform>>>) -> usize {
        self.blend_job
            .layers_mut()
            .push(BlendingLayer::new(input.clone()));
        let i = self.blend_job.layers().len() - 1;
        self.blend_job.layers_mut()[i].weight = 1.0;

        self.looping.push(false);
        self.seek.push(0.0);
        self.finished_anims.push(false);
        self.speed.push(1.0);

        i
    }
}

// This is used by the graph evaluator whether or not to evaluate the next node.
// It indexes into the bool params vector...
#[derive(Copy, Clone)]
pub struct ConditionNode {
    pub index: usize,
}

impl ConditionNode {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

// THis is the same as a condition node, but is based on the inverse of the indexed boolean
// It indexes into the bool params vector...
#[derive(Copy, Clone)]
pub struct ConditionNodeNot {
    pub index: usize,
}

impl ConditionNodeNot {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

// This is where your state machine ends its execution. State machines are evaluated every frame.
pub struct EndNode {
    pub output: Rc<RefCell<Vec<SoaTransform>>>,
}

impl EndNode {
    pub fn new(skeleton: Rc<Skeleton>) -> Self {
        let output = Rc::new(RefCell::new(vec![
            SoaTransform::default();
            skeleton.num_joints()
        ]));
        Self { output }
    }
}
// This node turns local-space bone matrices into model-space matrices.
// It is usually the output node of an animgraph
pub struct LocalToModelNode {
    pub l2m_job: LocalToModelJobRc,
    pub models: Rc<RefCell<Vec<glam::Mat4>>>,
}

impl LocalToModelNode {
    pub fn new(skeleton: Rc<Skeleton>) -> Self {
        let mut l2m_job = LocalToModelJobRc::default();
        l2m_job.set_skeleton(skeleton.clone());
        let models = Rc::new(RefCell::new(vec![
            glam::Mat4::IDENTITY;
            skeleton.num_joints()
            ]));
        l2m_job.set_output(models.clone());

        Self { l2m_job, models }
    }

    pub fn set_input(&mut self, locals: Rc<RefCell<Vec<SoaTransform>>>) {
        self.l2m_job.clear_input();
        self.l2m_job.set_input(locals.clone());

        // let results = self.l2m_job.validate();
        // println!("LOCAL TO MODEL NODE SET INPUT. VALID: {}", results);
    }
}

pub struct SamplerNode {
    pub sample_job: SamplingJobRc,
    pub seek: f32,
    pub speed: f32,
    pub looping: bool,
    pub finished: bool,
    pub speed_param: Option<usize>,
}

// This node samples an animation. This is the simplest node and should be used whenever a single animation will be used, as it is the fastest.
impl SamplerNode {
    pub fn new(skeleton: Rc<Skeleton>, animation: Rc<Animation>, speed: f32) -> Self {
        let mut sample_job = SamplingJobRc::default();

        sample_job.set_animation(animation.clone());

        sample_job.set_context(SamplingContext::new(animation.num_tracks()));

        let sample_out = Rc::new(RefCell::new(vec![
            SoaTransform::default();
            skeleton.num_soa_joints()
        ]));

        sample_job.set_output(sample_out.clone());

        Self {
            sample_job,
            seek: 0.0,
            speed,
            looping: false,
            finished: false,
            speed_param: None,
        }
    }

    pub fn update(&mut self, dt: web_time::Duration) {
        let duration = self.sample_job.animation().unwrap().duration();
        self.seek += dt.as_secs_f32() * self.speed;
        if self.looping && !self.finished {
            self.seek %= duration;
        } else {
            if self.seek > duration {
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
    pub start: NodeIndex,
    pub end: NodeIndex,
    //pub active_node: NodeIndex,
    pub output: Rc<RefCell<Vec<SoaTransform>>>,
    pub trackers: HashSet<NodeIndex>,
}

impl StateMachineNode {
    pub fn new(end_node_idx: usize) -> Self {
        let mut graph = SlotMapGraph::<AnimNode, AnimEdge>::default();
        let start_idx = graph.add_node(AnimNode::Start);
        let end_idx = graph.add_node(AnimNode::End(end_node_idx));
        let mut trackers = HashSet::new();
        trackers.insert(start_idx);
        Self {
            graph: SlotMapGraph::<AnimNode, AnimEdge>::default(),
            start: start_idx,
            end: end_idx,
            //active_node: start_idx,
            output: Rc::new(RefCell::new(Vec::new())),
            trackers,
        }
    }
}

// This is used to do transitions between two state machines
// Currently uses lerp to blend
// In the future it'll send events
pub struct TransitionNode {
    pub blend: BlendNode,
    pub weight1: f32,
    pub weight2: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl TransitionNode {
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
