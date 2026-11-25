#!/usr/bin/env python3
"""The differential harness against stand-ins: a program
compared with itself is the same on every channel, and a stand-in that
differs in one byte of one channel is caught on that channel. The parts the
gate runs as concurrent jobs hold every case of the port exactly once."""
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
CASES = os.path.join(os.path.dirname(HERE), "tests", "port", "cases.json")
sys.path.insert(0, HERE)
import difftest  # noqa: E402

BASE = """#!/bin/sh
cat > copy.txt
printf 'out %s\\n' "$1"
printf 'err\\n' >&2
exit 3
"""


def script(d, name, text):
    p = os.path.join(d, name)
    with open(p, "w") as f:
        f.write(text)
    os.chmod(p, os.stat(p).st_mode | stat.S_IXUSR)
    return p


class Difftest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = self.tmp.name
        self.base = script(self.d, "base", BASE)

    def tearDown(self):
        self.tmp.cleanup()

    def compare(self, other):
        a = difftest.observe([self.base, "x"], b"in\n", 30)
        b = difftest.observe([other, "x"], b"in\n", 30)
        return difftest.first_difference(a, b)

    def test_a_program_is_the_same_as_itself(self):
        self.assertIsNone(self.compare(self.base))

    def test_one_byte_of_each_channel_is_caught(self):
        for what, old, new in [
            ("stdout", "'out %s\\n'", "'ouT %s\\n'"),
            ("stderr", "'err\\n'", "'erR\\n'"),
            ("exit", "exit 3", "exit 4"),
            ("file", "cat > copy.txt", "cat | tr i I > copy.txt"),
            ("file", "cat > copy.txt", "cat > copy.txt; : > more.txt"),
        ]:
            self.assertIn(old, BASE)
            other = script(self.d, "other", BASE.replace(old, new))
            diff = self.compare(other)
            self.assertIsNotNone(diff, what)
            self.assertEqual(diff["what"], what)

    def test_a_missing_program_differs(self):
        diff = self.compare(os.path.join(self.d, "absent"))
        self.assertEqual(diff["what"], "exit")

    def test_stdout_difference_names_the_line(self):
        a = {"stdout": b"a\nb\nc\n", "stderr": b"", "exit": 0, "files": {}}
        b = {"stdout": b"a\nb\nC\n", "stderr": b"", "exit": 0, "files": {}}
        diff = difftest.first_difference(a, b)
        self.assertEqual((diff["byte"], diff["line"]), (4, 3))
        short = dict(b, stdout=b"a\n")
        self.assertEqual(difftest.first_difference(a, short)["byte"], 2)

    def test_argv_placeholders(self):
        argv = difftest.argv_of(
            ["{build:p}", "{bmb}", "-x", "{inputs}"], ["f", "g"], {"p": "/b/p"}, "/t/bmb"
        )
        self.assertEqual(argv, ["/b/p", "/t/bmb", "-x", "f", "g"])
        rooted = difftest.argv_of(["{root}/scripts/x.py"], [], {}, "/t/bmb")
        self.assertEqual(rooted, [os.path.join(difftest.ROOT, "scripts/x.py")])

    def test_input_sets(self):
        runs, n = difftest.runs_of(
            {"files": ["scripts/check.sh", "scripts/ship.sh"], "except": ["scripts/ship*"], "chunk": 1}
        )
        self.assertEqual((n, len(runs)), (1, 1))
        self.assertTrue(runs[0][1][0].endswith("scripts/check.sh"))
        runs, n = difftest.runs_of({"files": ["scripts/check.sh", "scripts/ship.sh"], "chunk": 1})
        self.assertEqual((len(runs), n), (2, 2))
        self.assertTrue(all(os.path.isabs(f) for _, fs, _ in runs for f in fs))
        runs, n = difftest.runs_of({"stdin_lines_of": ["scripts/toolchain.env"]})
        self.assertEqual(len(runs), 1)
        self.assertEqual(n, runs[0][2].count(b"\n"))
        runs, _ = difftest.runs_of({"none": True})
        self.assertEqual(runs, [("-", [], b"")])
        with self.assertRaises(ValueError):
            difftest.runs_of({"other": 1})

    def test_builds(self):
        # A `bmb` build passes the files of its globs but those left out, in
        # path order; an example is built by cargo next to the compiler
        # under test, in its profile, and copied.
        where = os.path.join(self.d, "release")
        os.makedirs(os.path.join(where, "examples"))
        bmb = script(where, "bmb", '#!/bin/sh\nfor a in "$@"; do last=$a; done\nprintf \'%s\\n\' "$@" > "$last"\n')
        out = os.path.join(self.d, "built")
        spec = {"bmb": ["scripts/difftest.py", "scripts/test_difftest*.py"], "except": ["scripts/difftest.py"]}
        self.assertIsNone(difftest.build(spec, out, bmb))
        with open(out) as f:
            args = f.read().split()
        self.assertEqual(args[0], "build")
        self.assertEqual([a for a in args if a.endswith(".py")], [os.path.join(HERE, "test_difftest.py")])
        self.assertIn("no file matches", difftest.build({"bmb": "scripts/absent-*.bmb"}, out, bmb))
        self.assertIn("unknown build", difftest.build({"other": 1}, out, bmb))
        bin_dir = os.path.join(self.d, "bin")
        os.makedirs(bin_dir)
        made = os.path.join(where, "examples", "ref")
        script(bin_dir, "cargo", f'#!/bin/sh\nprintf \'%s \' "$@" > "{made}"\n')
        path = os.environ["PATH"]
        os.environ["PATH"] = bin_dir + os.pathsep + path
        try:
            self.assertIsNone(difftest.build({"example": "ref"}, out, bmb))
            with open(out) as f:
                self.assertEqual(f.read().split(), ["build", "--quiet", "--example", "ref", "--release"])
            os.remove(made)
            script(bin_dir, "cargo", "#!/bin/sh\necho broken >&2\nexit 101\n")
            self.assertIn("broken", difftest.build({"example": "ref"}, out, bmb))
        finally:
            os.environ["PATH"] = path

    def test_run_reports_the_case_that_differs(self):
        other = script(self.d, "other", BASE.replace("exit 3", "exit 4"))
        cases = {
            "cases": [
                {"name": "same", "a": [self.base], "b": [self.base], "inputs": {"none": True}},
                {"name": "differs", "a": [self.base], "b": [other], "inputs": {"none": True}},
            ]
        }
        path = os.path.join(self.d, "cases.json")
        with open(path, "w") as f:
            json.dump(cases, f)
        cmd = [sys.executable, os.path.join(HERE, "difftest.py"), "run", path]
        r = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)
        lines = [json.loads(l) for l in r.stdout.splitlines()]
        self.assertEqual([l["same"] for l in lines], [True, False])
        self.assertEqual(lines[1]["difference"]["what"], "exit")
        r = subprocess.run(cmd + ["--case", "same"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0)
        r = subprocess.run(cmd + ["--case", "absent"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)

    def write_cases(self, cases, costs=None):
        path = os.path.join(self.d, "cases.json")
        with open(path, "w") as f:
            json.dump(cases, f)
        if costs is not None:
            with open(os.path.join(self.d, "costs.json"), "w") as f:
                json.dump({"seconds": costs}, f)
        return path

    def test_shards_are_disjoint_whole_and_balanced(self):
        cases = [{"name": n} for n in "abcdefg"]
        costs = {"a": 10, "b": 1, "c": 6, "d": 5, "e": 4, "f": 3}  # g counts as the mean, 29/6
        parts, loads = difftest.shards(cases, costs, 3)
        self.assertEqual(sorted(i for p in parts for i in p), list(range(7)))
        self.assertEqual(parts, [[0, 1], [2, 4], [3, 5, 6]])
        self.assertEqual([round(x, 2) for x in loads], [11, 10, 12.83])
        self.assertEqual(difftest.shards(cases, costs, 3), (parts, loads))
        self.assertEqual(difftest.shards(cases, {}, 1)[0], [list(range(7))])
        self.assertEqual(difftest.shards(cases, {}, 9)[0][7:], [[], []])
        with self.assertRaises(ValueError):
            difftest.shards(cases, costs, 0)
        self.assertEqual(difftest.parse_shard("2/3"), (2, 3))
        for bad in ("0/3", "4/3", "3", "a/b", "-1/2"):
            with self.assertRaises(ValueError):
                difftest.parse_shard(bad)

    def test_run_takes_a_shard_and_built_programs(self):
        other = script(self.d, "other", BASE.replace("exit 3", "exit 4"))
        cases = {
            "builds": {"p": {"rustc": "absent.rs"}},
            "cases": [
                {"name": "built", "a": [self.base], "b": ["{build:p}"], "inputs": {"none": True}},
                {"name": "differs", "a": [self.base], "b": [other], "inputs": {"none": True}},
                {"name": "same", "a": [self.base], "b": [self.base], "inputs": {"none": True}},
            ],
        }
        path = self.write_cases(cases, {"built": 1, "differs": 5, "same": 1})
        cmd = [sys.executable, os.path.join(HERE, "difftest.py"), "run", path]
        names = lambda out: [json.loads(l).get("case", json.loads(l).get("build")) for l in out.splitlines()]
        # part 1 holds the costly case alone; part 2 the two others
        r = subprocess.run(cmd + ["--shard", "1/2"], capture_output=True, text=True)
        self.assertEqual((r.returncode, names(r.stdout)), (1, ["differs"]))
        built = os.path.join(self.d, "built")
        os.makedirs(built)
        r = subprocess.run(cmd + ["--shard", "2/2", "--built", built], capture_output=True, text=True)
        self.assertEqual((r.returncode, names(r.stdout)), (1, ["p"]))
        self.assertIn("not built", r.stdout)
        with open(os.path.join(built, "p.error"), "w") as f:
            f.write("it broke")
        r = subprocess.run(cmd + ["--shard", "2/2", "--built", built], capture_output=True, text=True)
        self.assertEqual(json.loads(r.stdout)["error"], "it broke")
        os.remove(os.path.join(built, "p.error"))
        script(built, "p", BASE)
        r = subprocess.run(cmd + ["--shard", "2/2", "--built", built], capture_output=True, text=True)
        self.assertEqual((r.returncode, names(r.stdout)), (0, ["built", "same"]))
        self.assertTrue(all("seconds" in json.loads(l) for l in r.stdout.splitlines()))
        r = subprocess.run(cmd + ["--shard", "2/2", "--prebuilt", f"p={other}"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)

    def test_build_leaves_the_error_of_a_failed_build(self):
        cases = {
            "builds": {"p": {"bmb": "scripts/absent-*.bmb"}},
            "cases": [{"name": "c", "a": ["{build:p}"], "b": ["{build:p}"], "inputs": {"none": True}}],
        }
        path = self.write_cases(cases)
        out = os.path.join(self.d, "out")
        cmd = [sys.executable, os.path.join(HERE, "difftest.py"), "build", path, out]
        r = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(r.returncode, 1)
        with open(os.path.join(out, "p.error")) as f:
            self.assertIn("no file matches", f.read())

    def test_costs_are_written_from_runs_over_every_case(self):
        cases = {"cases": [{"name": "x"}, {"name": "y"}]}
        path = self.write_cases(cases)
        rows = os.path.join(self.d, "rows.jsonl")
        with open(rows, "w") as f:
            f.write('{"case": "y", "seconds": 2.5}\n{"build": "p", "error": "e"}\n')
        cmd = [sys.executable, os.path.join(HERE, "difftest.py"), "costs", path, rows]
        self.assertEqual(subprocess.run(cmd, capture_output=True).returncode, 1)
        with open(rows, "a") as f:
            f.write('{"case": "x", "seconds": 0.5}\n')
        self.assertEqual(subprocess.run(cmd, capture_output=True).returncode, 0)
        self.assertEqual(difftest.load_costs(path), {"x": 0.5, "y": 2.5})


BIG = """#!/bin/sh
# 300,000 bytes of numbered lines on stdout, one file of as many, one small
python3 -c "import sys; sys.stdout.write(''.join('%07d\\\\n' % i for i in range(37500)))"
python3 -c "import sys; sys.stdout.write(''.join('%07d\\\\n' % i for i in range(37500)))" > big.txt
echo small > small.txt
"""


class Digests(unittest.TestCase):
    """The comparison of outputs too large to hold (`"digest": true`)."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = self.tmp.name

    def tearDown(self):
        self.tmp.cleanup()

    def pair(self, other_text, base_text=BIG):
        base = script(self.d, "base", base_text)
        other = script(self.d, "other", other_text)
        a = difftest.observe([base], b"", 30, digest=True)
        b = difftest.observe([other], b"", 30, digest=True, against=a)
        return a, b

    def test_a_program_is_the_same_as_itself(self):
        a, b = self.pair(BIG)
        self.assertIsNone(difftest.first_difference(a, b))
        self.assertEqual(a["stdout"].size, 300000)
        self.assertEqual(sorted(a["files"]), ["big.txt", "small.txt"])
        # What is held is the digests and the first bytes, not the output.
        self.assertEqual(len(a["stdout"].head), difftest.HEAD)
        self.assertEqual(len(a["files"]["big.txt"].head), difftest.FILE_HEAD)
        self.assertEqual(len(a["stdout"].blocks), -(-300000 // difftest.BLOCK))
        self.assertIsNone(b["stdout"].excerpt)

    def test_a_difference_is_located_to_its_block_and_line(self):
        # line 20,001 (byte 160,000) of stdout, in the third block
        a, b = self.pair(BIG.replace("range(37500)))\"\n", "range(37500)).replace('0020000', '002000x'))\"\n", 1))
        diff = difftest.first_difference(a, b)
        self.assertEqual(diff["what"], "stdout")
        k = 160000 // difftest.BLOCK
        self.assertEqual((diff["byte_from"], diff["byte_to"]), (k * difftest.BLOCK, (k + 1) * difftest.BLOCK))
        self.assertEqual(diff["line_from"], k * difftest.BLOCK // 8 + 1)
        self.assertNotEqual(*diff["sha256"])
        at, data = b["stdout"].excerpt
        self.assertEqual(at, k * difftest.BLOCK)
        self.assertIn(b"002000x", data)
        self.assertEqual(len(data), difftest.BLOCK)

    def test_a_file_that_differs_or_is_missing_is_named(self):
        a, b = self.pair(BIG.replace("> big.txt", "| head -c 299999 > big.txt"))
        diff = difftest.first_difference(a, b)
        self.assertEqual((diff["what"], diff["name"], diff["size"]), ("file", "big.txt", [300000, 299999]))
        self.assertEqual(diff["byte_from"], 299999 // difftest.BLOCK * difftest.BLOCK)
        a, b = self.pair(BIG.replace("echo small > small.txt", "echo small > small.txt; : > more.txt"))
        self.assertEqual(difftest.first_difference(a, b), {"what": "file", "name": "more.txt", "only": "b"})
        # a shorter stream that ends at a block's edge
        cut = BIG.replace("range(37500)))\"\n", f"range(37500)))\" | head -c {2 * difftest.BLOCK}\n", 1)
        a, b = self.pair(cut)
        diff = difftest.first_difference(a, b)
        self.assertEqual((diff["what"], diff["byte_from"]), ("stdout", 2 * difftest.BLOCK))
        self.assertIsNone(b["stdout"].excerpt)

    def test_excerpts_hold_the_bytes_around_a_difference_not_the_outputs(self):
        a, b = self.pair(BIG.replace("range(37500)))\"\n", "range(37500)).replace('0020000', '002000x'))\"\n", 1))
        diff = difftest.first_difference(a, b)
        out = os.path.join(self.d, "excerpts")
        difftest.write_excerpts(out, diff, a, b)
        names = sorted(os.listdir(out))
        at = diff["byte_from"]
        self.assertEqual(names, ["a.head", "b.from-%d" % at, "b.head", "difference.json"])
        with open(os.path.join(out, "difference.json")) as f:
            self.assertEqual(json.load(f), json.loads(json.dumps(diff)))
        # whole outputs: the window around the byte that differs
        x = {"stdout": b"a" * 100000 + b"x", "stderr": b"", "exit": 0, "files": {}}
        y = dict(x, stdout=b"a" * 100000 + b"y")
        diff = difftest.first_difference(x, y)
        out = os.path.join(self.d, "whole")
        difftest.write_excerpts(out, diff, x, y)
        at = 100000 - difftest.EXCERPT // 2
        with open(os.path.join(out, f"b.from-{at}"), "rb") as f:
            self.assertEqual(f.read()[-1:], b"y")

    def test_both_sides_out_of_time_is_no_agreement(self):
        slow = script(self.d, "slow", "#!/bin/sh\nsleep 5 &\nwait\n")
        for digest in (False, True):
            a = difftest.observe([slow], b"", 0.5, digest)
            b = difftest.observe([slow], b"", 0.5, digest, a if digest else None)
            self.assertEqual(a["exit"], "timeout")
            self.assertLess(a["seconds"], 4)  # its child is ended with it
            self.assertEqual(difftest.first_difference(a, b), {"what": "timeout"})

    def test_a_run_reports_each_side_with_its_memory(self):
        base = script(self.d, "base", BIG)
        cases = {"cases": [{"name": "big", "a": [base], "b": [base], "inputs": {"none": True}, "digest": True,
                            "timeout": 30}]}
        path = os.path.join(self.d, "cases.json")
        with open(path, "w") as f:
            json.dump(cases, f)
        r = subprocess.run([sys.executable, os.path.join(HERE, "difftest.py"), "run", path], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        row = json.loads(r.stdout)
        self.assertTrue(row["same"])
        self.assertEqual(sorted(row["sides"]), ["a", "b"])
        self.assertGreater(row["sides"]["a"]["rss_mb"], 1)  # python3 holds the text

    def test_a_proved_build_is_a_cache_directory(self):
        bmb = script(self.d, "bmb", '#!/bin/sh\nprintf \'%s\\n\' "$@" > "$3/args"\n')
        cache = os.path.join(self.d, "cache")
        os.makedirs(cache)
        with open(os.path.join(cache, "proofs"), "w") as f:
            f.write("p\n")
        out = os.path.join(self.d, "built", "c")
        os.makedirs(os.path.dirname(out))
        self.assertIsNone(difftest.build({"proved": ["scripts/difftest.py"]}, out, bmb, cache))
        with open(os.path.join(out, "args")) as f:
            self.assertEqual(f.read().split(), ["verify", "--cache", out, os.path.join(HERE, "difftest.py")])
        with open(os.path.join(out, "proofs")) as f:
            self.assertEqual(f.read(), "p\n")
        failing = script(self.d, "failing", '#!/bin/sh\necho \'{"def":"x","status":"failed"}\'\nexit 1\n')
        self.assertIn("failed", difftest.build({"proved": ["scripts/difftest.py"]}, out + "2", failing, cache))
        built, errors = difftest.from_built(["c"], os.path.dirname(out))
        self.assertEqual((built, errors), ({"c": out}, []))


class GateParts(unittest.TestCase):
    """scripts/check.sh runs the port's cases as `port_shards` jobs, one per
    part: each case is in exactly one job."""

    def test_every_case_is_in_exactly_one_job(self):
        with open(os.path.join(os.path.dirname(HERE), "scripts", "check.sh")) as f:
            gate = f.read()
        counts = re.findall(r"(?m)^port_shards=(\d+)$", gate)
        self.assertEqual(len(counts), 1, "check.sh sets port_shards once")
        n = int(counts[0])
        runs = re.findall(r"difftest\.py run tests/port/cases\.json[^\n\"]*", gate)
        self.assertEqual(len(runs), 1, f"one command runs the cases, once per part: {runs}")
        self.assertIn("--shard $i/$port_shards", runs[0])
        self.assertRegex(gate, r'for i in \$\(seq 1 "\$port_shards"\); do')
        with open(CASES) as f:
            cases = json.load(f)["cases"]
        parts, _ = difftest.shards(cases, difftest.load_costs(CASES), n)
        seen = {}
        for k, part in enumerate(parts, 1):
            for i in part:
                seen.setdefault(cases[i]["name"], []).append(k)
        names = [c["name"] for c in cases]
        self.assertEqual(len(set(names)), len(names), "case names are unique")
        self.assertEqual([n for n in names if n not in seen], [], "in no job")
        self.assertEqual({n: k for n, k in seen.items() if len(k) > 1}, {}, "in two jobs")
        self.assertTrue(all(parts), "every job has a case")

    def test_costs_name_only_cases(self):
        with open(CASES) as f:
            names = {c["name"] for c in json.load(f)["cases"]}
        stale = sorted(set(difftest.load_costs(CASES)) - names)
        self.assertEqual(stale, [], "costs.json names cases that are gone")


if __name__ == "__main__":
    unittest.main()
