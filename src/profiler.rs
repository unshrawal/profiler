use std::collections::{HashMap, HashSet};
use std::{thread::sleep, time::Duration};
use std::time::Instant;
use py_spy::StackTrace;
use tree_ds::prelude::{Node, Tree};
pub(crate) struct Profiler{
    functions: HashSet<String>,
    process:py_spy::PythonSpy,
    tree : Tree<String, Duration>,
}
impl Profiler{
    pub fn new(functions: HashSet<String>, pid: i32) -> Result<Self, anyhow::Error> {
        let config = py_spy::Config::default();
        let process = py_spy::PythonSpy::new(pid, &config)?;
        let mut tree = Tree::<String, Duration>::new(Some("graph"));
        let _ = tree.add_node(Node::new("Origin".to_owned(), Some(Duration::from_secs(0))), None);
        Ok(Profiler {
            functions,
            process,
            tree
        })
    }

    fn sample(&mut self) -> Result<(), anyhow::Error>{
        let traces = self.process.get_stack_traces()?;
        let mut pruned_traces = Vec::<StackTrace>::new();
        for trace in &traces {
            let mut flag = false;
            for frame in &trace.frames {
                if self.functions.is_empty() || self.functions.contains(&frame.name) {
                    flag = true;
                    break;
                }
            }
            if flag {
                pruned_traces.push(trace.clone());
            }
        }

        for trace in pruned_traces{
            let mut node = self.tree.get_root_node().unwrap();
            for frame in trace.frames{
                let frame_name = &frame.name;
                if let Some(child_node) = node.get_children_ids().iter().find(|&id| id == frame_name){
                    node = self.tree.get_node_by_id(child_node).unwrap();
                }
                else{
                    //Node doesn't exist
                    let child_node = Node::new(frame.name.clone(), Some(Duration::from_secs(0)));
                    let _ = self.tree.add_node(child_node, Some(&node.get_node_id()));
                    node = self.tree.get_node_by_id(frame_name).expect("WHAT IS HAPPENING HERE");
                }
            }
        }


        Ok(())
    }

    pub fn run_sampling_loop(&mut self, delay_ms: u64) -> Result<(), anyhow::Error>{
        loop {
            match self.sample() {
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
        println!("{:#?}", self.tree);
    }
}