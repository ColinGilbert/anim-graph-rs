use std::collections::HashMap;
use std::rc::Rc;

use mapgraph::graph::Node;
use ozz_animation_rs::*;

use crate::edges::*;
use crate::nodes::*;

use mapgraph::{
    aliases::SlotMapGraph,
    map::slotmap::{EdgeIndex, NodeIndex},
};

// This is the structure for our animation graph.
pub struct AnimGraph {
    skeleton: Rc<Skeleton>, // We keep this here, as many of the nodes inside will require a skeleton
    pub graph: SlotMapGraph<AnimGraphNode, ()>,
    pub root: NodeIndex, // This should be your character's idle state
    current_node: NodeIndex,
    output_node: LocalToModelNode, // This node is special in that it dynamically gets connected to the outputs of whichever state machine or transition is currently being evaluated, while also being used as the results we're looking for.
    // The following are the parameters this graph stores.
    bools: Vec<bool>,       // To control graph flow along with condition nodes
    floats: Vec<f32>,       // To manually control playback speed, blend layer weights, etc.
    uints: Vec<usize>,      // Reserved
    ints: Vec<i64>,         // Reserved
    vec3s: Vec<glam::Vec3>, // For use with IK nodes (coming soon!)
    // The following are for the client programmer to build a map of enums to parameters indices.
    bool_names: HashMap<String, usize>,
    float_names: HashMap<String, usize>,
    uint_names: HashMap<String, usize>,
    int_names: HashMap<String, usize>,
    vec3_names: HashMap<String, usize>,
    // The following are used to store the heavyweight node types that can't be copied in-memory
    sampler_nodes: Vec<SamplerNode>,
    blend_nodes: Vec<BlendNode>,
    end_nodes: Vec<EndNode>,
    state_machine_nodes: Vec<StateMachineNode>,
    transition_nodes: Vec<TransitionNode>,
}

impl AnimGraph {
    pub fn new(skeleton: Rc<Skeleton>, root: StateMachineNode) -> Self {
        let mut graph = SlotMapGraph::<AnimGraphNode, ()>::default();
        let mut state_machine_nodes = Vec::new();
        state_machine_nodes.push(root);
        let idx = state_machine_nodes.len() - 1;
        let root = graph.add_node(AnimGraphNode::StateMachine(idx));
        let output_node = LocalToModelNode::new(skeleton.clone());
        Self {
            skeleton: skeleton.clone(),
            graph,
            root,
            current_node: root,
            output_node,
            bools: Vec::new(),
            floats: Vec::new(),
            uints: Vec::new(),
            ints: Vec::new(),
            vec3s: Vec::new(),
            bool_names: HashMap::new(),
            float_names: HashMap::new(),
            uint_names: HashMap::new(),
            int_names: HashMap::new(),
            vec3_names: HashMap::new(),
            sampler_nodes: Vec::new(),
            blend_nodes: Vec::new(),
            end_nodes: Vec::new(),
            state_machine_nodes,
            transition_nodes: Vec::new(),
        }
    }

    pub fn create_bool_param(&mut self, value: bool, param: String) -> usize {
        self.bools.push(value);
        let result = self.bools.len() - 1;
        self.bool_names.insert(param, result);

        result
    }

    pub fn create_float_param(&mut self, value: f32, param: String) -> usize {
        self.floats.push(value);
        let result = self.floats.len() - 1;
        self.float_names.insert(param, result);

        result
    }

    pub fn create_uint_param(&mut self, value: usize, param: String) -> usize {
        self.uints.push(value);
        let result = self.uints.len() - 1;
        self.uint_names.insert(param, result);

        result
    }

    pub fn create_int_param(&mut self, value: i64, param: String) -> usize {
        self.ints.push(value);
        let result = self.ints.len() - 1;
        self.int_names.insert(param, result);

        result
    }

    pub fn create_vec3_param(&mut self, value: glam::Vec3, param: String) -> usize {
        self.vec3s.push(value);
        let result = self.vec3s.len() - 1;
        self.vec3_names.insert(param, result);

        result
    }

