use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use mapgraph::map::slotmap::EdgeIndex;
use ozz_animation_rs::*;

use crate::animgraph_definitions::AnimGraphDefinition;
use crate::edges::*;
use crate::node_definitions::AnimNodeDefinition;
use crate::node_definitions::TransitionNodeDefinition;
use crate::nodes::*;

use mapgraph::{aliases::SlotMapGraph, map::slotmap::NodeIndex};

// This is the structure for our animation graph.
pub struct AnimGraph {
    skeleton: Rc<Skeleton>, // We keep this here, as many of the nodes inside will require a skeleton
    graph: SlotMapGraph<AnimGraphNode, ()>,
    root: Option<NodeIndex>, // This should be your character's idle state
    current_node: Option<NodeIndex>,
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
    pub fn new(skeleton: Rc<Skeleton>) -> Self {
        let graph = SlotMapGraph::<AnimGraphNode, ()>::default();
        let output_node = LocalToModelNode::new(skeleton.clone());
        Self {
            skeleton: skeleton.clone(),
            graph,
            root: None,
            current_node: None,
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
            state_machine_nodes: Vec::new(),
            transition_nodes: Vec::new(),
        }
    }

    pub fn create_from_definition(
        skeleton: Rc<Skeleton>,
        definition: &AnimGraphDefinition,
        animations_by_name: &HashMap<String, Rc<Animation>>,
    ) -> Option<Self> {
        let mut results = AnimGraph::new(skeleton);

        //  Do the params
        results.bools = definition.bools.clone();
        results.bool_names = definition.bool_names.clone();
        results.floats = definition.floats.clone();
        results.float_names = definition.float_names.clone();
        results.uints = definition.uints.clone();
        results.uint_names = definition.uint_names.clone();
        results.ints = definition.ints.clone();
        results.int_names = definition.int_names.clone();
        results.vec3s = definition.vec3s.clone();
        results.vec3_names = definition.vec3_names.clone();

        // Make sure to keep a map of nodes belonging to both graphs for quick reference, as indices are not stable.
        let mut state_machine_defines_to_runtimes = HashMap::<NodeIndex, NodeIndex>::new();
        let mut transition_defines_to_runtimes = HashMap::<EdgeIndex, NodeIndex>::new();
        let mut state_machine_runtimes_to_defines = HashMap::<NodeIndex, NodeIndex>::new();
        let mut transition_runtimes_to_defines = HashMap::<NodeIndex, EdgeIndex>::new();

        // We now extract the definitions graph and turn it into an animgraph...

        match definition.root {
            Some(_) => {}
            None => {
                println!("[AnimGraph] Could not get root node definition.");
                return None;
            }
        }
        let root_state_machine_idx_define = definition.root.unwrap();
        // let root_state_machine_idx_runtime = results.root.unwrap();
        // state_machine_defines_to_runtimes.insert(
        //     root_state_machine_idx_define,
        //     root_state_machine_idx_runtime,
        // );

        // We clear these on each state machine iteration
        let mut anim_node_defines_to_runtimes = HashMap::<NodeIndex, NodeIndex>::new();
        let mut anim_node_runtimes_to_defines = HashMap::<NodeIndex, NodeIndex>::new();

        // Iterate over state machines and transitions of the definition graph and add corresponding ones to the anim graph.
        let mut current_state_machine_idx_define = root_state_machine_idx_define;
        let root_node_runtime_idx = results.create_state_machine();
        state_machine_defines_to_runtimes
            .insert(current_state_machine_idx_define, root_node_runtime_idx);
        state_machine_runtimes_to_defines
            .insert(root_node_runtime_idx, current_state_machine_idx_define);
        let mut finished = false;
        let mut state_machines_to_evaluate = Vec::<NodeIndex>::new();
        let mut anim_nodes_to_evaluate = Vec::<NodeIndex>::new();

        while !finished {
            let mut state_machine_runtime: Option<NodeIndex> = None;
            for (edge_idx, edge_define) in
                definition.graph.outputs(current_state_machine_idx_define)
            {
                //state_machine_runtime = None;
                if !state_machine_defines_to_runtimes.contains_key(&edge_define.to()) {
                    state_machine_runtime = Some(results.create_state_machine());

                    state_machine_runtimes_to_defines
                        .insert(state_machine_runtime.unwrap(), edge_define.to());
                    state_machine_defines_to_runtimes
                        .insert(edge_define.to(), state_machine_runtime.unwrap());
                    state_machines_to_evaluate.push(edge_define.to());
                } else {
                    println!("[AnimGraph] Got the state machine");
                    state_machine_runtime = state_machine_defines_to_runtimes
                        .get(&edge_define.to())
                        .copied();
                }

                if !transition_defines_to_runtimes.contains_key(&edge_idx) {
                    let transition_runtime =
                        results.create_transition_from_definition(edge_define.weight());
                    match transition_runtime {
                        Some(val) => {
                            transition_runtimes_to_defines.insert(val, edge_idx);
                            transition_defines_to_runtimes.insert(edge_idx, val);
                        }
                        None => {
                            println!("[AnimGraph] Could not create transition from definition.");
                            return None;
                        }
                    }
                    // Now we add the edges to the transition
                    let add_from_results = results.graph.add_edge(
                        (),
                        *state_machine_defines_to_runtimes
                            .get(&edge_define.from())
                            .unwrap(),
                        *transition_defines_to_runtimes.get(&edge_idx).unwrap(),
                    );
                    match add_from_results {
                        Ok(_) => {}
                        Err(_) => {
                            println!(
                                "[AnimGraph] Could not add \"from\" edge to transition node, from definition."
                            );
                            return None;
                        }
                    }
                    let add_to_results = results.graph.add_edge(
                        (),
                        *transition_defines_to_runtimes.get(&edge_idx).unwrap(),
                        *state_machine_defines_to_runtimes
                            .get(&edge_define.from())
                            .unwrap(),
                    );

                    match add_to_results {
                        Ok(_) => {}
                        Err(_) => {
                            println!(
                                "[AnimGraph] Could not add \"to\" edge to transition node, from definition."
                            );
                            return None;
                        }
                    }
                }
            }

            // Now we do the anim nodes and connections of the state machine
            let state_machine_definition = definition
                .graph
                .node(current_state_machine_idx_define)
                .unwrap()
                .weight();

            // println!("[AnimGraph] state machine runtime node {:?}", state_machine_runtime.unwrap());

            // Check to find out whether we have a node to use
            match state_machine_runtime {
                Some(_) => {}
                None => {
                    break;
                }
            }

            let state_machine_node = results
                .graph
                .node(state_machine_runtime.unwrap())
                .unwrap()
                .weight();

            match state_machine_node {
                AnimGraphNode::StateMachine(_) => {}
                AnimGraphNode::Transition(_) => {
                    // WTF?
                    println!(
                        "[AnimGraph] Transition found where state machine was expected, from definition."
                    );
                    return None;
                }
            }

            // Now we add and connect the anim nodes
            let mut state_machine_finished_adding_nodes = false;
            let mut anim_node_definition_graph_idx = state_machine_definition.start;
            while !state_machine_finished_adding_nodes {
                let anim_node_definition = state_machine_definition
                    .graph
                    .node(anim_node_definition_graph_idx);

                if anim_node_defines_to_runtimes.contains_key(&anim_node_definition_graph_idx) {
                    let anim_node_runtime_graph_idx = results.create_anim_node_from_definition(
                        state_machine_runtime.unwrap(),
                        // state_machine_pool_idx,
                        anim_node_definition.unwrap().weight(),
                        animations_by_name,
                    );

                    match anim_node_runtime_graph_idx {
                        Some(_) => {}
                        None => {
                            println!("[AnimGraph] Could not add anim node from definition.");
                            return None;
                        }
                    }

                    anim_node_defines_to_runtimes.insert(
                        anim_node_definition_graph_idx,
                        anim_node_runtime_graph_idx.unwrap(),
                    );
                    anim_node_runtimes_to_defines.insert(
                        anim_node_runtime_graph_idx.unwrap(),
                        anim_node_definition_graph_idx,
                    );
                }

                // Now we check for more anim nodes to add via the edges.
                for (_, definition_edge) in state_machine_definition
                    .graph
                    .outputs(anim_node_definition_graph_idx)
                {
                    anim_nodes_to_evaluate.push(definition_edge.to());
                }

                // We now check to see if we stop adding anim nodes and move onto adding the edges between them.
                let last = anim_nodes_to_evaluate.last();
                match last {
                    Some(val) => {
                        anim_node_definition_graph_idx = *val;
                        anim_nodes_to_evaluate.pop();
                    }
                    None => {
                        state_machine_finished_adding_nodes = true;
                        println!("FINISHED ADDING ANIM NODES TO STATE MACHINE");
                    }
                }
            }

            // We add the edges between the anim nodes.
            let mut state_machine_finished_adding_edges = false;
            let mut current_anim_node_definition = definition.root.unwrap(); // This is a temporary value that is guaranteed to change to a valid one. Did this so the compiler would let me do this without using an Option
            //anim_nodes_to_evaluate.push(definition.root.unwrap());
            let mut already_evaluated = HashSet::<NodeIndex>::new();
            while !state_machine_finished_adding_edges {
                // Evaluate current node: Check for edges and add them.
                // Add the other nodes to anim_nodes_to_evaluate if they're not in already_evaluated
                let current_anim_node_results = anim_node_defines_to_runtimes
                    .get(&current_anim_node_definition)
                    .unwrap();
                for (_, edge) in state_machine_definition
                    .graph
                    .outputs(current_anim_node_definition)
                {
                    results.connect_anim_nodes(
                        state_machine_defines_to_runtimes[&current_state_machine_idx_define],
                        *current_anim_node_results,
                        edge.to(),
                    );

                    if !already_evaluated.contains(&edge.to()) {
                        anim_nodes_to_evaluate.push(edge.to());
                    }
                }

                already_evaluated.insert(current_anim_node_definition);
                // We check to see if we're done
                let last = anim_nodes_to_evaluate.last();
                match last {
                    Some(val) => {
                        current_anim_node_definition = *val;
                        anim_nodes_to_evaluate.pop();
                    }
                    None => {
                        state_machine_finished_adding_edges = true;
                        println!("NO MORE ANIM NODES TO EVALUATE FOR STATE MACHINE")
                    }
                }
            }

            anim_node_defines_to_runtimes.clear();
            anim_node_runtimes_to_defines.clear();
            //anim_edges_defines_to_runtimes.clear();
            //anim_edges_runtimes_to_defines.clear();

            // Now now check to find out whether we should stop the loop.
            let last = state_machines_to_evaluate.last();

            match last {
                Some(val) => {
                    current_state_machine_idx_define = *val;
                    state_machines_to_evaluate.pop();
                }
                None => {
                    finished = true;
                    println!("FINISHING ANIM GRAPH CONSTRUCTION");
                }
            }
        }

        // Finally, setup the root node
        match definition.root {
            Some(val) => {
                let root_node = state_machine_defines_to_runtimes[&val];
                results.root = Some(root_node);
                results.current_node = Some(root_node);
            }

            None => {
                println!("[AnimGraph] Invalid root node found in definition");
                return None;
            }
        }

        // For each state machine, extract and replicate the graph of the state machine definition to the anim graph's one.
        Some(results)
    }

