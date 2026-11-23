# `bmb mcp` threat model

> Backlog 2D-20, ADR-0040 §3. The subject is BMB's own tool surface (`bmb mcp DIR`, `compiler/src/mcp.rs`). The security of agents in general (whether a model follows injected instructions, the supply chain) is outside the language's guarantee (ADR-0040 §3, the non-goals of DESIGN.md).

## Boundaries

| Subject | Trust | Reason |
| --- | --- | --- |
| The person who started the server and their environment (the `DIR` argument, `z3`·`clang` on `PATH`) | Trusted | The server runs with that person's privileges. The toolchain is the trust base |
| Requests from the MCP client (the agent) | Partly trusted | The agent is the party that was given the job of changing the workspace. But it can be deceived by text inside the workspace into sending a wrong request, so no request may reach outside the workspace |
| The contents of the workspace (`src/`, `.bmb-cache/`, `target/`) | Untrusted | It may be a repository someone else made. File names, links, unit summaries and definition names are all input |

## Threats and responses

| # | Threat | Response | Test |
| --- | --- | --- | --- |
| T1 | Writing outside the workspace through a unit or definition path (`../x`) | Every path part is checked against the identifier rule (1D-15) | `invalid_paths_are_rejected_and_write_nothing` in `tests/workspace.rs` |
| T2 | Reading or writing outside through a symbolic link: `src`·`.bmb-cache`·`target` is a link, `src/*.bmb` is a link, a temporary file link planted in advance | If any of the three directories or a unit file is a link, every tool refuses (`workspace::refuse_symlink`). A temporary file is opened with `create_new` after deleting whatever was there (`workspace::write_new`, the same for the proof and incremental cache files). **Before 2D-20, the planted temporary file link was followed and the target file was overwritten** | `untrusted_workspace_cannot_reach_outside` in `tests/mcp.rs` |
| T3 | Writing outside through the output name of `build` | The name is only a plain file name without a path separator, NUL or a leading `.` (1D-19), and the output goes under `target/` | `tool_errors_over_mcp` |
| T4 | Stack overflow through deep nesting (server exit) | JSON `MAX_DEPTH` 256, parser `MAX_NESTING` 256, 256MiB stack (1A-11) | `tests/nesting.rs`, the MCP 200k `[` test |
| T5 | Resource exhaustion: a copy of the workspace per open transaction | Open transactions `MAX_OPEN_TXS` = 64 | `untrusted_workspace_cannot_reach_outside` |
| T6 | Resource exhaustion: a query that does not end (`edges` depth 10^11) | Stops when the frontier is empty (1D-14) | `edges_with_a_huge_depth_terminate` in `tests/map.rs` |
| T7 | Resource exhaustion: solver time | A fixed `rlimit` per obligation, `-T` 300 seconds per definition (1D-18) | `tests/gate.rs` |
| T8 | Breaking the response framing: a string that looks like a protocol message in the text of the workspace (a summary) | Every response is one line escaped as a JSON string. Text goes out only as data | `untrusted_workspace_cannot_reach_outside`, `tests/mcp_fuzz.rs` |
| T9 | A malformed request makes the server hang or die | -32600 for a request that is not an object (1D-23), fuzzing of arbitrary call orders | `tests/mcp_fuzz.rs` |
| T10 | A lost write through a race with a concurrent writer | `tx_commit` refuses if the workspace changed after `tx_begin`, and files are written as a temporary file and then renamed (1D-19) | `tests/workspace.rs` (the atomic write of 1D-19), `untrusted_workspace_cannot_reach_outside` (concurrent writer) |

## Remaining risks (not addressed)

- **Instruction injection.** A unit summary (`#[summary]`) is free text and goes to the agent in query responses. The server only escapes it as data, and cannot prevent the agent from following that text as an instruction (out of scope). Definition and unit names are restricted to identifiers, so the injection channel is narrow.
- **The total time of one `verify`.** Solver time is spent in proportion to the number of definitions. The upper bound exists only per definition. The client sets the time limit.
- **Toolchain.** `build` runs the `clang` on `PATH`, and `verify` runs `z3`. The versions are checked against pinned values (INF-02·03), but the executables themselves are trusted.
- **Paths outside the workspace that are not links.** That `DIR` itself or a directory above it is a link is the choice of the person who started the server, so it is not checked.
