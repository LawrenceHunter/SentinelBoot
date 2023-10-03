from os.path import exists
from time import sleep
import subprocess


def run_command(cmd):
    p1 = subprocess.Popen(('echo', cmd), stdout=subprocess.PIPE)
    p2 = subprocess.check_output(["nc", "-U", "qemu-monitor-socket"], stdin=p1.stdout).decode()
    for line in p2.split("\n"):
        if not line.startswith("(qemu)") and not line.startswith("QEMU"):
            print(line)

if __name__ == "__main__":
    while not exists("qemu-monitor-socket"):
        sleep(1)

    run_command("info cpus")
    run_command("info memory_size_summary")
