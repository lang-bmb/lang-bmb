# bmb-fs — Filesystem Module API

Directory operations, file management, and path utilities.

## Directory Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `is_dir(path)` | `(String) -> i64` | Returns 1 if directory, 0 if not |
| `make_dir(path)` | `(String) -> i64` | Create directory. Returns 0 on success |
| `list_dir(path)` | `(String) -> String` | List entries (newline-separated) |
| `remove_dir(path)` | `(String) -> i64` | Remove empty directory |

**Contracts**: `pre path.len() > 0`

## File Management

| Function | Signature | Description |
|----------|-----------|-------------|
| `remove_file(path)` | `(String) -> i64` | Delete a file. Returns 0 on success |

## Working Directory

| Function | Signature | Description |
|----------|-----------|-------------|
| `current_dir()` | `() -> String` | Get current working directory |

## Path Utilities (Pure)

| Function | Signature | Description |
|----------|-----------|-------------|
| `is_valid_path(path)` | `(String) -> bool` | Check path validity (non-empty, ≤4096 chars) |
| `extension(path)` | `(String) -> String` | Get file extension (without dot) |
| `filename(path)` | `(String) -> String` | Get filename from path |
| `parent(path)` | `(String) -> String` | Get parent directory |
| `join(base, name)` | `(String, String) -> String` | Join path segments with `/` |
| `has_extension(path, ext)` | `(String, String) -> bool` | Check if path has given extension |

## Error Codes

| Constant | Value | Meaning |
|----------|-------|---------|
| `FS_SUCCESS` | 0 | Operation succeeded |
| `FS_ERROR_NOT_FOUND` | -2 | File/directory not found |
| `FS_ERROR_PERMISSION` | -13 | Permission denied |
| `FS_ERROR_EXISTS` | -17 | Already exists |
| `FS_ERROR_NOT_DIR` | -20 | Not a directory |
| `FS_ERROR_IS_DIR` | -21 | Is a directory |
| `FS_ERROR_NOT_EMPTY` | -39 | Directory not empty |

## Example

```bmb
use fs::extension;
use fs::filename;
use fs::join;

fn main() -> i64 = {
    let path = join("src", "main.bmb");
    let name = filename(path);      // "main.bmb"
    let ext = extension(path);      // "bmb"
    0
};
```
