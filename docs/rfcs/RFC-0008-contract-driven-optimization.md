# RFC-0001: Contract-Driven Optimization (CDO)

> Expand the role of contracts beyond safety verification to enable unprecedented optimization across the entire programming workflow.

**RFC ID**: RFC-0001
**Title**: Contract-Driven Optimization
**Author**: BMB Core Team
**Date**: 2026-01-24
**Status**: Draft
**Labels**: `enhancement`, `optimization`, `contracts`, `architecture`, `rfc`

---

## Summary

BMB's contracts and annotations represent **deep semantic knowledge** about code behavior. Currently, this knowledge is primarily used for safety verification. This RFC proposes treating contracts as a **universal optimization resource** that should be leveraged across all stages of the programming lifecycle: writing, dependency resolution, compilation, linking, and deployment.

---

## Motivation

### The Untapped Potential

Traditional languages treat static analysis information as a byproduct:

```
Source Code → Compiler → Analysis (limited) → Binary
                              ↓
                         "Is this variable used?"
                         "Is this function called?"
                         (Surface-level questions only)
```

BMB is different. Contracts encode **semantic intent**:

```bmb
fn sort(arr: &[i32]) -> Vec<i32>
  pre arr.len() <= 1000
  post forall i: 0..ret.len()-1. ret[i] <= ret[i+1]
  post ret.len() == arr.len()
```

This tells us:
- Input constraints (max 1000 elements)
- Output guarantees (sorted, same length)
- Semantic meaning (this IS a sort function)

**This is not just safety information. This is optimization fuel.**

### The Conceptual Trap

We risk falling into a trap: viewing contracts only through the lens of "safety" because that's their traditional role in formal methods. But BMB is AI-first. AI writes verbose contracts without complaint. We will have **more semantic information than any language in history**.

If we only use this for safety checks, we waste most of its value.

---

## Proposal

### Core Principle

> **Every piece of semantic information should be exploited at every stage of the workflow.**

Contracts should inform:

| Stage | Traditional Use | Proposed Use |
|-------|-----------------|--------------|
| **Writing** | Error prevention | Intelligent autocomplete, semantic suggestions |
| **Dependencies** | Version compatibility | Semantic compatibility, minimal extraction |
| **Compilation** | Safety verification | Aggressive optimization, code specialization |
| **Linking** | Symbol resolution | Semantic deduplication, cross-module optimization |
| **Deployment** | — | Contract-specific binary generation |

### Mental Model Shift

```
OLD: Contracts prove code is safe
NEW: Contracts describe what code MEANS, enabling everything that follows
```

---

## Detailed Applications

### 1. Dependency Resolution: Semantic Extraction

**Current approach (all languages)**:
```
my-app uses json::parse()
→ Include json package (or tree-shaken subset)
→ Generic code included
```

**Proposed approach**:
```
my-app uses json::parse() with pre input.len() < 10000
→ Analyze json::parse() contract compatibility
→ Extract ONLY code paths valid under constraint
→ Generate contract-specialized minimal version
```

**Concrete example**:
```bmb
// json library has:
fn parse(s: &str) -> Result<Value, Error>
  // Handles: small strings, large files, streaming, unicode edge cases

// My code:
fn my_parse(s: &str) -> Value
  pre s.len() < 1000
  pre s.is_ascii()
= json::parse(s).unwrap();

// CDO extracts:
// - Only ASCII parsing paths
// - Only small-string paths
// - No streaming support
// - No unicode normalization
// - Simplified error handling (unwrap)
```

**Impact**: 80%+ reduction in dependency code for constrained use cases.

### 2. Compilation: Semantic Dead Code Elimination

**Current approach**:
```
if condition { A } else { B }
→ Compile both A and B
→ Runtime decides
```

**Proposed approach**:
```bmb
fn process(x: i32) -> i32
  pre x > 0
{
    if x <= 0 { return handle_error(); }  // UNREACHABLE by contract
    return compute(x);
}

// CDO eliminates:
// - The entire if branch
// - handle_error() function
// - All transitive dependencies of handle_error()
```

**This is not just dead code elimination. It's semantic dead code elimination** — removing code that is logically unreachable given contract constraints, not just syntactically unreachable.

### 3. Linking: Semantic Function Deduplication

**Current approach**:
```
Library A: sort_ascending() → compiled code A
Library B: order_numbers() → compiled code B
→ Both included, even if semantically identical
```

**Proposed approach**:
```bmb
// Library A
fn sort_ascending(arr: &[i32]) -> Vec<i32>
  post forall i: 0..ret.len()-1. ret[i] <= ret[i+1]
  post is_permutation(ret, arr)

// Library B
fn order_numbers(arr: &[i32]) -> Vec<i32>
  post forall i: 0..ret.len()-1. ret[i] <= ret[i+1]
  post is_permutation(ret, arr)

// CDO detects: EQUIVALENT CONTRACTS
// → Merge to single implementation
// → Choose faster implementation
// → Update all call sites
```

**Impact**: Reduced binary size, improved cache locality, automatic selection of optimal implementation.

### 4. Pure Functions: Computation as Data

```bmb
pure fn fibonacci(n: u32) -> u64
  pre n <= 50
  post ret == fib_mathematical_definition(n)
```

The `pure` annotation guarantees no side effects. Combined with `pre n <= 50`:

- **Compile-time**: Precompute lookup table for all 51 values
- **Link-time**: Replace function with table lookup
- **Cross-module**: Cache results across compilation units

