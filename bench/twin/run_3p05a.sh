#!/usr/bin/env bash
# The first measurement on the port twin (backlog 3P-05a, ADR-0144):
# comprehension (c*) and refactoring (r*) tasks, qwen three times and sonnet
# once, and the Rust arm as written (comments kept) with sonnet once.
# A run that was cut goes on from its rows: start the script again.
set -euo pipefail
cd "$(dirname "$0")/../.."
export BMB="${BMB:-target/release/bmb}"
out=bench/twin/results/3p-05a
mkdir -p "$out"
ids=$(cd bench/twin/port && ls c*.json r*.json | sed 's/\.json$//' | tr '\n' ' ')
run() {  # NAME AGENT [ARMS..]
    local name=$1 agent=$2
    shift 2
    # shellcheck disable=SC2086
    python3 bench/twin/pair.py run --agent "$agent" --tasks bench/twin/port --only $ids --arms "${@:-bmb rust}" \
        --out "$out/$name.jsonl"
}
run qwen-1 openai:qwen3.8-27b bmb rust
run sonnet-1 anthropic:claude-sonnet-5-5 bmb rust
PAIR_DOCS=all run sonnet-all-rust anthropic:claude-sonnet-5-5 rust
run qwen-2 openai:qwen3.8-27b bmb rust
run qwen-3 openai:qwen3.8-27b bmb rust
