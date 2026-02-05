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

See `VERSION` file for current golden binary version.

## Usage

### Bootstrap without Rust

```bash
# Windows
./golden/windows-x64/bmb.exe bootstrap/compiler.bmb stage1.ll
opt -O3 stage1.ll -o stage1_opt.ll
clang stage1_opt.ll bmb/runtime/*.c -o bmb-stage1.exe

# Linux/macOS (when available)
./golden/linux-x86_64/bmb bootstrap/compiler.bmb stage1.ll
opt -O3 stage1.ll -o stage1_opt.ll
clang stage1_opt.ll bmb/runtime/*.c -o bmb-stage1
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
