# BMB Documentation Index

Quick reference guide to all BMB language documentation.

## Current Status: v0.10 Sunrise

**Last Updated**: 2026-01-03

---

## Core Documents

| Document | Description | Status |
|----------|-------------|--------|
| [SPECIFICATION.md](SPECIFICATION.md) | Complete language specification | Active |
| [LAWS.md](LAWS.md) | Design principles and philosophy | Active |
| [ROADMAP.md](ROADMAP.md) | Development roadmap with phase tracking | Active |
| [CONTRACT_CHECKLIST.md](CONTRACT_CHECKLIST.md) | Contract verification checklist | Active |

## Implementation Notes

Version-specific implementation details:

| Version | Focus | Document |
|---------|-------|----------|
| v0.1 | Lexer, Parser, AST | [IMPLEMENTATION_v0.1.md](IMPLEMENTATION_v0.1.md) |
| v0.2 | Type System, Contracts | [IMPLEMENTATION_v0.2.md](IMPLEMENTATION_v0.2.md) |
| v0.3 | Interpreter, REPL | [IMPLEMENTATION_v0.3.md](IMPLEMENTATION_v0.3.md) |
| v0.4 | MIR, LLVM Backend | [IMPLEMENTATION_v0.4.md](IMPLEMENTATION_v0.4.md) |
| v0.5 | Standard Library | [IMPLEMENTATION_v0.5.md](IMPLEMENTATION_v0.5.md) |

## Archived Documents

Historical references (completed or superseded):

| Document | Status | Notes |
|----------|--------|-------|
| [SPEC_V02_PLAN.md](SPEC_V02_PLAN.md) | Archived | v0.2 implementation plan (completed 2026-01-02) |

## Bootstrap Documentation

Self-hosted compiler components written in BMB:

- [bootstrap/README.md](../bootstrap/README.md) - Bootstrap overview and component details

## External Links

- [README.md](../README.md) - Project overview
- [CLAUDE.md](../CLAUDE.md) - Claude Code guidance

---

## Document Maintenance Guidelines

1. **Always update roadmap** when completing phases
2. **Archive completed plans** with clear status markers
3. **Keep version headers current** in all specifications
4. **Cross-reference** related documents
5. **Single source of truth**: ROADMAP.md is master status tracker
