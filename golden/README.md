# BMB Golden Binaries

Pre-built BMB compiler binaries for bootstrapping without Rust.

## Available Binaries

| Platform | Path | Status |
|----------|------|--------|
| Windows x64 | `windows-x64/bmb.exe` | ✅ Available |
| Linux x86_64 | `linux-x86_64/bmb` | 📋 Planned |
| Linux aarch64 | `linux-aarch64/bmb` | 📋 Planned |
| macOS Universal | `darwin-universal/bmb` | 📋 Planned |

## Version

Current version: **v0.93.32** (2026-02-14)

See `VERSION` file for full version information.

### Version History

| Version | Date | Features |
|---------|------|----------|
| v0.93.32 | 2026-02-14 | Fix variable shadowing, 43715 lines LLVM IR, 66 golden tests |
| v0.93.31 | 2026-02-14 | Fix empty string "" codegen bug, 43715 lines LLVM IR, 65 golden tests |
| v0.93.30 | 2026-02-14 | 5-pass IR optimization pipeline, 43680 lines LLVM IR, 60 golden tests, 610KB binary |
| v0.90.89 | 2026-02-14 | match expr, struct init, 68624 lines LLVM IR, 13 golden tests, build command |
| v0.89.20 | 2026-02-09 | 3-stage fixed point verified, 40612 lines LLVM IR |
| v0.88.2 | 2026-02-06 | Arena allocator for memory management |
| v0.88.0 | 2026-02-06 | Concurrency support, emit-ir CLI |
| v0.69.1 | 2026-02-05 | Initial golden binary |

## Usage

### Bootstrap without Rust

```bash
# Full automated bootstrap (recommended)
./scripts/golden-bootstrap.sh --verify

# Manual steps (Windows)
./golden/windows-x64/bmb.exe bootstrap/compiler.bmb stage1.ll
opt -O3 stage1.ll -S -o stage1_opt.ll
clang -O3 stage1_opt.ll bmb/runtime/libbmb_runtime.a -o bmb-stage1.exe -lm -lws2_32

# Manual steps (Linux/macOS)
./golden/linux-x86_64/bmb bootstrap/compiler.bmb stage1.ll
opt -O3 stage1.ll -S -o stage1_opt.ll
clang -O3 stage1_opt.ll bmb/runtime/libbmb_runtime.a -o bmb-stage1 -lm -lpthread
```

### Full 3-Stage Bootstrap

```bash
# Use the golden binary bootstrap script
./scripts/golden-bootstrap.sh
```

## Verification

The golden binary was generated through:

1. Rust compiler builds Stage 1 (BMB₁)
2. BMB₁ compiles bootstrap → Stage 2 (BMB₂)
3. BMB₂ compiles bootstrap → Stage 3 (BMB₃)
4. Verified: Stage 2 IR == Stage 3 IR (Fixed Point)

The golden binary is the Stage 2 binary that achieved fixed point.

## Requirements

- LLVM tools: `opt`, `clang` (or `llc` + `gcc`)
- Runtime files: `bmb/runtime/*.c`

## Updating Golden Binaries

After significant compiler changes:

```bash
# Run 3-stage bootstrap
./scripts/bootstrap.sh

# Verify fixed point achieved
# Copy new Stage 2 binary
cp target/bootstrap/bmb-stage2.exe golden/windows-x64/bmb.exe

# Update VERSION
echo "vX.Y.Z" > golden/VERSION
```
