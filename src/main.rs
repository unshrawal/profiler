mod profiler;
mod stack_node;
use std::result::Result::{Ok, Err};
use std::env;
use std::thread::sleep;
use std::time::Duration;
use anyhow::Error;
use profiler::Profiler;
use std::collections::{HashSet};
use std::process::Command;

fn main(){
    match env::current_dir() {
        Ok(path) => println!("Current working directory: {}", path.display()),
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
    let mut child = Command::new("python3")
                                .arg("../test.py")
                                .spawn()
                                .expect("Failed to start process");
    let pid = child.id() as i32;
    println!("Spawned python process with PID {}", pid);
    let mut set: HashSet<String> = HashSet::new();
    //set.insert("func1".to_string());

    sleep(Duration::from_millis(30));
    let mut profiler = Profiler::new(set, pid).expect("Failed to create profiler");
    let _ = profiler.run_sampling_loop(1);
    let _ = child.wait();

    profiler.print_tree();
}