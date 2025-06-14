import profiler.profiler
import subprocess

process = subprocess.Popen(["python3", "./test.py"]).pid
p = profiler.profiler.Profiler(set(), process)
p.run_sampling_loop(100)
p.print_tree()
print("hi")