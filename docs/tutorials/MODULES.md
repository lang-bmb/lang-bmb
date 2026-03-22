# Modules & Project Structure in BMB

BMB uses a simple module system for organizing code across files.

## Importing Functions

Use `use module::function` to import from another module:

```bmb
use math::sqrt;
use string::starts_with;
use array::binary_search;

fn main() -> i64 = {
    let root = sqrt(2.0);
    0
};
```

## Module Resolution

When you write `use foo::bar`, BMB looks for `foo` in this order:

1. `foo.bmb` — single-file module in the same directory
2. `foo/mod.bmb` — directory module (stdlib convention)
3. `foo/src/lib.bmb` — package module (gotgan convention)

With `-I` include paths, the same patterns are tried in each include directory.

## Visibility

Use `pub` to make functions, structs, and enums accessible from other modules:

```bmb
// In math/mod.bmb:
pub fn sqrt(x: f64) -> f64 = ...;     // accessible from outside
fn newton_step(x: f64) -> f64 = ...;  // private helper
```

Without `pub`, functions are private to their module.

## Project Structure with gotgan

A gotgan project has this layout:

```
my-project/
├── gotgan.toml      # Package manifest
├── src/
│   ├── lib.bmb      # Library entry point (for libraries)
│   └── main.bmb     # Application entry point (for binaries)
└── target/          # Build output (auto-created)
```

### gotgan.toml

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "My BMB project"

[dependencies]
bmb-math = { path = "../bmb-math" }
```

### Using Dependencies

After declaring a dependency in `gotgan.toml`, import it with underscores:

```bmb
// gotgan.toml: bmb-math = { path = "..." }
// In your code:
use bmb_math::sqrt;
```

Hyphens in package names become underscores in `use` statements.

### Commands

```bash
gotgan new my-project    # Create new project
gotgan build             # Build the project
gotgan run               # Build and run
gotgan check             # Type-check only
gotgan test              # Run tests
gotgan tree              # Show dependency tree
gotgan update            # Update lock file
```

## stdlib Modules

BMB's standard library provides these modules:

| Module | Purpose |
|--------|---------|
| `core` | Fundamental types: num, bool, option, result |
| `string` | String operations: search, trim, parse |
| `array` | Array utilities: search, sort, aggregate |
| `io` | File I/O: read_file, write_file |
| `fs` | Filesystem: directories, path utilities |
| `time` | Timing: now_ns, now_ms, sleep_ms |
| `process` | Process execution: run_command, getenv |
| `json` | Zero-copy JSON parser/serializer |
| `math` | Mathematics: trig, power, gcd, fibonacci |
| `collections` | Data structures: Stack, Heap, Queue |
| `parse` | Position-based text parsing |
| `test` | Test assertions |

Import stdlib functions by module name:

```bmb
use math::gcd;
use string::starts_with;
use time::now_ms;
```
