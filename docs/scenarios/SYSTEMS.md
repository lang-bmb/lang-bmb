# Systems Programming with BMB

> Building low-level, memory-safe software with contract verification

## Overview

BMB is designed for systems programming—operating systems, embedded systems, compilers, and performance-critical applications. It combines C-level control with static verification for memory safety.

## Memory Model

### Direct Memory Access

BMB provides raw memory operations with contract-enforced safety:

```bmb
// Allocate memory
fn malloc(size: i64) -> i64
  pre size > 0
  post ret != 0 or ret == 0  // May fail

// Read/write memory
fn load_i64(ptr: i64) -> i64
  pre ptr != 0;

fn store_i64(ptr: i64, value: i64) -> i64
  pre ptr != 0;

// Free memory
fn free(ptr: i64) -> i64
  pre ptr != 0;
```

### Safe Memory Patterns

```bmb
// Allocate with size tracking
struct Buffer {
    ptr: i64,
    size: i64
}

fn buffer_new(size: i64) -> Buffer
  pre size > 0
= {
    let ptr = malloc(size * 8);
    Buffer { ptr: ptr, size: size }
};

fn buffer_get(buf: Buffer, idx: i64) -> i64
  pre idx >= 0
  pre idx < buf.size
= load_i64(buf.ptr + idx * 8);

fn buffer_set(buf: Buffer, idx: i64, value: i64) -> i64
  pre idx >= 0
  pre idx < buf.size
= store_i64(buf.ptr + idx * 8, value);

fn buffer_free(buf: Buffer) -> i64
  pre buf.ptr != 0
= free(buf.ptr);
```

## Data Structures

### Linked List

```bmb
// Node: [next_ptr: i64, value: i64]
fn node_new(value: i64) -> i64 = {
    let ptr = malloc(16);
    let s1 = store_i64(ptr, 0);      // next = null
    let s2 = store_i64(ptr + 8, value);
    ptr
};

fn node_next(node: i64) -> i64
  pre node != 0
= load_i64(node);

fn node_value(node: i64) -> i64
  pre node != 0
= load_i64(node + 8);

fn list_push(head: i64, value: i64) -> i64 = {
    let node = node_new(value);
    let s = store_i64(node, head);  // new.next = head
    node
};

fn list_len(head: i64) -> i64
  post ret >= 0
= if head == 0 { 0 } else { 1 + list_len(node_next(head)) };
```

### Ring Buffer

```bmb
// Header: [capacity, head, tail] + data
fn ring_new(capacity: i64) -> i64
  pre capacity > 0
  post ret != 0
= {
    let ptr = malloc((3 + capacity) * 8);
    let s1 = store_i64(ptr, capacity);
    let s2 = store_i64(ptr + 8, 0);   // head
    let s3 = store_i64(ptr + 16, 0);  // tail
    ptr
};

fn ring_capacity(ring: i64) -> i64
  pre ring != 0
= load_i64(ring);

fn ring_len(ring: i64) -> i64
  pre ring != 0
  post ret >= 0
= {
    let cap = load_i64(ring);
    let head = load_i64(ring + 8);
    let tail = load_i64(ring + 16);
    if tail >= head { tail - head }
    else { cap - head + tail }
};

fn ring_is_full(ring: i64) -> bool
  pre ring != 0
= ring_len(ring) == ring_capacity(ring);

fn ring_push(ring: i64, value: i64) -> bool
  pre ring != 0
= {
    if ring_is_full(ring) { false }
    else {
        let cap = load_i64(ring);
        let tail = load_i64(ring + 16);
        let data_offset = 24 + tail * 8;
        let s1 = store_i64(ring + data_offset, value);
        let new_tail = if tail + 1 >= cap { 0 } else { tail + 1 };
        let s2 = store_i64(ring + 16, new_tail);
        true
    }
};
```

## Bit Manipulation

```bmb
// Bitwise operations for systems programming
fn set_bit(value: i64, bit: i64) -> i64
  pre bit >= 0 and bit < 64
= value bor (1 << bit);

fn clear_bit(value: i64, bit: i64) -> i64
  pre bit >= 0 and bit < 64
= value band (bnot (1 << bit));

fn test_bit(value: i64, bit: i64) -> bool
  pre bit >= 0 and bit < 64
= (value band (1 << bit)) != 0;

fn count_ones(value: i64) -> i64
  post ret >= 0 and ret <= 64
= count_ones_iter(value, 0);

fn count_ones_iter(v: i64, count: i64) -> i64 =
    if v == 0 { count }
    else { count_ones_iter(v band (v - 1), count + 1) };
```

