"""Session records (backlog INF-17, ADR-0070).

With `--sessions DIR` a measurement harness writes one gzip'd JSON file per
session, `DIR/<task id>.json.gz`, so a failure can be attributed after the run
(AUTONOMY §3.2): every tool call with its arguments and the full result text
the model got, each model request's stop reason, tokens and visible text (not
the reasoning, which is large and not sent back), and the final workspace as
the files that differ from the seed. Nothing here is shown to the model, and
the result rows are the same with or without it.
"""
import gzip
import json
import os

SKIP_DIRS = {".bmb-cache", "target"}


def tree_files(root):
    """{relative path: text} of the text files under `root`."""
    out = {}
    if not root or not os.path.isdir(root):
        return out
    for d, dirs, files in os.walk(root):
        dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
        for name in sorted(files):
            path = os.path.join(d, name)
            try:
                with open(path, encoding="utf-8") as f:
                    out[os.path.relpath(path, root)] = f.read()
            except (UnicodeDecodeError, OSError):
                continue
    return out


def workspace_changes(seed, ws):
    """The files of `ws` that are new or differ from `seed`, and the removed ones."""
    before, after = tree_files(seed), tree_files(ws)
    return {"changed": {p: t for p, t in after.items() if before.get(p) != t},
            "removed": sorted(set(before) - set(after))}


def record(task_id, agent_kind, agent, calls, workspace):
    return {"id": task_id, "agent": agent_kind, "calls": calls,
            "requests": list(getattr(agent, "requests", [])), "workspace": workspace}


def write(directory, rec):
    """Writes `rec` to `directory/<id>.json.gz` (byte-stable: no gzip time)."""
    os.makedirs(directory, exist_ok=True)
    data = json.dumps(rec, ensure_ascii=False, sort_keys=True).encode()
    with open(os.path.join(directory, rec["id"] + ".json.gz"), "wb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as f:
            f.write(data)


def read(path):
    with gzip.open(path, "rt", encoding="utf-8") as f:
        return json.load(f)
