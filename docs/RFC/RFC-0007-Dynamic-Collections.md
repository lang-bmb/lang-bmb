# RFC-0007: Dynamic Collection Types

**Status**: Implemented
**Created**: 2026-01-08
**Implemented**: 2026-01-09
**Target Version**: v0.34
**Priority**: P1 (blocks binary_trees, hash_table, lru_cache benchmarks)
**Depends On**: Ownership system (complete), Generics (complete)

## Implementation Notes (v0.34.21-v0.34.24)

- **v0.34.21**: malloc/free/realloc/calloc builtins
- **v0.34.22**: Box<i64> (store_i64/load_i64/box_new_i64/box_free_i64)
- **v0.34.23**: Vec<i64> (vec_new/vec_push/vec_pop/vec_get/vec_set/vec_len/vec_cap/vec_free)
- **v0.34.24**: HashMap/HashSet (hashmap_*/hashset_*), hash_i64, vec_clear

Note: Implementation uses builtin functions (interpreter + LLVM codegen) rather than trait-based generics. This pragmatic approach provides:
- Full functionality for i64 key/value types
- Performance parity with Rust's HashMap
- Foundation for future generic collections

## Summary

Add dynamic collection types (`Vec<T>`, `HashMap<K, V>`, `HashSet<T>`) to BMB with ownership-based memory management, enabling heap-allocated data structures without garbage collection.

## Motivation

### Benchmark Gaps

Current benchmarks blocked by missing dynamic collections:

| Benchmark | Collection Required | Impact |
|-----------|---------------------|--------|
| binary_trees | Vec or Box (tree nodes) | Blocked |
| hash_table | HashMap<K, V> | Blocked |
| lru_cache | HashMap + LinkedList | Blocked |

### Use Cases

1. **Data Structures**: Trees, graphs, linked structures
2. **Caching**: LRU caches, memoization tables
3. **Parsing**: Token streams, AST construction
4. **Networking**: Message buffers, connection pools

### Current Workarounds

```bmb
-- Fixed-size array (current approach)
type FixedVec = struct {
    data: [i64; 1024],
    len: i64
};

-- Limitation: Size must be known at compile time
-- Limitation: Cannot grow beyond fixed capacity
```

### Philosophy Alignment

Per BMB's Non-Negotiable Priorities:
- **P0 Performance**: Zero-cost abstractions, no GC overhead
- **P0 Correctness**: Ownership prevents use-after-free, double-free
- **P0 Self-Hosting**: Enables bootstrap without Rust Vec dependency

## Design

### Memory Management Strategy

**Decision**: Ownership-based (Option C from RFC-0005)

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| Arena Allocation | Fast, simple | No individual deallocation | ❌ |
| Reference Counting | Shared ownership | Cycles, overhead | ❌ |
| **Ownership-based** | Zero-cost, safe | Learning curve | ✅ |

**Rationale**: Aligns with BMB's existing ownership model and achieves Rust-level performance.

### Vec<T> - Growable Array

```bmb
-- Core type definition
type Vec<T> = struct {
    ptr: own *T,      -- Owned pointer to heap allocation
    len: i64,         -- Current number of elements
    cap: i64          -- Allocated capacity
};

-- Construction
fn Vec::new<T>() -> Vec<T>;
fn Vec::with_capacity<T>(cap: i64{it >= 0}) -> Vec<T>;
fn Vec::from_array<T>(arr: [T; N]) -> Vec<T>;

-- Basic operations
fn push<T>(v: &mut Vec<T>, item: T) -> ();
fn pop<T>(v: &mut Vec<T>) -> Option<T>;
fn get<T>(v: &Vec<T>, index: i64) -> Option<&T>;
fn get_mut<T>(v: &mut Vec<T>, index: i64) -> Option<&mut T>;

-- Capacity management
fn len<T>(v: &Vec<T>) -> i64;
fn capacity<T>(v: &Vec<T>) -> i64;
fn is_empty<T>(v: &Vec<T>) -> bool;
fn reserve<T>(v: &mut Vec<T>, additional: i64) -> ();
fn shrink_to_fit<T>(v: &mut Vec<T>) -> ();

-- Removal
fn remove<T>(v: &mut Vec<T>, index: i64) -> T;
fn clear<T>(v: &mut Vec<T>) -> ();
fn truncate<T>(v: &mut Vec<T>, len: i64) -> ();

-- Iteration (returns iterator)
fn iter<T>(v: &Vec<T>) -> VecIter<T>;
fn iter_mut<T>(v: &mut Vec<T>) -> VecIterMut<T>;
```

