use std::{process::Command, thread::sleep, time::Duration};
use std::result::Result::{Ok, Err};
use std::env;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

struct Profiler{
    functions: HashSet<String>,
    process:py_spy::PythonSpy,
    active_timers: HashMap<String, Instant>,
    elapsed_timers: HashMap<String, Duration>
}
impl Profiler{
    fn new(functions: HashSet<String>, pid: i32) -> Result<Self, anyhow::Error> {
        let config = py_spy::Config::default();
        let process = py_spy::PythonSpy::new(pid, &config)?;
        let active_timers  = HashMap::new();
        let elapsed_timers = HashMap::new();
        Ok(Profiler {
            functions,
            process,
            active_timers,
            elapsed_timers
        })
    }

    fn should_track(&self, func_name: &str) -> bool {
        return self.functions.contains(func_name);
    }

    fn sample(&mut self) -> Result<(), anyhow::Error>{
        let traces = self.process.get_stack_traces()?;

        let mut seen_funcs = HashSet::new();
        for trace in &traces {
            for frame in &trace.frames {
                if self.functions.is_empty() || self.functions.contains(&frame.name) {
                    seen_funcs.insert(frame.name.clone());
                }
            }
        }

        for func in &seen_funcs{
            if !self.active_timers.contains_key(func){
                self.active_timers.insert(func.clone(), Instant::now());
            }
        }

        let prev_active: Vec<_> = self.active_timers.keys().cloned().collect();
        for func in prev_active{
            if !seen_funcs.contains(&func) {
                //Stack has been deallocated
                if let Some(start_time) = self.active_timers.remove(&func){
                    let elapsed = start_time.elapsed();
                    *self.elapsed_timers.entry(func.clone()).or_default() += elapsed;
                }
            }
        }

        Ok(())
    }

    fn run_sampling_loop(&mut self, delay_ms: u64) -> Result<(), anyhow::Error>{
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
}

fn print_python_stacks(mut profiler: Profiler) -> Result<(), anyhow::Error> {
    for x in 1..11{
        let traces = profiler.process.get_stack_traces()?;
        for trace in traces{
            let should_print = profiler.functions.is_empty() ||
                                     trace.frames.iter().any(|frame| profiler.should_track(&frame.name));

            if should_print{
                println!("Thread {:#X} ({})", trace.thread_id, trace.status_str());
                for frame in &trace.frames {
                    println!("\t{} ({}:{})", frame.name, frame.filename, frame.line);
                }
            }
        }
        sleep(Duration::from_millis(5));
    }
    Ok(())
}

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


    let mut profiler = Profiler::new(set, pid).unwrap();
    let _ = profiler.run_sampling_loop(5);
    let _ = child.wait();

    println!("{:#?}", profiler.elapsed_timers);
}