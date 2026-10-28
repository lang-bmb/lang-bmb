#!/usr/bin/env python3
"""ADR-0059 readability check (H-ID, "가독성 쪽"): the maps of the corpus,
without the `id` field and with hash values masked, are byte for byte the
maps a compiler before ADR-0059 printed.

  python3 dev-docs/experiments/rename-ripple/readability.py OLD_BMB NEW_BMB

OLD_BMB is a build of the commit before T1-10 (`git archive <commit> compiler
Cargo.toml Cargo.lock rust-toolchain.toml scripts | tar -x -C DIR`, then
`cargo build --release` there). Prints the first differing row per file and
a summary line; exits 1 on a difference.
"""
import glob
import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))


def main():
    old, new = sys.argv[1], sys.argv[2]
    files = sorted(glob.glob(ROOT + "/tests/run/*.bmb") + glob.glob(ROOT + "/tests/verify/pass/*.bmb")
                   + glob.glob(ROOT + "/tests/check/ok/*.bmb") + glob.glob(ROOT + "/lib/*.bmb"))
    mask = re.compile(r'"(hash|body|iface)":"sha256:[0-9a-f]+"')
    same = differ = skipped = rows = 0
    for f in files:
        a = subprocess.run([old, "map", "all", f], capture_output=True, text=True)
        b = subprocess.run([new, "map", "all", f], capture_output=True, text=True)
        if a.returncode != 0 or b.returncode != 0:
            skipped += 1
            continue
        without_id = re.sub(r',"id":"sha256:[0-9a-f]+(\.[0-9]+)?"', "", b.stdout)
        x, y = mask.sub(r'"\1":"H"', a.stdout), mask.sub(r'"\1":"H"', without_id)
        rows += len(x.splitlines())
        if x == y:
            same += 1
            continue
        differ += 1
        for l1, l2 in zip(x.splitlines(), y.splitlines()):
            if l1 != l2:
                print(os.path.relpath(f, ROOT), "\n -", l1[:300], "\n +", l2[:300])
                break
    print(f"files {len(files)}, identical {same}, different {differ}, skipped {skipped}, rows {rows}")
    sys.exit(1 if differ else 0)


if __name__ == "__main__":
    main()
