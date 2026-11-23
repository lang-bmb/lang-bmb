# Security

BMB is at version 0.x and is not ready for use outside its own development. There are no supported releases; fixes go to `main`.

## What counts

- **Soundness.** The compiler reports a program as proved (or builds it) although a contract, a bound, an overflow check or termination can fail. This is the defect the language exists to prevent and is treated as a security defect.
- **The tool surface.** `bmb mcp` serves a workspace to an agent. Reading or writing outside the workspace, running a program other than the pinned Z3 and clang, or a write that bypasses the proof gate is a defect. The model of what is defended and what is trusted is in [`docs/mcp-threat-model.md`](docs/mcp-threat-model.md).
- **Generated code.** Memory unsafety or undefined behavior in a compiled program that the checker and the proof gate accepted.

A crash of the compiler on malformed input is an ordinary defect: open an issue.

## Reporting

Report a soundness or tool-surface defect privately through GitHub's "Report a vulnerability" on the repository's Security tab. Include the smallest program or request that shows it, the commit, and the output of `bmb doctor`.

If private reporting is not available to you, open an issue that says a soundness defect exists without the reproducing program, and a maintainer will arrange a channel.
