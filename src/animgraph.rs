use std::collections::HashMap;
use std::rc::Rc;

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
    pub graph: SlotMapGraph<StateMachineNode, TransitionEdge>,
    pub root: Option<NodeIndex>, // This should be your character's idle state
    current_state_machine: Option<NodeIndex>,
    current_transition: Option<EdgeIndex>,
    pub output: LocalToModelNode, // This node is special in that it dynamically gets connected to the outputs of whichever state machine or transition is currently being evaluated, while also being used as the results we're looking for.
    // The following are the parameters this graph stores.
    bools: Vec<bool>,
    floats: Vec<f32>,
    uints: Vec<usize>,
    ints: Vec<i64>,
    vec3s: Vec<glam::Vec3>,
    // The following are for the client programmer to build a map of enums to parameters indices.
    bool_names: HashMap<String, usize>,
    float_names: HashMap<String, usize>,
    uint_names: HashMap<String, usize>,
    int_names: HashMap<String, usize>,
    vec3_names: HashMap<String, usize>,
}

impl AnimGraph {
    pub fn new(skeleton: Rc<Skeleton>) -> Self {
        let graph = SlotMapGraph::<StateMachineNode, TransitionEdge>::default();
        let output = LocalToModelNode::new(skeleton.clone());
        Self {
            skeleton: skeleton.clone(),
            graph,
            root: None,
            current_state_machine: None,
            current_transition: None,
            output,
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

    pub fn evaluate() {
        
    }


    fn evaluate_state_machine(&mut self, state_machine_idx: NodeIndex) {

    }
    // The two NodeIndex types refer to different graph instances.
    // Possible to-do: Make typesafe
    fn evaluate_anim_node(&mut self, state_machine_idx: NodeIndex, anim_node_idx: NodeIndex) {

    }

}