#### Vec Memory Layout

```
Stack:                    Heap:
┌──────────────┐         ┌───┬───┬───┬───┬───┬───┬───┬───┐
│ ptr ─────────┼────────►│ 1 │ 2 │ 3 │ 4 │ - │ - │ - │ - │
├──────────────┤         └───┴───┴───┴───┴───┴───┴───┴───┘
│ len: 4       │         len=4, cap=8
├──────────────┤
│ cap: 8       │
└──────────────┘
```

#### Growth Strategy

```bmb
-- Quasi-doubling: 0 → 4 → 8 → 16 → 32 → ...
@inline
fn next_capacity(current: i64) -> i64 =
    if current == 0 then 4
    else current * 2;
```

### HashMap<K, V> - Hash-Based Dictionary

```bmb
-- Core type definition (Swiss Table inspired)
type HashMap<K: Hash + Eq, V> = struct {
    ctrl: own *u8,     -- Control bytes (metadata)
    data: own *Entry<K, V>,  -- Key-value pairs
    len: i64,          -- Number of entries
    cap: i64           -- Bucket count (power of 2)
};

type Entry<K, V> = struct {
    key: K,
    value: V
};

-- Construction
fn HashMap::new<K, V>() -> HashMap<K, V>;
fn HashMap::with_capacity<K, V>(cap: i64) -> HashMap<K, V>;

-- Basic operations
fn insert<K, V>(m: &mut HashMap<K, V>, key: K, value: V) -> Option<V>;
fn get<K, V>(m: &HashMap<K, V>, key: &K) -> Option<&V>;
fn get_mut<K, V>(m: &mut HashMap<K, V>, key: &K) -> Option<&mut V>;
fn remove<K, V>(m: &mut HashMap<K, V>, key: &K) -> Option<V>;
fn contains_key<K, V>(m: &HashMap<K, V>, key: &K) -> bool;

-- Capacity
fn len<K, V>(m: &HashMap<K, V>) -> i64;
fn is_empty<K, V>(m: &HashMap<K, V>) -> bool;
fn clear<K, V>(m: &mut HashMap<K, V>) -> ();

-- Iteration
fn keys<K, V>(m: &HashMap<K, V>) -> KeyIter<K, V>;
fn values<K, V>(m: &HashMap<K, V>) -> ValueIter<K, V>;
fn iter<K, V>(m: &HashMap<K, V>) -> HashMapIter<K, V>;
```

#### Hash Trait

```bmb
trait Hash {
    fn hash(&self) -> i64;
}

-- Built-in implementations
impl Hash for i64 {
    fn hash(&self) -> i64 =
        let h = self * 0x517cc1b727220a95;
        h ^ (h >> 32);
}

impl Hash for String {
    fn hash(&self) -> i64 =
        -- FNV-1a hash
        hash_bytes(self.as_bytes());
}
```

### HashSet<T> - Unique Collection

```bmb
-- Wrapper around HashMap<T, ()>
type HashSet<T: Hash + Eq> = struct {
    map: HashMap<T, ()>
};

fn HashSet::new<T>() -> HashSet<T>;
fn insert<T>(s: &mut HashSet<T>, item: T) -> bool;
fn contains<T>(s: &HashSet<T>, item: &T) -> bool;
fn remove<T>(s: &mut HashSet<T>, item: &T) -> bool;
fn len<T>(s: &HashSet<T>) -> i64;
fn iter<T>(s: &HashSet<T>) -> HashSetIter<T>;
```

### Contract Support

