# BMB v0.46 Golden Binary Release

This release marks the completion of BMB's 3-stage bootstrap verification, enabling BMB-only compiler builds without Rust.

## Contents

| File | Description | Platform |
|------|-------------|----------|
| `bmb-golden-linux-x64` | Golden binary compiler | Linux x86_64 |
| `libruntime_linux.a` | Pre-built runtime library | Linux x86_64 |
| `runtime.c` | Runtime source code | All platforms |

## Quick Start

### Prerequisites

- LLVM 21+ toolchain (`llc-21`, `clang-21`)
- Linux x86_64 or WSL

### Compile a BMB File

```bash
# Make binary executable
chmod +x bmb-golden-linux-x64

# Generate LLVM IR
./bmb-golden-linux-x64 build your_file.bmb > output.ll

# Compile and link
llc-21 -filetype=obj -O2 output.ll -o output.o
clang-21 -o output output.o libruntime_linux.a -lm

# Run
./output
```

### Build BMB Compiler from Source (Self-Hosting)

```bash
# Download the bootstrap source
git clone https://github.com/bmb-lang/bmb.git
cd bmb

# Generate IR for bootstrap compiler
./bmb-golden-linux-x64 build bootstrap/bmb_unified_cli.bmb > bmbc.ll

# Compile to binary
llc-21 -filetype=obj -O2 bmbc.ll -o bmbc.o
clang-21 -o bmbc bmbc.o libruntime_linux.a -lm

# Verify it works
./bmbc build bootstrap/bmb_unified_cli.bmb > verify.ll
diff bmbc.ll verify.ll  # Should be identical
```

## Building Runtime for Other Platforms

For platforms without pre-built runtime:

```bash
# Compile runtime.c
gcc -O2 -c runtime.c -o runtime.o
ar rcs libruntime.a runtime.o

# Use with your platform's LLVM toolchain
```

## Verification

This binary was produced through 3-stage bootstrap verification:

```
Stage 1 (Rust BMB):  bmb_unified_cli.bmb → bmbc_stage1 (160KB)
Stage 2 (Stage 1):   bmb_unified_cli.bmb → stage2.ll (1,068,850 bytes)
Stage 3 (Stage 2):   bmb_unified_cli.bmb → stage3.ll (1,068,850 bytes)

✅ stage2.ll = stage3.ll (byte-for-byte identical)
```

## Technical Details

- **Compiler Version**: v0.50.56
- **Bootstrap Size**: 143KB, 2895 lines of BMB
- **Binary Size**: 160KB (stripped)
- **Runtime Functions**: 33 (see runtime.c)
- **Target Triple**: x86_64-unknown-linux-gnu

## License

BMB is open source. See LICENSE file in the repository.
