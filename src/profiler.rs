use core::panic;
use std::collections::{HashSet};
use std::process::Command;
use std::{thread::sleep, time::Duration};
use py_spy::{PythonSpy};
use tree_ds::prelude::{AutomatedId, Node, Tree};

use crate::stack_node::StackNode;
use pyo3::prelude::*;

#[pyclass(unsendable)]
pub(crate) struct Profiler{
    functions: HashSet<String>,
    process:py_spy::PythonSpy,
    tree : Tree<AutomatedId, StackNode>,
}

#[pymethods]
impl Profiler{
    #[new]
    #[pyo3(text_signature = "(functions=None, pid=None)")]
    pub fn new(functions: Option<HashSet<String>>, pid: Option<i32>) -> PyResult<Self> {
        let pid = pid.expect("error");
        let functions = functions.unwrap_or(HashSet::<String>::new());

        let process = retry_profiler_latch(pid);
        let mut tree = Tree::<AutomatedId, StackNode>::new(Some("Profiling Output"));
        let node = Some(StackNode::new("Origin".to_string(), Duration::from_secs(0)).expect("Error"));
        let _ = tree.add_node(Node::new_with_auto_id(node), None);
        Ok(Profiler {
            functions,
            process,
            tree,
        })
    }

    fn sample(&mut self, delay_ms: u64) -> PyResult<()>{
        let traces = self.process.get_stack_traces().expect("error");
        println!("{:#?}", traces);
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
            }
        }
        Ok(())
    }

    #[pyo3(text_signature = "($self, delay_ms)")]
    pub fn run_sampling_loop(&mut self, delay_ms: u64) -> PyResult<()>{
        let mut samples = 0;
        loop {
            match self.sample(delay_ms) {
                Ok(_) => {
                    sleep(Duration::from_millis(delay_ms));
                    samples += 1;
                    let root = self.tree.get_root_node();
                    root.expect("error").set_value(Some(StackNode::new("Origin".to_owned(), Duration::from_millis(samples * delay_ms)).expect("error")));
                }
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

#[pyfunction]
fn spawn_process(filepath: String) -> PyResult<i32>{
    let mut child = Command::new("python3")
                                .arg(filepath)
                                .spawn()
                                .expect("Failed to start process");
    Ok(child.id() as i32)
}

fn retry_profiler_latch(pid: i32) -> PythonSpy{
    sleep(Duration::from_millis(50)); //IMPORTANT
    let config = py_spy::Config::default();
    for _ in 1..5{
        match py_spy::PythonSpy::new(pid, &config) {
            Ok(process) => return process,
            Err(e) => {
                eprintln!("Profiler attach failed (retrying): {}", e);
                sleep(Duration::from_millis(10));
            }
        }
    }
    
    panic!("Failed to link to process after 5 retries");
}

