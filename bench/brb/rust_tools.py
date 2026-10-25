"""Tool server of the Rust arm of BRB (ADR-0034 §2, backlog 2H-03).

The counterpart of `bmb mcp` for the Rust arm: the agent works in a sandbox
crate (`solution`) through these tools only, with no shell. Tools have the
MCP shape (`name`, `description`, `inputSchema`) so the same agent adapter
drives both arms, and `call` returns `(text, is_error)` like the MCP client.

- `list_files`, `read_file`, `write_file`: files of the crate. Writes are
  limited to `.rs` files under `src/` and `tests/`; `Cargo.toml` is fixed (the
  dependencies are the same for every session: none, plus `proptest` as a
  dev-dependency when enabled).
- `cargo_build`: `cargo build --all-targets` (compiler errors, as `verify`
  gives diagnostics in the BMB arm).
- `cargo_test`: `cargo test [FILTER]` (the agent's own tests, the analogue of
  `verify` as the BMB arm's checker).

Cargo always runs `--offline`: the template's dependencies are fetched once
before any session (`harness.py`), so no session reaches the network.

Standard library only.
"""
import os
import subprocess

OUTPUT_LIMIT = 20000  # characters of cargo output returned to the agent
CARGO_TIMEOUT = 600  # seconds per cargo command
WRITE_LIMIT = 200000  # bytes per file


def _schema(props, required):
    return {"type": "object",
            "properties": {k: {"type": t, "description": d} for k, t, d in props},
            "required": required}


TOOLS = [
    {"name": "list_files",
     "description": "List the files of the crate (paths relative to the crate root).",
     "inputSchema": _schema([], [])},
    {"name": "read_file",
     "description": "Read a file of the crate.",
     "inputSchema": _schema([("path", "string", "path relative to the crate root")], ["path"])},
    {"name": "write_file",
     "description": "Create or replace a .rs file under src/ or tests/ with the given content.",
     "inputSchema": _schema([("path", "string", "path relative to the crate root, e.g. src/lib.rs"),
                             ("content", "string", "the complete new file content")], ["path", "content"])},
    {"name": "cargo_build",
     "description": "Run `cargo build --all-targets` (library and tests). Returns the exit status and compiler output.",
     "inputSchema": _schema([], [])},
    {"name": "cargo_test",
     "description": "Run `cargo test` (debug build: integer overflow panics), optionally only tests whose name contains FILTER. Returns the exit status and output.",
     "inputSchema": _schema([("filter", "string", "optional test name filter")], [])},
]


class RustTools:
    def __init__(self, crate, target_dir):
        self.crate = os.path.abspath(crate)
        self.target_dir = target_dir
        self.tools = TOOLS

    def _path(self, rel, write):
        if not isinstance(rel, str) or os.path.isabs(rel) or "\\" in rel:
            return None
        norm = os.path.normpath(rel)
        if norm.startswith("..") or norm.split(os.sep)[0] in ("target", ".git"):
            return None
        if write and not (norm.split(os.sep)[0] in ("src", "tests") and norm.endswith(".rs")):
            return None
        return os.path.join(self.crate, norm)

    def _cargo(self, args):
        env = {**os.environ, "CARGO_TARGET_DIR": self.target_dir, "CARGO_TERM_COLOR": "never",
               "RUST_BACKTRACE": "0", "CARGO_NET_OFFLINE": "true"}
        try:
            r = subprocess.run(["cargo", *args, "--offline"], cwd=self.crate, capture_output=True,
                               text=True, env=env, timeout=CARGO_TIMEOUT)
            code, out = r.returncode, r.stderr + r.stdout
        except subprocess.TimeoutExpired:
            code, out = -1, f"timed out after {CARGO_TIMEOUT} s"
        if len(out) > OUTPUT_LIMIT:
            out = out[:OUTPUT_LIMIT] + f"\n... ({len(out) - OUTPUT_LIMIT} more characters)"
        return f"exit status: {code}\n{out}", code != 0

    def call(self, name, args):
        if name == "list_files":
            files = []
            for root, dirs, fs in os.walk(self.crate):
                dirs[:] = sorted(d for d in dirs if d not in ("target", ".git"))
                files += [os.path.relpath(os.path.join(root, f), self.crate) for f in sorted(fs)]
            return "\n".join(files), False
        if name == "read_file":
            p = self._path(args.get("path"), write=False)
            if p is None or not os.path.isfile(p):
                return f"no such file: {args.get('path')}", True
            with open(p, encoding="utf-8", errors="replace") as f:
                return f.read(), False
        if name == "write_file":
            p = self._path(args.get("path"), write=True)
            content = args.get("content")
            if p is None:
                return "write_file: only .rs files under src/ or tests/ may be written", True
            if not isinstance(content, str) or len(content.encode()) > WRITE_LIMIT:
                return f"write_file: content must be a string of at most {WRITE_LIMIT} bytes", True
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, "w", encoding="utf-8") as f:
                f.write(content)
            return f"wrote {os.path.relpath(p, self.crate)} ({len(content.encode())} bytes)", False
        if name == "cargo_build":
            return self._cargo(["build", "--all-targets"])
        if name == "cargo_test":
            flt = args.get("filter")
            return self._cargo(["test"] + ([flt] if isinstance(flt, str) and flt else []))
        return f"unknown tool {name}", True

    def close(self):
        pass
