from subprocess import Popen, PIPE
from streamreader import LineStreamReader
import time
import sys

example_args = ["stdbuf", "-oL", "java", "-jar", "server-1.15.1.jar", "nogui"]
cwd = "/tmp/tmpserver"

p = Popen(example_args, stdout=PIPE, stdin=PIPE, stderr=PIPE, cwd=cwd)

stdout = LineStreamReader(p.stdout)
stdin = LineStreamReader(sys.stdin)

while p.poll() == None:
    line = stdout.getLine()
    if line != None:
        print(line.decode().strip())
    inp = stdin.getLine()
    if inp != None:
        for i in inp:
            if i == '\x1b':
                print("Detached from server")
                pass
        p.stdin.write(inp.encode())
        p.stdin.flush()
print("Server Terminated")