    pub fn set_bool(&mut self, value: bool, idx: usize) {
        self.bools[idx] = value;
    }

    pub fn set_float(&mut self, value: f32, idx: usize) {
        self.floats[idx] = value;
    }

    pub fn set_uint(&mut self, value: usize, idx: usize) {
        self.uints[idx] = value;
    }

    pub fn set_int(&mut self, value: i64, idx: usize) {
        self.ints[idx] = value;
    }

    pub fn set_vec3(&mut self, value: glam::Vec3, idx: usize) {
        self.vec3s[idx] = value;
    }

    pub fn get_bool_index(&self, name: &String) -> usize {
        let result = self.bool_names[name];
        result
    }

    pub fn get_float_index(&self, name: &String) -> usize {
        let result = self.float_names[name];
        result
    }

    pub fn get_uint_index(&self, name: &String) -> usize {
        let result = self.uint_names[name];
        result
    }

    pub fn get_int_index(&self, name: &String) -> usize {
        let result = self.int_names[name];
        result
    }

    pub fn get_vec3_index(&self, name: &String) -> usize {
        let result = self.vec3_names[name];
        result
    }

    pub fn create_state_machine(&mut self) -> NodeIndex {
        self.end_nodes.push(EndNode::new(self.skeleton.clone()));
        let end_node_idx = self.end_nodes.len() - 1;
        let result = self
            .graph
            .add_node(AnimGraphNode::StateMachine(end_node_idx));
        result
    }

    // Returns success status
    pub fn create_transition(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        duration: f32,
    ) -> Option<NodeIndex> {
        let transition = TransitionNode::new(self.skeleton.clone(), duration);
        self.transition_nodes.push(transition);
        let transition_node_pool_idx = self.transition_nodes.len() - 1;
        let transition_node_graph_idx = self
            .graph
            .add_node(AnimGraphNode::Transition(transition_node_pool_idx));
        let from_results = self.graph.add_edge((), from, transition_node_graph_idx);
        match from_results {
            Ok(_) => {}
            Err(_) => {
                let _ = self.graph.remove_node(transition_node_graph_idx);
                return None;
            }
        }
        let to_results = self.graph.add_edge((), transition_node_graph_idx, to);
        match to_results {
            Ok(_) => {}
            Err(_) => {
                let _ = self.graph.remove_node(transition_node_graph_idx);
                return None;
            }
        }

        Some(transition_node_graph_idx)
    }

    pub fn create_sampler_node(
        &mut self,
        animation: Rc<Animation>,
        state_machine_idx: NodeIndex,
        parent_node: NodeIndex,
        speed: f32,
    ) -> Option<NodeIndex> {
        self.sampler_nodes.push(SamplerNode::new(
            self.skeleton.clone(),
            animation.clone(),
            speed,
        ));
        let sampler_node_idx = self.sampler_nodes.len() - 1;
        let animgraph_node = self.graph.node_mut(state_machine_idx).unwrap().weight_mut();
        match animgraph_node {
            AnimGraphNode::StateMachine(val) => {
                let new_node = self.state_machine_nodes[*val]
                    .graph
                    .add_node(AnimNode::Sampler(sampler_node_idx));

                let _ = self.state_machine_nodes[*val]
                    .graph
                    .add_edge(AnimEdge::Simple, parent_node, new_node)
                    .unwrap();

                Some(new_node)
            }

            AnimGraphNode::Transition(_) => None,
        }
    }

    pub fn create_blend_node(
        &mut self,
        animations: Vec<Rc<Animation>>,
        state_machine_idx: NodeIndex,
        parent_node: NodeIndex,
    ) -> Option<NodeIndex> {
        self.blend_nodes
            .push(BlendNode::new(self.skeleton.clone(), animations));
        let node_idx = self.blend_nodes.len() - 1;
        let animgraph_node = self.graph.node_mut(state_machine_idx).unwrap().weight_mut();
        match animgraph_node {
            AnimGraphNode::StateMachine(val) => {
                let new_node = self.state_machine_nodes[*val]
                    .graph
                    .add_node(AnimNode::Blend(node_idx));

                let _ = self.state_machine_nodes[*val]
                    .graph
                    .add_edge(AnimEdge::Simple, parent_node, new_node)
                    .unwrap();

                Some(new_node)
            }
            AnimGraphNode::Transition(_) => None,
        }
    }

