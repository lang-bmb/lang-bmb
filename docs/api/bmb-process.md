# packages/bmb-process/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`run_command`](#run_command)
- [`run_command_output`](#run_command_output)
- [`run_system`](#run_system)
- [`getenv`](#getenv)
- [`PROC_SUCCESS`](#PROC_SUCCESS)
- [`PROC_ERROR_NOT_FOUND`](#PROC_ERROR_NOT_FOUND)
- [`PROC_ERROR_PERMISSION`](#PROC_ERROR_PERMISSION)
- [`PROC_ERROR_FAILED`](#PROC_ERROR_FAILED)
- [`command_exists`](#command_exists)
- [`build_args`](#build_args)
- [`build_args3`](#build_args3)

## Functions

### `run_command`

```bmb
pub fn run_command(command: String, args: String) -> i64
```

---

### `run_command_output`

```bmb
pub fn run_command_output(command: String, args: String) -> String
```

---

### `run_system`

```bmb
pub fn run_system(command: String) -> i64
```

---

### `getenv`

```bmb
pub fn getenv(name: String) -> String
```

---

### `PROC_SUCCESS`

```bmb
pub fn PROC_SUCCESS() -> i64
```

Process success

---

### `PROC_ERROR_NOT_FOUND`

```bmb
pub fn PROC_ERROR_NOT_FOUND() -> i64
```

Command not found

---

### `PROC_ERROR_PERMISSION`

```bmb
pub fn PROC_ERROR_PERMISSION() -> i64
```

Permission denied

---

### `PROC_ERROR_FAILED`

```bmb
pub fn PROC_ERROR_FAILED() -> i64
```

Execution failed

---

### `command_exists`

```bmb
pub fn command_exists(cmd: String) -> bool
```

Check if a command exists by trying to run it with --version

---

### `build_args`

```bmb
pub fn build_args(arg1: String, arg2: String) -> String
```

Build a command string with multiple arguments

---

### `build_args3`

```bmb
pub fn build_args3(arg1: String, arg2: String, arg3: String) -> String
```

Build a command string with three arguments

---