    pub fn get_output(&mut self, output: &mut Vec<glam::Mat4>) {
        output.clear();
        self.output_node.l2m_job.run().unwrap();
        for m in self.output_node.models.borrow().iter() {
            output.push(*m);
        }
    }

    pub fn evaluate(&mut self, dt: web_time::Duration) {
        self.evaluate_animgraph_node(self.current_node.unwrap(), dt);
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

    pub fn get_skeleton(&self) -> Rc<Skeleton> {
        self.skeleton.clone()
    }

    //////////////////////////////////////////////////////////
    // This marks the end of externally-available methods
    //////////////////////////////////////////////////////////

    fn create_state_machine(&mut self) -> NodeIndex {
        self.end_nodes.push(EndNode::new(self.skeleton.clone()));
        let end_node_pool_idx = self.end_nodes.len() - 1;

        self.state_machine_nodes
            .push(StateMachineNode::new(end_node_pool_idx));
        let state_machine_pool_idx = self.state_machine_nodes.len() - 1;

        self.state_machine_nodes[state_machine_pool_idx].output =
            self.end_nodes[end_node_pool_idx].output.clone();

        let start_node_graph_idx = self.state_machine_nodes[state_machine_pool_idx]
            .graph
            .add_node(AnimNode::Start);
        let end_node_graph_idx = self.state_machine_nodes[state_machine_pool_idx]
            .graph
            .add_node(AnimNode::End(end_node_pool_idx));

        self.state_machine_nodes[state_machine_pool_idx].start = start_node_graph_idx;
        self.state_machine_nodes[state_machine_pool_idx].end = end_node_graph_idx;

        let result = self
            .graph
            .add_node(AnimGraphNode::StateMachine(state_machine_pool_idx));
        result
    }

    // Returns success status
    fn create_transition(&mut self, duration: f32) -> Option<NodeIndex> {
        let transition = TransitionNode::new(self.skeleton.clone(), duration);
        self.transition_nodes.push(transition);
        let transition_node_pool_idx = self.transition_nodes.len() - 1;
        let transition_node_graph_idx = self
            .graph
            .add_node(AnimGraphNode::Transition(transition_node_pool_idx));

        Some(transition_node_graph_idx)
    }

    fn create_sampler_node(
        &mut self,
        animation: Rc<Animation>,
        state_machine_idx: NodeIndex,
        //parent_node: NodeIndex,
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

                // let _ = self.state_machine_nodes[*val]
                //     .graph
                //     .add_edge(AnimEdge::Simple, parent_node, new_node)
                //     .unwrap();

                Some(new_node)
            }

            AnimGraphNode::Transition(_) => {
                println!("Tried to create a sampler node as a child of a transition. Invalid.");
                None
            }
        }
    }

    fn create_blend_node(
        &mut self,
        samplers: Vec<String>,
        animations_by_name: &HashMap<String, Rc<Animation>>,
        state_machine_idx: NodeIndex,
        // parent_node: NodeIndex,
    ) -> Option<NodeIndex> {
        let mut animations = Vec::<Rc<Animation>>::new();
        for s in samplers {
            animations.push(animations_by_name[&s].clone());
        }
        self.blend_nodes
            .push(BlendNode::new(self.skeleton.clone(), animations));
        let node_idx = self.blend_nodes.len() - 1;
        let animgraph_node = self.graph.node(state_machine_idx).unwrap().weight();
        match animgraph_node {
            AnimGraphNode::StateMachine(val) => {
                let new_node = self.state_machine_nodes[*val]
                    .graph
                    .add_node(AnimNode::Blend(node_idx));

                // let _ = self.state_machine_nodes[*val]
                //     .graph
                //     .add_edge(AnimEdge::Simple, parent_node, new_node)
                //     .unwrap();

                Some(new_node)
            }
            AnimGraphNode::Transition(_) => None,
        }
    }

    fn create_condition_node(
        &mut self,
        state_machine_graph_idx: NodeIndex,
        param_index: usize,
    ) -> Option<NodeIndex> {
        let state_machine_node_idx = self.graph.node(state_machine_graph_idx).unwrap().weight();
        match state_machine_node_idx {
            AnimGraphNode::StateMachine(val) => {
                let node_idx = self.state_machine_nodes[*val]
                    .graph
                    .add_node(AnimNode::Condition(param_index));
                Some(node_idx)
            }
            AnimGraphNode::Transition(_) => {
                println!("[AnimGraph] Cannot create a condition node as a child of a transition");
                None
            }
        }
    }
    //     // fn create_end_node(state_machine_graph_idx: NodeIndex) -> Option<NodeIndex> {}
    // fn create_sampler_node(state_machine_graph_idx: NodeIndex, animation: String, speed: f32) -> Option<NodeIndex> {

    //     None
    // }

    // Returns success status
    fn connect_anim_nodes(
        &mut self,
        state_machine_idx: NodeIndex,
        parent_idx: NodeIndex,
        child_idx: NodeIndex,
    ) -> bool {
        let state_machine_node = self.graph.node_mut(state_machine_idx).unwrap().weight_mut();
        let i: usize;
        match state_machine_node {
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
                    // AnimNode::LocalToModel(_) => {
                    //     return false;
                    // }
                    AnimNode::Sampler(val) => {
                        is_simple_edge = false;
                        let parent_outputs = self.sampler_nodes[*val]
                            .sample_job
                            .output()
                            .unwrap()
                            .clone();
                        let layer = self.blend_nodes[*val].set_input(parent_outputs);
                        let _ = self.state_machine_nodes[i].graph.add_edge(
                            AnimEdge::Blend(layer),
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
                            AnimEdge::Blend(layer),
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

    fn evaluate_animgraph_node(&mut self, node_idx: NodeIndex, dt: web_time::Duration) {
        let node_results = self.graph.node(node_idx).unwrap().weight();
        match node_results {
            AnimGraphNode::StateMachine(val) => {
                self.evaluate_state_machine(node_idx, *val, dt, true);
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
        let mut from = transition_graph_idx; // It won't stay at this value.
        for (i, edge) in froms.into_iter().enumerate() {
            if i > 0 {
                println!("AnimGraph] Warning: Transition with more than one input.");
                break;
            }

            from = edge.1.from();
            let n = self.graph.node(from).unwrap().weight();
            match n {
                AnimGraphNode::StateMachine(val) => {
                    evaluate = true;
                    state_machine_pool_idx = *val;
                    self.transition_nodes[transition_pool_idx]
                        .blend
                        .blend_job
                        .layers_mut()[0]
                        .transform = self.state_machine_nodes[state_machine_pool_idx]
                        .output
                        .clone();
                }
                AnimGraphNode::Transition(_) => {
                    println!("[AnimGraph] Warning: Transitions from transitions aren't supported.");
                    return;
                }
            }
        }

        if evaluate {
            self.evaluate_state_machine(from, state_machine_pool_idx, dt, false);
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
                        .output
                        .clone();
                }
                AnimGraphNode::Transition(_) => {
                    println!("[AnimGraph] Warning: Transitions to transitions aren't supported.");
                    return;
                }
            }
        }

        if evaluate {
            self.evaluate_state_machine(to, state_machine_pool_idx, dt, false);
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
                println!("Ozz error in transition blend job: {}", val);
            }
        }

        // When the transition is over, set the current node to the "to" state machine
        if weight2 >= 1.0 {
            self.current_node = Some(to);
        }
    }

    fn evaluate_state_machine(
        &mut self,
        state_machine_graph_idx: NodeIndex,
        state_machine_pool_idx: usize,
        dt: web_time::Duration,
        final_output: bool,
    ) {
        if final_output {
            self.output_node.l2m_job.set_input(
                self.state_machine_nodes[state_machine_pool_idx]
                    .output
                    .clone(),
            );
        }
        let start = self.state_machine_nodes[state_machine_pool_idx]
            .graph
            .node(self.state_machine_nodes[state_machine_pool_idx].start)
            .unwrap()
            .weight();

        match start {
            AnimNode::Start => {
                for out in self
                    .graph
                    .outputs(self.state_machine_nodes[state_machine_pool_idx].start)
                {
                    self.state_machine_nodes[state_machine_pool_idx]
                        .trackers
                        .insert(out.1.from());
                }
            }
            _ => {
                println!("[AnimGraph] Invalid start node type")
            }
        }

        let mut finished = false;

        let mut nodes_to_evaluate = Vec::<NodeIndex>::new();
        let mut next_nodes = HashSet::<NodeIndex>::new();
        while !finished {
            // For each node...
            // Evaluate the current node and obtain the next set of nodes to track
            for n in &self.state_machine_nodes[state_machine_pool_idx].trackers {
                nodes_to_evaluate.push(*n);
            }
            for n in &nodes_to_evaluate {
                //println!("EVALUATING ANIM NODE {:?}", n);
                self.evaluate_anim_node(
                    state_machine_pool_idx,
                    state_machine_graph_idx,
                    *n,
                    dt,
                    &mut next_nodes,
                );
            }

            //println!("[AnimGraph] Nodes to evaluate {:?}", nodes_to_evaluate);
            // Remove the visited nodes from the current trackers set
            self.state_machine_nodes[state_machine_pool_idx]
                .trackers
                .clear();
            nodes_to_evaluate.clear();

            // Once the current trackers set is empty, add the next set of nodes to track to the current trackers set
            for n in &next_nodes {
               // println!("ADDING NEW NODES TO TRACKERS");
                self.state_machine_nodes[state_machine_pool_idx]
                    .trackers
                    .insert(*n);
            }
            next_nodes.clear();

            if self.state_machine_nodes[state_machine_pool_idx]
                .trackers
                .len()
                == 0
            {
                finished = true;
            }
  
        }
    }

    // The two NodeIndex types refer to different graph instances
    // Possible to-do: Make typesafe
    fn evaluate_anim_node(
        &mut self,
        state_machine_pool_idx: usize,
        #[allow(unused)] state_machine_graph_idx: NodeIndex,
        anim_node_idx: NodeIndex,
        dt: web_time::Duration,
        next_nodes: &mut HashSet<NodeIndex>,
    ) {
        // For each anim node type, update them accordingly
        let anim_node = self.state_machine_nodes[state_machine_pool_idx]
            .graph
            .node(anim_node_idx)
            .unwrap()
            .weight();

            println!("Evaluating anim node {:?}", anim_node_idx);

        match anim_node {
            AnimNode::Blend(val) => {
                self.blend_nodes[*val].update(dt);
                for (_, edge) in self.state_machine_nodes[state_machine_pool_idx]
                    .graph
                    .outputs(anim_node_idx)
                {
                    let to = edge.to();
                    next_nodes.insert(to);
                }
            }
            AnimNode::Condition(val) => {
                if self.bools[*val] {
                    for (_, edge) in self.state_machine_nodes[state_machine_pool_idx]
                        .graph
                        .outputs(anim_node_idx)
                    {
                        let to = edge.to();
                        next_nodes.insert(to);
                    }
                }
            }
            AnimNode::ConditionNot(val) => {
                if !self.bools[*val] {
                    for (_, edge) in self.state_machine_nodes[state_machine_pool_idx]
                        .graph
                        .outputs(anim_node_idx)
                    {
                        let to = edge.to();
                        next_nodes.insert(to);
                    }
                }
            }
            AnimNode::End(_) => {} // Do nothing as this is the end
            AnimNode::Sampler(val) => {
                self.sampler_nodes[*val].update(dt);
                for (_, edge) in self.state_machine_nodes[state_machine_pool_idx]
                    .graph
                    .outputs(anim_node_idx)
                {
                    let to = edge.to();
                    next_nodes.insert(to);
                }
            }
            // Start node is handled in evaluate_state_machine
            AnimNode::Start => {}
        }
    }

    fn create_anim_node_from_definition(
        &mut self,
        state_machine_graph_idx: NodeIndex,
        definition: &AnimNodeDefinition,
        animations_by_name: &HashMap<String, Rc<Animation>>,
    ) -> Option<NodeIndex> {
        match definition {
            AnimNodeDefinition::Blend(val) => {
                let results = self.create_blend_node(
                    val.samplers.clone(),
                    animations_by_name,
                    state_machine_graph_idx,
                );
                results
            }
            AnimNodeDefinition::Condition(val) => {
                let results = self.create_condition_node(state_machine_graph_idx, val.index);
                results
            }
            AnimNodeDefinition::End => {
                println!("[AnimGraph] Warning: User gave end node as definition. Invalid.");
                None
            }
            AnimNodeDefinition::LocalToModel => {
                println!(
                    "[AnimGraph] Warning: User gave local to model node as definition. Invalid."
                );
                None
            }
            AnimNodeDefinition::Sampler(val) => {
                let results = self.create_sampler_node(
                    animations_by_name[&val.animation].clone(),
                    state_machine_graph_idx,
                    val.speed,
                );
                results
            }
            AnimNodeDefinition::Start => {
                println!("[AnimGraph] Warning: User gave start node as definition. Invalid.");
                None
            } // AnimNodeDefinition::StateMachine(val) => {}
        }
    }

    fn create_transition_from_definition(
        &mut self,
        definition: &TransitionNodeDefinition,
    ) -> Option<NodeIndex> {
        let results = self.create_transition(definition.duration);

        results
    }
}