    // Returns success status
    pub fn connect_anim_nodes(
        &mut self,
        node_idx: NodeIndex,
        parent_idx: NodeIndex,
        child_idx: NodeIndex,
    ) -> bool {
        let animgraph_node = self.graph.node_mut(node_idx).unwrap().weight_mut();
        let i: usize;
        match animgraph_node {
            AnimGraphNode::StateMachine(val) => {
                i = *val;
            }
            AnimGraphNode::Transition(_) => {
                return false;
            }
        }
        let parent = self.state_machine_nodes[i]
            .graph
            .node(parent_idx)
            .unwrap()
            .weight();
        let child = self.state_machine_nodes[i]
            .graph
            .node(child_idx)
            .unwrap()
            .weight();
        let mut is_simple_edge = true;
        match child {
            AnimNode::Blend(_) => {
                // Get output of parent node
                match parent {
                    AnimNode::End(_) => {
                        return false;
                    }
                    AnimNode::LocalToModel(_) => {
                        return false;
                    }
                    AnimNode::Sampler(val) => {
                        is_simple_edge = false;
                        let parent_outputs = self.sampler_nodes[*val]
                            .sample_job
                            .output()
                            .unwrap()
                            .clone();
                        let layer = self.blend_nodes[*val].set_input(parent_outputs);
                        let _ = self.state_machine_nodes[i].graph.add_edge(
                            AnimEdge::Blend(BlendEdge::new(layer)),
                            parent_idx,
                            child_idx,
                        );
                    }
                    AnimNode::Blend(val) => {
                        is_simple_edge = false;
                        let parent_outputs = self.sampler_nodes[*val]
                            .sample_job
                            .output()
                            .unwrap()
                            .clone();
                        let layer = self.blend_nodes[*val].set_input(parent_outputs);
                        let _ = self.state_machine_nodes[i].graph.add_edge(
                            AnimEdge::Blend(BlendEdge::new(layer)),
                            parent_idx,
                            child_idx,
                        );
                    }
                    _ => {}
                }
                // let layer_idx = self.blend_nodes[*val].set_input(input);
            }
            AnimNode::Start => {
                return false;
            }
            _ => {}
        }

        if is_simple_edge {
            let _ =
                self.state_machine_nodes[i]
                    .graph
                    .add_edge(AnimEdge::Simple, parent_idx, child_idx);
        }

        true
    }

    pub fn get_output(&mut self, output: &mut Vec<glam::Mat4>) {
        output.clear();
        self.output_node.l2m_job.run().unwrap();
        for m in self.output_node.models.borrow().iter() {
            output.push(*m);
        }
    }

    pub fn evaluate(&mut self, dt: web_time::Duration) {
        self.evaluate_animgraph_node(self.current_node, dt);
    }

    fn evaluate_animgraph_node(&mut self, node_idx: NodeIndex, dt: web_time::Duration) {
        let node_results = self.graph.node(node_idx).unwrap().weight();
        match node_results {
            AnimGraphNode::StateMachine(val) => {
                self.evaluate_state_machine(*val, dt);
            }
            AnimGraphNode::Transition(val) => {
                self.evaluate_transition(*val, node_idx, dt);
            }
        }
    }

