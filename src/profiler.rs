use std::collections::{HashMap, HashSet};
use std::{thread::sleep, time::Duration};
use std::time::Instant;
use py_spy::StackTrace;
use tree_ds::prelude::{AutomatedId, Node, TraversalStrategy, Tree};

use crate::stack_node::StackNode;

pub(crate) struct Profiler{
    functions: HashSet<String>,
    process:py_spy::PythonSpy,
    tree : Tree<AutomatedId, StackNode>,
}
impl Profiler{
    pub fn new(functions: HashSet<String>, pid: i32) -> Result<Self, anyhow::Error> {
        let config = py_spy::Config::default();
        let process = py_spy::PythonSpy::new(pid, &config)?;
        let mut tree = Tree::<AutomatedId, StackNode>::new(Some("Profiling Output"));
        let node = Some(StackNode::new("Origin".to_string(), Duration::from_secs(0)).expect("Error"));
        let _ = tree.add_node(Node::new_with_auto_id(node), None);
        Ok(Profiler {
            functions,
            process,
            tree,
        })
    }

    fn sample(&mut self, delay_ms: u64) -> Result<(), anyhow::Error>{
        let traces = self.process.get_stack_traces()?;
        let mut pruned_traces = Vec::<Vec<py_spy::Frame>>::new();
        for trace in &traces {
            let mut flag = false;
            for frame in &trace.frames {
                if self.functions.is_empty() || self.functions.contains(&frame.name) {
                    flag = true;
                    break;
                }
            }
            if flag {
                let reversed_frames: Vec<_> = trace.frames.iter().cloned().rev().collect();
                pruned_traces.push(reversed_frames);
            }
        }

        for trace in pruned_traces{
            let mut node = self.tree.get_root_node().unwrap();
            for frame in trace{                
                if let Some(child_id) = node.get_children_ids().iter().find(|&&id| {
                    if let Some(child_node) = self.tree.get_node_by_id(&id){
                        if let Some(val) = child_node.get_value(){
                            val.get_name() == frame.name
                        }
                        else{
                            false
                        }
                    }else{
                        false
                    }
                }){
                    node = self.tree.get_node_by_id(child_id).unwrap();
                    let curr_duration = node.get_value().expect("error").get_duration();
                    let mut x = StackNode::new(node.get_value().expect("error").get_name(), curr_duration).expect("error");
                    x.increment(Duration::from_millis(delay_ms));
                    node.set_value(Some(x));
                }
                else{
                    let n = Some(StackNode::new(frame.name.clone(), Duration::from_secs(0)).expect("Error"));
                    let id = self.tree.add_node(Node::new_with_auto_id(n), Some(&node.get_node_id()));
                    node = self.tree.get_node_by_id(&id.expect("Error")).expect("WHAT IS HAPPENING HERE");
                }


                // if let Some(child_node) = node.get_children_ids().iter().find(|&id| id == ){
                //     node = self.tree.get_node_by_id(child_node).unwrap();

                //     let curr_duration = node.get_value().expect("").get_duration();

                //     let mut x = StackNode::new(node.get_value().expect("error").get_name(), curr_duration).expect("Error");
                //     x.increment(Duration::from_millis(delay_ms));
                //     node.set_value(Some(x));
                // }
                // else{
                //     //Node doesn't exist
                //     let n = Some(StackNode::new(frame.name.clone(), Duration::from_secs(0)).expect("Error"));
                //     let id = self.tree.add_node(Node::new_with_auto_id(n), Some(&node.get_node_id()));
                //     node = self.tree.get_node_by_id(&id.expect("Error")).expect("WHAT IS HAPPENING HERE");
                // }
            }
        }

        Ok(())
    }

    pub fn run_sampling_loop(&mut self, delay_ms: u64) -> Result<(), anyhow::Error>{
        loop {
            match self.sample(delay_ms) {
                Ok(_) => sleep(Duration::from_millis(delay_ms)),
                Err(e) => {
                    eprintln!("Stopped sampling");
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn print_tree(&self){
        println!("{}", self.tree);
    }
}