## I/O Operations

```bmb
// File operations with error handling
fn file_open(path: String) -> i64
  post ret >= 0 or ret == 0 - 1;  // Returns handle or -1

fn file_read(handle: i64) -> String
  pre handle > 0;

fn file_write(handle: i64, content: String) -> i64
  pre handle > 0
  post ret >= 0 or ret == 0 - 1;

fn file_close(handle: i64) -> i64
  pre handle > 0;

// Safe file processing
fn process_file(path: String) -> i64 = {
    let handle = file_open(path);
    if handle <= 0 { 0 - 1 }
    else {
        let content = file_read(handle);
        let result = process_content(content);
        let c = file_close(handle);
        result
    }
};
```

## Process Management

```bmb
// Command-line arguments
fn argc() -> i64
  post ret >= 0
= arg_count();

fn argv(idx: i64) -> String
  pre idx >= 0 and idx < arg_count()
= get_arg(idx);

// Environment
fn getenv(name: String) -> String;

// Exit
fn exit(code: i64) -> i64;
```

## String Operations

```bmb
// Efficient string building
fn build_path(dir: String, file: String) -> String = {
    let sb = sb_new();
    let s1 = sb_push(sb, dir);
    let s2 = sb_push(sb, "/");
    let s3 = sb_push(sb, file);
    sb_build(sb)
};

// String parsing
fn parse_int(s: String) -> i64 = parse_int_rec(s, 0, 0);

fn parse_int_rec(s: String, pos: i64, acc: i64) -> i64 =
    if pos >= s.len() { acc }
    else {
        let c = s.byte_at(pos);
        if c >= 48 and c <= 57 {
            parse_int_rec(s, pos + 1, acc * 10 + (c - 48))
        } else { acc }
    };
```

## Comparison with C

| Feature | C | BMB |
|---------|---|-----|
| Memory safety | Manual, error-prone | Contract-verified |
| Null pointer | Runtime crash | Compile-time caught |
| Buffer overflow | Silent corruption | Pre-condition violation |
| Resource leaks | Manual tracking | Pattern-enforced |
| Type safety | Weak | Strong with refinements |

## Example: Simple Allocator

```bmb
// Free list allocator
struct Allocator {
    heap: i64,
    free_list: i64,
    capacity: i64
}

fn allocator_new(size: i64) -> Allocator
  pre size > 0
= {
    let heap = malloc(size);
    Allocator {
        heap: heap,
        free_list: 0,
        capacity: size
    }
};

fn allocator_alloc(alloc: Allocator, size: i64) -> i64
  pre size > 0
  pre size <= alloc.capacity
= {
    // Simplified: allocate from heap linearly
    // Real implementation would use free list
    let ptr = alloc.heap;
    ptr
};

fn allocator_free(alloc: Allocator, ptr: i64) -> i64
  pre ptr >= alloc.heap
  pre ptr < alloc.heap + alloc.capacity
= {
    // Add to free list
    let s = store_i64(ptr, alloc.free_list);
    0
};
```

## Best Practices

1. **Always use preconditions for pointer validity**
   ```bmb
   fn access(ptr: i64) -> i64
     pre ptr != 0
   = load_i64(ptr);
   ```

2. **Track allocation sizes**
   ```bmb
   struct Array { ptr: i64, len: i64 }
   ```

3. **Use refinement types for invariants**
   ```bmb
   type NonNull = i64 where self != 0;
   ```

4. **Pair allocations with deallocations**
   ```bmb
   fn with_buffer(size: i64, f: fn(i64) -> i64) -> i64 = {
       let buf = malloc(size);
       let result = f(buf);
       let freed = free(buf);
       result
   };
   ```

## Next Steps

- [Contracts](CONTRACTS.md) - Deep dive into verification
- [Performance](PERFORMANCE.md) - Achieving C-level speed
- [AI Native](AI_NATIVE.md) - AI-assisted systems programming
