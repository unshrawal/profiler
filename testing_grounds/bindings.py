from profiler import Profiler
import subprocess

print("got here")
process = subprocess.Popen(["python3", ""])
print(process.pid)
p = Profiler(None, process.pid)
print("got here")
p.run_sampling_loop(1000)
print("got here")
print(f"printing tree: {p.print_tree()}")
