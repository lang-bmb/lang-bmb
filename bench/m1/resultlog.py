"""Results files written as sessions finish, and resumed (backlog INF-16).

A run with `--out FILE` first writes a header line `{"partial": true, ...}`
naming the run (agent, task set, manifest, commit, task ids), then one line per
finished session, flushed to disk. When every task has a row, the file is
rewritten in task order with the summary row last and no header, the same file
an uninterrupted run writes.

Started again on the same FILE, a run whose header matches skips the tasks that
already have a row. A task whose row was written is never run again, so this is
not a retry (ADR-0021 §3). A session cut off before its row was written left no
result and runs once from the start. The summary counts these restarts
(`resumes`), so a report can tell an interrupted run apart. A file with a
summary row is a finished run and is not resumed. A header that does not match
(another agent, manifest, commit or task list) is refused.
"""
import json
import os
import threading


def read(path):
    """The JSON lines of a results file. A last line cut off mid-write (the
    process stopped while appending) is dropped: that session left no row."""
    with open(path) as f:
        lines = [line for line in f.read().split("\n") if line.strip()]
    out = []
    for i, line in enumerate(lines):
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            if i != len(lines) - 1:
                raise
    return out


def status(path):
    """("finished", summary row) or ("partial", header) of a results file, or
    (None, {}) for anything else."""
    lines = read(path)
    if summary := next((r for r in lines if r.get("summary")), None):
        return "finished", summary
    if lines and lines[0].get("partial"):
        return "partial", lines[0]
    return None, {}


class ResultLog:
    def __init__(self, path, run):
        """`run`: the fields that name the run (agent, task_set, manifest,
        commit, tasks). Without `path` rows are only kept in memory."""
        self.path, self.run = path, run
        self.lock = threading.Lock()
        self.rows = {}
        self.resumes = 0
        if not path:
            return
        if os.path.exists(path) and os.path.getsize(path):  # an empty file (`mktemp`) is a new run
            state, head = status(path)
            if state == "finished":
                raise SystemExit(f"{path}: a finished run (a frozen set is run once, ADR-0021)")
            if state != "partial" or {k: head.get(k) for k in run} != run:
                raise SystemExit(f"{path}: not a partial run of {json.dumps(run, ensure_ascii=False)}")
            lines = read(path)
            self.resumes = sum(1 for r in lines if r.get("resumed")) + 1
            self.rows = {r["id"]: r for r in lines if "id" in r}
            # Rewrite without a cut-off last line before appending to it.
            self._write([lines[0]] + lines[1:] + [{"resumed": True}])
        else:
            self._write([{"partial": True, **run}])

    def done(self, task_id):
        return task_id in self.rows

    def add(self, row):
        with self.lock:
            self.rows[row["id"]] = row
            if self.path:
                with open(self.path, "a") as f:
                    f.write(json.dumps(row, ensure_ascii=False) + "\n")
                    f.flush()
                    os.fsync(f.fileno())

    def finish(self, rows, summary):
        """The final file: `rows` in task order, then `summary`."""
        if self.resumes:
            summary["resumes"] = self.resumes
        if self.path:
            self._write(rows + [summary])

    def _write(self, objs):
        tmp = self.path + ".tmp"
        with open(tmp, "w") as f:
            for o in objs:
                f.write(json.dumps(o, ensure_ascii=False) + "\n")
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self.path)