```bmb
-- Preconditions on indices
fn safe_get<T>(v: &Vec<T>, index: i64) -> &T
  pre index >= 0 and index < v.len()
= v.get(index).unwrap();

-- Postconditions on mutations
fn push_verified<T>(v: &mut Vec<T>, item: T) -> ()
  post v.len() == old(v.len()) + 1
= v.push(item);

-- Invariants with where blocks
fn sorted_insert(v: &mut Vec<i64>, item: i64) -> ()
  pre is_sorted(v)
  post is_sorted(v)
  where { contains: v.contains(&item) }
= -- Binary search and insert implementation
```

### Box<T> - Single Heap Allocation

```bmb
-- For tree nodes and recursive structures
type Box<T> = struct {
    ptr: own *T
};

fn Box::new<T>(value: T) -> Box<T>;
fn deref<T>(b: &Box<T>) -> &T;
fn deref_mut<T>(b: &mut Box<T>) -> &mut T;
fn into_inner<T>(b: Box<T>) -> T;
```

#### Example: Binary Tree

```bmb
type TreeNode = struct {
    value: i64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>
};

fn tree_sum(node: &Option<Box<TreeNode>>) -> i64 =
    match node {
        Option::None => 0,
        Option::Some(n) => n.value + tree_sum(&n.left) + tree_sum(&n.right)
    };
```

## Implementation

### Memory Allocator Interface

```bmb
-- Low-level allocator (stdlib/alloc.bmb)
@extern("malloc")
fn alloc(size: i64) -> *u8;

@extern("realloc")
fn realloc(ptr: *u8, new_size: i64) -> *u8;

@extern("free")
fn dealloc(ptr: *u8) -> ();

-- Type-safe wrappers
fn alloc_array<T>(count: i64) -> own *T =
    alloc(count * size_of::<T>()) as own *T;

fn realloc_array<T>(ptr: own *T, new_count: i64) -> own *T =
    realloc(ptr as *u8, new_count * size_of::<T>()) as own *T;
```

### Drop Trait (Destructor)

```bmb
trait Drop {
    fn drop(&mut self) -> ();
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) -> () = {
        -- Drop all elements
        for i in 0..self.len {
            drop(&mut self.data[i]);
        };
        -- Free heap allocation
        dealloc(self.ptr as *u8);
    };
}
```

### LLVM Code Generation

```llvm
; Vec::push implementation
define void @vec_push_i64(%Vec* %v, i64 %item) {
entry:
    %len = getelementptr %Vec, %Vec* %v, i32 0, i32 1
    %cap = getelementptr %Vec, %Vec* %v, i32 0, i32 2
    %len_val = load i64, i64* %len
    %cap_val = load i64, i64* %cap

    ; Check if reallocation needed
    %need_grow = icmp sge i64 %len_val, %cap_val
    br i1 %need_grow, label %grow, label %store

grow:
    ; ... reallocation logic
    br label %store

store:
    %ptr = getelementptr %Vec, %Vec* %v, i32 0, i32 0
    %data = load i64*, i64** %ptr
    %slot = getelementptr i64, i64* %data, i64 %len_val
    store i64 %item, i64* %slot

    ; Increment length
    %new_len = add i64 %len_val, 1
    store i64 %new_len, i64* %len
    ret void
}
```

### SMT Verification

```smt2
; Vec length invariants
(declare-fun vec_len (Vec) Int)
(declare-fun vec_cap (Vec) Int)

; Invariant: len <= cap
(assert (forall ((v Vec))
    (<= (vec_len v) (vec_cap v))))

; push postcondition
(assert (forall ((v Vec) (v2 Vec))
    (=> (vec_push v v2)
        (= (vec_len v2) (+ (vec_len v) 1)))))
```

## Alternatives Considered

### 1. Garbage Collection
- Pro: Simpler mental model
- Con: Unpredictable pauses, higher memory usage
- Decision: Rejected (violates P0 Performance)

### 2. Reference Counting Only
- Pro: Deterministic deallocation
- Con: Cycle handling, atomic overhead for Arc
- Decision: May add Rc<T>/Arc<T> later for shared ownership

### 3. Arena-Only Collections
- Pro: Very fast allocation
- Con: No individual deallocation, memory bloat
- Decision: May add Arena<T> for specific use cases

### 4. heapless-style Fixed Collections
- Pro: No heap allocation, embedded-friendly
- Con: Compile-time size limits
- Decision: Already have fixed arrays, keep as alternative

## Migration