    fn evaluate_transition(
        &mut self,
        transition_pool_idx: usize,
        transition_graph_idx: NodeIndex,
        dt: web_time::Duration,
    ) {
        self.output_node.l2m_job.set_input(
            self.transition_nodes[transition_pool_idx]
                .blend
                .blend_job
                .output()
                .unwrap()
                .clone(),
        );
        
        // Pipe the output of "from" and "to" state machines to our blend job, and evaluate them 
        let froms = self.graph.inputs(transition_graph_idx);
        let mut evaluate = false;
        let mut state_machine_pool_idx = 0;

        for (i, edge) in froms.into_iter().enumerate() {
            if i > 0 {
                println!("AnimGraph] Warning: Transition with more than one input.");
                break;
            }

            let from_graph_idx = edge.1.from();
            let n = self.graph.node(from_graph_idx).unwrap().weight();
            match n {
                AnimGraphNode::StateMachine(val) => {
                    evaluate = true;
                    state_machine_pool_idx = *val;
                    self.transition_nodes[transition_pool_idx]
                        .blend
                        .blend_job
                        .layers_mut()[0]
                        .transform = self.state_machine_nodes[state_machine_pool_idx]
                        .outputs
                        .clone();
                }
                AnimGraphNode::Transition(_) => {
                    println!(
                        "[AnimGraph] Warning: Transition to transition edges aren't supported."
                    );
                    return;
                }
            }
        }
    
        if evaluate {
            self.evaluate_state_machine(state_machine_pool_idx, dt);
        } else {
            println!("[AnimGraph] Warning: No \"from\" node present.");
            return;
        }

        let tos = self.graph.outputs(transition_graph_idx);
        evaluate = false;
        state_machine_pool_idx = 0;
        let mut to = transition_graph_idx;

        for (i, edge) in tos.into_iter().enumerate() {
            if i > 0 {
                println!("AnimGraph] Warning: Transition with more than one output.");
                break;
            }
            let to_graph_idx = edge.1.to();
            to = to_graph_idx;
            let n = self.graph.node(to_graph_idx).unwrap().weight();
            match n {
                AnimGraphNode::StateMachine(val) => {
                    evaluate = true;
                    state_machine_pool_idx = *val;
                    self.transition_nodes[transition_pool_idx]
                        .blend
                        .blend_job
                        .layers_mut()[1]
                        .transform = self.state_machine_nodes[state_machine_pool_idx]
                        .outputs
                        .clone();
                }
                AnimGraphNode::Transition(_) => {
                    println!(
                        "[AnimGraph] Warning: Transition to transition edges aren't supported."
                    );
                    return;
                }
            }
        }

        if evaluate {
            self.evaluate_state_machine(state_machine_pool_idx, dt);
        } else {
            println!("[AnimGraph] Warning: No \"to\" node present.");
            return;
        }

        // Calculate the blend weights based on time elapsed
        self.transition_nodes[transition_pool_idx].elapsed =
            self.transition_nodes[transition_pool_idx].elapsed + dt.as_secs_f32();

        let mut weight2 = self.transition_nodes[transition_pool_idx].elapsed
            / self.transition_nodes[transition_pool_idx].duration;
        if weight2 > 1.0 {
            weight2 = 1.0;
        }
        let weight1 = 1.0 - weight2;

        self.transition_nodes[transition_pool_idx]
            .blend
            .blend_job
            .layers_mut()[0]
            .weight = weight1;
        self.transition_nodes[transition_pool_idx]
            .blend
            .blend_job
            .layers_mut()[1]
            .weight = weight2;
        // Run blend job
        let blend_results = self.transition_nodes[transition_pool_idx]
            .blend
            .blend_job
            .run();
        match blend_results {
            Ok(_) => {}
            Err(val) => {
                println!("Ozz error in blend job: {}", val);
            }
        }

        // When the transition is over, set the current node to the "to" state machine
        if weight2 >= 1.0 {
            self.current_node = to;
        }
    }

    fn evaluate_state_machine(&mut self, state_machine_idx: usize, dt: web_time::Duration) {}

    // The two NodeIndex types refer to different graph instances
    // Possible to-do: Make typesafe
    fn evaluate_anim_node(
        &mut self,
        state_machine_idx: NodeIndex,
        anim_node_idx: NodeIndex,
        dt: web_time::Duration,
    ) {
        // For each anim node type, update them accordingly
    }
}