```bmb
// Before CDO
fn use_fib() {
    let a = fibonacci(10);
    let b = fibonacci(10);  // Redundant call
    return a + b;
}

// After CDO
fn use_fib() {
    return 110;  // Fully constant-folded
}
```

### 5. Build: Contract-Specific Artifacts

Different deployment targets have different constraints:

```bmb
// Embedded target
#[target("embedded")]
pre HEAP_SIZE <= 64KB
pre NO_FLOAT

// Server target
#[target("server")]
pre HEAP_SIZE <= 16GB
pre ALLOW_THREADING
```

**CDO generates different binaries**:
- Embedded: No floating-point code, static allocation only
- Server: Full feature set, parallelized implementations

**Same source, contract-specialized outputs.**

### 6. IDE/Tooling: Semantic Autocomplete

Current autocomplete: "What methods exist on this type?"

Contract-aware autocomplete:
```bmb
fn process(arr: &[i32]) -> i32
  pre arr.len() > 0
{
    arr.|
    // Autocomplete suggests:
    // ✓ first() — valid, arr is non-empty
    // ✓ [0] — valid, arr is non-empty
    // ⚠ last() — valid but consider using first()
    // ✗ pop() — would violate immutability
}
```

---

## Implementation Strategy

### Phase 1: Foundation (Compiler Infrastructure)

1. **Contract IR**: Intermediate representation for contracts
2. **Semantic Equivalence Checker**: Determine if two contracts are equivalent
3. **Contract Propagation**: Flow contracts through call graphs

### Phase 2: Intra-Module Optimization

1. **Semantic DCE**: Remove contract-unreachable code
2. **Contract Specialization**: Generate specialized versions for constrained calls
3. **Pure Function Optimization**: CSE, memoization, constant folding

### Phase 3: Cross-Module Optimization

1. **Dependency Analysis**: Contract-aware dependency graph
2. **Semantic Deduplication**: Merge equivalent functions across modules
3. **Minimal Extraction**: Extract only contract-compatible code from dependencies

### Phase 4: Build System Integration

1. **gotgan Integration**: Contract-aware package resolution
2. **Target Specialization**: Contract-specific builds per target
3. **Incremental CDO**: Cache optimization decisions

---

## Expected Impact

### Quantitative (Estimates)

| Metric | Traditional | With CDO | Improvement |
|--------|-------------|----------|-------------|
| Binary size | 100% | 30-50% | 50-70% reduction |
| Dependency code included | 100% | 20-40% | 60-80% reduction |
| Dead code | ~10% eliminated | ~40% eliminated | 4x more |
| Compile-time optimization opportunities | Limited | Extensive | 5-10x more |

### Qualitative

- **Competitive advantage**: No other language can do this
- **AI synergy**: More contracts = more optimization = reward for verbosity
- **Ecosystem efficiency**: Smaller, faster binaries across all BMB programs

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Compile time increase | Incremental CDO, caching |
| Complexity | Phased implementation, clear IR design |
| Debug difficulty | Preserve source maps, CDO annotations in debug builds |
| Soundness | Formal verification of CDO transformations |

---

## Open Questions

1. **Contract equivalence decidability**: How do we handle undecidable equivalence cases?
2. **Incremental compilation**: How do contract changes propagate through the build?
3. **ABI stability**: Do contract-specialized functions need stable ABIs?
4. **Opt-out mechanism**: Should users be able to disable specific CDO passes?

---

## Relationship to Other Components

| Component | CDO Integration |
|-----------|-----------------|
| **BMB Compiler** | Core CDO passes in MIR optimization |
| **gotgan** | Contract-aware dependency resolution |
| **bmb-mcp (Chatter)** | AI understands CDO benefits when generating code |
| **bmb-test** | Tests validate CDO preserves semantics |
| **vscode-bmb** | Semantic autocomplete powered by CDO analysis |

---

## Proposed Milestones

| Version | Milestone | Description |
|---------|-----------|-------------|
| v0.55 | Contract IR | Contract intermediate representation |
| v0.56 | Semantic DCE | Intra-module dead code elimination |
| v0.57 | Pure Optimization | Pure function CSE, memoization |
| v0.58 | Contract Specialization | Generate specialized function variants |
| v0.60 | Cross-Module CDO | Link-time semantic optimization |
| v0.65 | gotgan CDO | Contract-aware package resolution |

---

## Conclusion

BMB's contract system gives us something unprecedented: **deep semantic knowledge about every function in the program**.

Using this only for safety verification is like having a map of every road in the world and only using it to check if roads exist. We should use it to find the shortest path, avoid traffic, discover scenic routes, and plan entire journeys.

**Contracts are not just guards. They are guides.**

The question is not "can we do this?" — Z3 and modern compilers prove we can. The question is "will we commit to this vision?" — treating contracts as the universal optimization resource they can be.

---

## Next Steps

1. [ ] Team discussion on conceptual alignment
2. [ ] Prototype: Contract equivalence checker
3. [ ] Prototype: Semantic DCE on single module
4. [ ] Benchmark: Measure real-world impact on bmb-test codebase
5. [ ] Design document: Contract IR specification
6. [ ] RFC: gotgan contract-aware dependency resolution

---

## References

- [BMB Language Specification v0.32.1](../SPECIFICATION.md)
- [BMB Development Guidelines](../DEVELOPMENT.md)
- LLVM Link-Time Optimization (existing cross-module optimization)
- Z3 Theorem Prover (semantic analysis engine)
