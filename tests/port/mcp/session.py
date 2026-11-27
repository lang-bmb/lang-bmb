#!/usr/bin/env python3
"""Drives `bmb mcp` through the sessions of script files and prints what it
answers.

    session.py BMB0 BMB ROOT SCRIPT..

BMB is the compiler whose server is driven, BMB0 the Rust compiler (both
sides of a case make their workspaces with it), ROOT the repository. Each
script runs in the directory of the run. A script is lines:

    # TEXT            a comment
    start DIR FLAG..  starts `BMB mcp FLAG.. DIR`
    > LINE            sends LINE and prints the one line the server answers
                      (`(no response)` when its output ended instead)
    >* N TEXT         the same with TEXT N times as the line
    - LINE            sends LINE and reads nothing (a notification, an
                      empty line)
    stop              ends the input, waits and prints the exit code and
                      what the server wrote to standard error
    end               the same without standard error (for a server that
                      ends in words of its own)
    sh COMMAND        runs COMMAND with `sh` (`$bmb0`, `$bmb` and `$root` are
                      set) and prints its output and, when it is not 0, its
                      exit code
    file PATH N       writes the next N lines of the script to PATH
    ids DIR           reads the identities of the definitions of DIR (`BMB0
                      hash DIR`); after it `@id{PATH}` in a line to send is
                      the identity of definition PATH
    tree              prints every file under the directory of the run with
                      its SHA-256 (a link with its target)

A response is read only after its request was sent and before the next one
is: a server that held its answers back until its input ended would hang
here. At the end the line that names the compiler build in every
per-definition cache (`.bmb-cache/defs`, line 2) is masked, in `tree` too,
so that the directories of the two sides compare.
"""
import hashlib
import json
import os
import re
import subprocess
import sys


def masked(path):
    data = open(path, "rb").read()
    if path.endswith(os.path.join(".bmb-cache", "defs")):
        lines = data.split(b"\n")
        if len(lines) > 1:
            lines[1] = b"id -"
        data = b"\n".join(lines)
    return data


def tree():
    for d, dirs, files in os.walk("."):
        dirs.sort()
        for name in sorted(dirs + files):
            p = os.path.join(d, name)
            if os.path.islink(p):
                print(f"{p} -> {os.readlink(p)}")
            elif os.path.isdir(p):
                print(f"{p}/")
            else:
                print(f"{p} {hashlib.sha256(masked(p)).hexdigest()}")


def main():
    bmb0, bmb, root = (os.path.abspath(a) for a in sys.argv[1:4])
    env = dict(os.environ, bmb0=bmb0, bmb=bmb, root=root)
    server = None
    errors = None
    ids = {}

    def send(line, answer):
        if server is None:
            print("(no server)")
            return
        text = re.sub(r"@id\{([^}]*)\}", lambda m: ids.get(m.group(1), "sha256:unknown"), line)
        try:
            server.stdin.write(text.encode() + b"\n")
            server.stdin.flush()
        except BrokenPipeError:
            pass
        if answer:
            got = server.stdout.readline()
            sys.stdout.write(got.decode(errors="replace") if got else "(no response)\n")

    for script in sys.argv[4:]:
        print(f"=== {os.path.basename(script)}")
        lines = open(script, encoding="utf-8").read().split("\n")
        k = 0
        while k < len(lines):
            line = lines[k]
            k += 1
            sys.stdout.flush()
            if line.startswith("#") or not line:
                continue
            word, _, rest = line.partition(" ")
            if word == "start":
                args = rest.split()
                print(f"== start {rest}")
                errors = open("stderr.txt", "wb")
                server = subprocess.Popen([bmb, "mcp"] + args[1:] + [args[0]], stdin=subprocess.PIPE,
                                          stdout=subprocess.PIPE, stderr=errors)
            elif word == ">":
                send(rest, True)
            elif word == ">*":
                n, _, text = rest.partition(" ")
                send(text * int(n), True)
            elif word == "-":
                send(rest, False)
            elif word in ("stop", "end"):
                try:
                    server.stdin.close()
                except BrokenPipeError:
                    pass
                more = server.stdout.read()
                if more:
                    sys.stdout.write("more: " + more.decode(errors="replace"))
                print(f"exit {server.wait()}")
                errors.close()
                if word == "stop":
                    sys.stdout.write(open("stderr.txt", "rb").read().decode(errors="replace"))
                os.remove("stderr.txt")
                server = None
            elif word == "sh":
                r = subprocess.run(["sh", "-c", rest], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                sys.stdout.write(r.stdout.decode(errors="replace"))
                if r.returncode != 0:
                    print(f"sh: exit {r.returncode}")
            elif word == "file":
                path, n = rest.rsplit(" ", 1)
                os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
                with open(path, "w", encoding="utf-8") as f:
                    f.write("".join(x + "\n" for x in lines[k:k + int(n)]))
                k += int(n)
            elif word == "ids":
                r = subprocess.run([bmb0, "hash", rest], stdout=subprocess.PIPE)
                for row in r.stdout.decode().splitlines():
                    v = json.loads(row)
                    if "def" in v and "id" in v:
                        ids[v["def"]] = v["id"]
            elif word == "tree":
                tree()
            else:
                print(f"session.py: {script}: unknown line: {line}")
                return 2
    sys.stdout.flush()
    for d, _, files in os.walk("."):
        if "defs" in files and os.path.basename(d) == ".bmb-cache":
            p = os.path.join(d, "defs")
            data = masked(p)
            open(p, "wb").write(data)
    return 0


if __name__ == "__main__":
    sys.exit(main())
