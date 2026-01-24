# RFC-0002: Module Header System

**Status**: Proposed
**Created**: 2026-01-05
**Target Version**: v0.31

## Summary

Add mandatory module headers to BMB files for AI-friendly navigation and dependency tracking.

## Motivation

Current BMB files lack metadata, making it difficult for AI tools to:
- Quickly understand module purpose without reading entire file
- Identify exported symbols without parsing full AST
- Track dependencies explicitly

## Design

### Syntax

```bmb
module math.arithmetic
  version 1.0.0
  summary "integer arithmetic with overflow protection"

  exports add, subtract, multiply, divide

  depends
    core.types (i64, i128)
    core.error (overflow_error)

---

fn add(a: i64, b: i64) -> i64
  post ret == a + b
= a + b
```

### Header Fields

| Field | Required | Description |
|-------|----------|-------------|
| module | Yes | Fully qualified module name |
| version | No | SemVer version |
| summary | No | One-line description |
| exports | Yes | Public symbols |
| depends | No | Explicit dependencies |

### Separator

`---` on its own line separates header from body.

## Implementation

1. Add `module_header.rs` parser
2. Modify lexer for header keywords
3. Generate index from headers only (fast)
4. Validate exports match defined symbols

## Compatibility

- Non-breaking: existing files get default empty header
- Migration: `bmb migrate add-header` command