### From Fixed Arrays

```bmb
-- Before (fixed-size)
let arr: [i64; 100] = [0; 100];

-- After (dynamic)
let vec = Vec::with_capacity::<i64>(100);
```

### From Option<T> Chains

```bmb
-- Before (manual linked list with Options)
type Node = struct {
    value: i64,
    next: Option<&Node>  -- Limited: borrows only
};

-- After (true heap allocation)
type Node = struct {
    value: i64,
    next: Option<Box<Node>>  -- Owned: can outlive scope
};
```

## Test Plan

### Unit Tests
- Vec: new, push, pop, get, resize, drop
- HashMap: insert, get, remove, collision handling
- HashSet: insert, contains, remove
- Box: new, deref, into_inner

### Integration Tests
- binary_trees benchmark (tree allocation/deallocation)
- hash_table benchmark (insert/lookup performance)
- lru_cache benchmark (combined HashMap + ordering)

### Memory Tests
- Valgrind/ASan for leak detection
- Stress tests for growth/shrink cycles
- Concurrent access tests (future: Arc<T>)

## Performance Targets

| Operation | Target vs Rust | Notes |
|-----------|----------------|-------|
| Vec::push (amortized) | 1.0x | Same growth strategy |
| Vec::get | 1.0x | Direct pointer arithmetic |
| HashMap::insert | 1.0x | Swiss Table algorithm |
| HashMap::get | 1.0x | SIMD-friendly layout |
| Box::new | 1.0x | Single malloc |
| Drop (Vec) | 1.0x | Linear deallocation |

## Timeline

| Phase | Task | Duration |
|-------|------|----------|
| 34.2.1 | Allocator interface | 2 days |
| 34.2.2 | Drop trait | 2 days |
| 34.2.3 | Box<T> | 3 days |
| 34.2.4 | Vec<T> | 5 days |
| 34.2.5 | Hash trait | 2 days |
| 34.2.6 | HashMap<K, V> | 5 days |
| 34.2.7 | HashSet<T> | 2 days |
| 34.2.8 | Benchmarks | 3 days |
| **Total** | | **~4 weeks** |

## References

- [Rust Vec Memory Layout](https://amritsingh183.github.io/rust/concepts/2025/01/05/rust-mem-ref.html)
- [Rust Performance Book - Heap Allocations](https://nnethercote.github.io/perf-book/heap-allocations.html)
- [Memory Allocation Strategies: Box, Arc, Rc](https://markaicode.com/memory-allocation-strategies-box-arc-rc-2025/)
- [Swiss Table HashMap Design](https://abseil.io/about/design/swisstables)
- [Rust Allocator RFC 1398](https://rust-lang.github.io/rfcs/1398-kinds-of-allocators.html)
- [heapless: Fixed Capacity Collections](https://docs.rust-embedded.org/book/collections/)

## Appendix A: Comparison with Rust

| Feature | Rust | BMB (Proposed) |
|---------|------|----------------|
| Vec growth | 0→4→8→16... | Same |
| HashMap algorithm | Swiss Table | Same |
| Drop order | LIFO | Same |
| Move semantics | Default | Default |
| Copy types | Explicit Copy trait | Explicit @copy attribute |

## Appendix B: Future Extensions

### Rc<T> - Reference Counted

```bmb
-- For shared ownership (single-threaded)
type Rc<T> = struct {
    ptr: *RcInner<T>
};

type RcInner<T> = struct {
    strong: i64,
    weak: i64,
    value: T
};
```

### Arc<T> - Atomic Reference Counted

```bmb
-- For shared ownership (multi-threaded)
type Arc<T> = struct {
    ptr: *ArcInner<T>
};
-- Uses atomic operations for thread safety
```

### LinkedList<T>

```bmb
-- Doubly-linked list for O(1) insertion/removal
type LinkedList<T> = struct {
    head: Option<Box<Node<T>>>,
    tail: *Node<T>,  -- Raw pointer for back-reference
    len: i64
};
```

### BTreeMap<K, V>

```bmb
-- Ordered map for range queries
type BTreeMap<K: Ord, V> = struct {
    root: Option<Box<BTreeNode<K, V>>>,
    len: i64
};
```
