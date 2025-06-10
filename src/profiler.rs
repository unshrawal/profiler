use std::collections::{HashMap, HashSet};
use std::{thread::sleep, time::Duration};
use std::time::Instant;
use py_spy::StackTrace;
use tree_ds::prelude::{Node, TraversalStrategy, Tree};

use crate::stack_node::StackNode;

pub(crate) struct Profiler{
    functions: HashSet<String>,
    process:py_spy::PythonSpy,
    tree : Tree<String, StackNode>,
}
impl Profiler{
    pub fn new(functions: HashSet<String>, pid: i32) -> Result<Self, anyhow::Error> {
        let config = py_spy::Config::default();
        let process = py_spy::PythonSpy::new(pid, &config)?;
        let mut tree = Tree::<String, StackNode>::new(Some("graph"));
        let node = Some(StackNode::new("Origin".to_string(), Duration::from_secs(0)).expect("Error"));
        let _ = tree.add_node(Node::new("Origin".to_string(), node), None);
        Ok(Profiler {
            functions,
            process,
            tree
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
                let frame_name = &frame.name;
                if let Some(child_node) = node.get_children_ids().iter().find(|&id| id == frame_name){
                    node = self.tree.get_node_by_id(child_node).unwrap();

                    let curr_duration = node.get_value().expect("").get_duration();

                    let mut x = StackNode::new(node.get_node_id(), curr_duration).expect("Error");
                    x.increment(Duration::from_millis(delay_ms));
                    node.set_value(Some(x));
                }
                else{
                    //Node doesn't exist
                    let n = Some(StackNode::new(frame.name.clone(), Duration::from_secs(0)).expect("Error"));
                    let child_node = Node::new(frame.name.clone(), n);
                    let _ = self.tree.add_node(child_node, Some(&node.get_node_id()));
                    node = self.tree.get_node_by_id(frame_name).expect("WHAT IS HAPPENING HERE");
                }
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