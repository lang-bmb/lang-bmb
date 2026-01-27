# BMB Compiler Architecture

## Design Philosophy

> **"Proofs are not for safety. Proofs are for speed."**

Traditional compilers treat verification as a correctness check—something that happens, produces errors or warnings, and is then forgotten. BMB inverts this relationship: **verification produces optimization data**.

When we prove `idx < len`, we're not just checking safety—we're generating a **mathematical fact** that eliminates bounds checking. When we prove `ptr != null`, we're not just avoiding undefined behavior—we're enabling direct memory access without conditional branches.

This insight leads to BMB's core compiler principle:

```
┌─────────────────────────────────────────────────────────────┐
│  Traditional: Source → Parse → Type → Optimize → Codegen   │
│                                  ↑                          │
│                            "best effort"                    │
│                                                             │
│  BMB:         Source → Parse → Type → PROVE → Optimize     │
│                                         ↑           ↑       │
│                              mathematical    uses proofs    │
│                                 facts        as directives  │
└─────────────────────────────────────────────────────────────┘
```

---

## The Six Phases

BMB compilation proceeds through six distinct phases, each with a clear mathematical purpose:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ╔═══════════════════════════════════════════════════════════════╗  │
│  ║  PHASE 1: LEXICAL                                             ║  │
│  ║  String → Token Stream                                        ║  │
│  ║  "What characters form meaningful units?"                     ║  │
│  ╚═══════════════════════════════════════════════════════════════╝  │
│                              ↓                                      │
│  ╔═══════════════════════════════════════════════════════════════╗  │
│  ║  PHASE 2: SYNTACTIC                                           ║  │
│  ║  Token Stream → Abstract Syntax Tree                          ║  │
│  ║  "What is the grammatical structure?"                         ║  │
│  ╚═══════════════════════════════════════════════════════════════╝  │
│                              ↓                                      │
│  ╔═══════════════════════════════════════════════════════════════╗  │
│  ║  PHASE 3: SEMANTIC                                            ║  │
│  ║  AST → Typed AST + Contract IR                                ║  │
│  ║  "What do the symbols mean? What are the constraints?"        ║  │
│  ╚═══════════════════════════════════════════════════════════════╝  │
│                              ↓                                      │
│  ╔═══════════════════════════════════════════════════════════════╗  │
│  ║  PHASE 4: VERIFICATION  ← BMB's Key Innovation                ║  │
│  ║  Contract IR → Proof-Annotated IR                             ║  │
│  ║  "What can we mathematically prove?"                          ║  │
│  ╚═══════════════════════════════════════════════════════════════╝  │
│                              ↓                                      │
│  ╔═══════════════════════════════════════════════════════════════╗  │
│  ║  PHASE 5: OPTIMIZATION                                        ║  │
│  ║  Proof-Annotated IR → Optimized MIR                           ║  │
│  ║  "How do we exploit proven facts for speed?"                  ║  │
│  ╚═══════════════════════════════════════════════════════════════╝  │
│                              ↓                                      │
│  ╔═══════════════════════════════════════════════════════════════╗  │
│  ║  PHASE 6: EMISSION                                            ║  │
│  ║  Optimized MIR → Target Code                                  ║  │
│  ║  "What machine instructions realize the computation?"         ║  │
│  ╚═══════════════════════════════════════════════════════════════╝  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Intermediate Representations

BMB uses **five distinct IRs**, each serving a specific mathematical purpose:

| IR | Full Name | Purpose | Key Property |
|----|-----------|---------|--------------|
| **AST** | Abstract Syntax Tree | Grammatical structure | Tree-shaped, untyped |
| **TAST** | Typed AST | Semantic meaning | Every node has a type |
| **CIR** | Contract IR | Contract normalization | Contracts as first-class values |
| **PIR** | Proof-Indexed IR | Verified semantics | Every expr carries proven facts |
| **MIR** | Machine IR | Control flow | CFG-based, proof-annotated |

### IR Transition Diagram

```
Source Code (.bmb)
       │
       │ Lexer (logos)
       ▼
Token Stream
       │
       │ Parser (lalrpop)
       ▼
   ┌───────┐
   │  AST  │  Untyped, tree structure
   └───┬───┘
       │
       │ Type Inference (Hindley-Milner)
       │ Name Resolution
       ▼
  ┌────────┐
  │  TAST  │  Typed AST - every node has type info
  └────┬───┘
       │
       │ Contract Extraction
       │ Effect Analysis
       ▼
   ┌───────┐
   │  CIR  │  Contract IR - contracts normalized
   └───┬───┘                 as logical propositions
       │
       │ SMT Translation (Z3)
       │ Proof Generation
       ▼
   ┌───────┐
   │  PIR  │  Proof-Indexed IR - every value
   └───┬───┘  carries its proven properties
       │
       │ CFG Construction
       │ Proof-Guided Lowering
       ▼
   ┌───────┐
   │  MIR  │  Machine IR - CFG with proof annotations
   └───┬───┘
       │
       │ Contract-Based Optimization
       │ Traditional Optimization
       ▼
Optimized MIR
       │
       │ Code Generation
       ▼
LLVM IR / WASM / Native
```

---

## Phase 1: Lexical Analysis

**Input**: Source code (UTF-8 string)
**Output**: Token stream with spans
**Complexity**: O(n) where n = source length

### Purpose

Transform character stream into meaningful lexical units. This phase is intentionally simple—complexity belongs in later phases.

### Token Categories

```
┌─────────────┬────────────────────────────────────────┐
│ Category    │ Examples                               │
├─────────────┼────────────────────────────────────────┤
│ Keywords    │ fn, let, if, while, pre, post, pure    │
│ Literals    │ 42, 3.14, "hello", true                │
│ Identifiers │ foo, _bar, Vec3                        │
│ Operators   │ +, -, *, /, ==, !=, &&, ||             │
│ Delimiters  │ (, ), {, }, [, ], ;, :, ->             │
│ Contracts   │ @pre, @post, @invariant, @pure         │
└─────────────┴────────────────────────────────────────┘
```

### Design Decision: No Preprocessing

BMB has no preprocessor. Conditional compilation uses `@cfg` attributes which are handled at AST level, not lexical level. This keeps the lexer simple and the language semantics clean.

---

## Phase 2: Syntactic Analysis

**Input**: Token stream
**Output**: Abstract Syntax Tree (AST)
**Grammar**: LR(1), defined in `grammar.lalrpop`

### Purpose

Establish grammatical structure. The AST represents the **shape** of the program without semantic meaning.

### AST Design Principles

1. **Span Preservation**: Every node carries source location for error reporting
2. **Minimal Desugaring**: Keep close to source syntax; desugar in later phases
3. **Expression-Oriented**: Everything is an expression (if, match, block)

### Core AST Nodes

```rust
Program {
    items: Vec<Item>
}

Item =
    | Function { name, params, ret_ty, contracts, body }
    | Struct { name, generics, fields }
    | Enum { name, generics, variants }
    | Trait { name, generics, methods }
    | Impl { target, trait_name, methods }
    | Extern { module, declarations }

Expr =
    | Literal(i64 | f64 | String | bool)
    | Ident(name)
    | Binary(op, lhs, rhs)
    | Unary(op, operand)
    | Call(callee, args)
    | If(cond, then, else)
    | Match(scrutinee, arms)
    | Block(stmts, final_expr)
    | Let(pattern, type_annotation, initializer)
    | While(cond, body)
    | FieldAccess(base, field)
    | Index(base, index)
    | Lambda(params, body)

Contract =
    | Pre(condition)
    | Post(condition, result_binding)
    | Invariant(condition)
```

---

## Phase 3: Semantic Analysis

**Input**: AST
**Output**: Typed AST (TAST) + Contract IR (CIR)
**Algorithm**: Hindley-Milner type inference with extensions

### Purpose

Establish meaning. After this phase:
- Every expression has a concrete type
- All names are resolved to their definitions
- Contracts are normalized into logical propositions

### 3.1 Name Resolution

Resolve all identifiers to their definitions:

```
┌─────────────────────────────────────────────────────────┐
│ Before:  let x = foo(bar)                               │
│ After:   let x@local_3 = foo@fn_7(bar@param_1)          │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Type Inference

BMB uses **bidirectional type inference** with Hindley-Milner as the foundation:

```
Γ ⊢ e : τ    (expression e has type τ in context Γ)

───────────────────  (Literal)
Γ ⊢ 42 : i64

Γ ⊢ e₁ : τ → σ    Γ ⊢ e₂ : τ
─────────────────────────────  (Application)
      Γ ⊢ e₁(e₂) : σ

Γ, x : τ ⊢ e : σ
────────────────────────────  (Lambda)
Γ ⊢ (x: τ) => e : τ → σ
```

### 3.3 Contract Normalization (→ CIR)

Contracts are extracted and normalized into **Contract IR (CIR)**:

```bmb
// Source
fn binary_search(arr: &[i64], target: i64) -> i64
    pre arr.len() > 0
    pre is_sorted(arr)
    post ret >= -1
    post ret < arr.len()
= { ... }
```

```
// CIR (Contract IR)
Function binary_search {
    preconditions: [
        Proposition::Comparison(Gt, arr.len, 0),
        Proposition::Call("is_sorted", [arr])
    ],
    postconditions: [
        Proposition::Comparison(Gte, ret, -1),
        Proposition::Comparison(Lt, ret, arr.len)
    ],
    assumptions: [],  // Filled by verification phase
    guarantees: []    // Derived from postconditions
}
```

### 3.4 Effect Analysis

Analyze side effects for each function:

```
┌────────────┬───────────────────────────────────────────┐
│ Effect     │ Description                               │
├────────────┼───────────────────────────────────────────┤
│ Pure       │ No side effects, deterministic            │
│ Read       │ Reads memory/globals                      │
│ Write      │ Writes memory/globals                     │
│ Allocate   │ Allocates memory                          │
│ IO         │ Performs I/O                              │
│ Diverge    │ May not terminate                         │
└────────────┴───────────────────────────────────────────┘
```

Pure functions with constant arguments can be evaluated at compile time.

---

## Phase 4: Verification (BMB's Key Innovation)

**Input**: Contract IR (CIR)
**Output**: Proof-Indexed IR (PIR)
**Solver**: Z3 SMT Solver

### Purpose

Transform contracts into **proven facts** that guide optimization. This is not just validation—it's **proof generation**.

### The Verification Equation

```
Traditional:  verify(contract) → {pass, fail}
BMB:          verify(contract) → ProofFact[]
```

A `ProofFact` is a mathematical statement proven true at a specific program point:

```rust
struct ProofFact {
    /// The proven proposition
    proposition: Proposition,
    /// Where this fact is valid
    scope: Scope,
    /// How it was proven
    evidence: ProofEvidence,
}

enum ProofEvidence {
    /// Directly from a precondition
    Precondition(ContractId),
    /// Derived by SMT solver
    SmtProof(SmtProofTree),
    /// Follows from control flow
    ControlFlowImplication(BranchCondition),
    /// Induction hypothesis
    Induction(LoopInvariant),
}
```

### 4.1 SMT Translation

Convert CIR to SMT-LIB2 for Z3:

```bmb
// BMB Contract
fn abs(x: i64) -> i64
    post (ret >= 0)
    post (ret == x || ret == -x)
= if x >= 0 { x } else { -x };
```

```smt2
; SMT-LIB2
(declare-const x Int)
(declare-const ret Int)

; Function body encoding
(assert (= ret (ite (>= x 0) x (- x))))

; Verify postcondition 1: ret >= 0
(push)
(assert (not (>= ret 0)))
(check-sat)  ; Expected: unsat (proof found)
(pop)

; Verify postcondition 2: ret == x || ret == -x
(push)
(assert (not (or (= ret x) (= ret (- x)))))
(check-sat)  ; Expected: unsat (proof found)
(pop)
```

### 4.2 Proof Propagation

Proven facts propagate through the program:

```bmb
fn process(arr: &[i64], idx: i64)
    pre idx >= 0
    pre idx < arr.len()
= {
    let val = arr[idx];     // PROOF: idx in bounds → no bounds check
    let next = idx + 1;
    if next < arr.len() {   // Branch condition becomes proof
        arr[next]           // PROOF: next in bounds → no bounds check
    } else {
        val
    }
};
```

Proof facts at each point:

```
Point 1 (entry):
  PROVEN: idx >= 0
  PROVEN: idx < arr.len()

Point 2 (let val = arr[idx]):
  PROVEN: idx in bounds (from preconditions)
  → EMIT: unchecked array access

Point 3 (if-then branch):
  PROVEN: next < arr.len() (from branch condition)
  PROVEN: next >= 0 (derived: idx >= 0 ∧ next = idx + 1 → next >= 1)
  → EMIT: unchecked array access
```

### 4.3 Proof-Indexed IR (PIR)

After verification, we have PIR where every subexpression carries its proven properties:

```
// PIR representation
let val = arr[idx]
    @proven { idx >= 0, idx < arr.len() }
    @guarantees { unchecked_access }
```

### 4.4 Proof Database

Proofs are cached and reusable:

```rust
struct ProofDatabase {
    /// Verified contract results
    contract_proofs: HashMap<ContractId, ProofResult>,

    /// Inter-procedural facts
    function_summaries: HashMap<FunctionId, FunctionSummary>,

    /// Incremental compilation support
    proof_cache: LruCache<ProofQuery, ProofResult>,
}

struct FunctionSummary {
    /// What the function assumes (preconditions)
    assumes: Vec<Proposition>,
    /// What the function guarantees (postconditions)
    guarantees: Vec<Proposition>,
    /// Effect classification
    effects: EffectSet,
    /// Whether function is proven terminating
    terminates: bool,
}
```

---

## Phase 5: Optimization

**Input**: Proof-Indexed IR (PIR)
**Output**: Optimized MIR
**Strategy**: Proof-guided transformation

### Purpose

Exploit proven facts for maximum performance. Every optimization is either:
1. **Traditional**: Dead code elimination, constant folding, etc.
2. **Proof-Enabled**: Only possible because of proven facts

### 5.1 PIR → MIR Lowering

Convert tree-structured PIR to CFG-based MIR:

```
// PIR (tree)
if x > 0 {
    f(x)
} else {
    g(x)
}

// MIR (CFG)
bb0:
    %cmp = icmp sgt i64 %x, 0
    br i1 %cmp, bb1, bb2

bb1:  // PROOF: x > 0
    %r1 = call @f(%x)
    br bb3

bb2:  // PROOF: x <= 0
    %r2 = call @g(%x)
    br bb3

bb3:
    %result = phi [%r1, bb1], [%r2, bb2]
    ret %result
```

### 5.2 Contract-Based Optimizations

These optimizations are **only possible with proofs**:

#### Bounds Check Elimination (BCE)

```bmb
// Source
fn sum(arr: &[i64]) -> i64
    pre arr.len() > 0
= {
    let mut total = 0;
    let mut i = 0;
    while i < arr.len() {
        total = total + arr[i];  // BCE: i < arr.len() proven
        i = i + 1;
    }
    total
};
```

```
// Without proof: arr[i] generates
  %in_bounds = icmp ult i64 %i, %len
  br i1 %in_bounds, %access, %panic

// With proof: direct access
  %ptr = getelementptr i64, ptr %arr, i64 %i
  %val = load i64, ptr %ptr
```

#### Null Check Elimination (NCE)

```bmb
fn process(node: Node?)
    pre node != null
= {
    node.value  // NCE: null check eliminated
};
```

#### Division by Zero Elimination

```bmb
fn divide(a: i64, b: i64) -> i64
    pre b != 0
= a / b;  // No zero check needed
```

#### Range-Based Optimization

```bmb
fn classify(score: i64) -> String
    pre score >= 0
    pre score <= 100
= {
    // Compiler knows: score ∈ [0, 100]
    // Can use jump table instead of if-else chain
    if score >= 90 { "A" }
    else if score >= 80 { "B" }
    else if score >= 70 { "C" }
    else if score >= 60 { "D" }
    else { "F" }
};
```

#### Unreachable Code Elimination

```bmb
fn process(x: i64)
    pre x > 0
= {
    if x <= 0 {
        // UNREACHABLE: contradicts precondition
        panic("negative")  // Entire branch eliminated
    }
    compute(x)
};
```

### 5.3 Traditional Optimizations

These work on any code, but benefit from proof annotations:

| Pass | Description | Proof Enhancement |
|------|-------------|-------------------|
| **Constant Folding** | `1 + 2` → `3` | Range proofs enable more folding |
| **Constant Propagation** | Propagate known values | Proof facts are propagated too |
| **Dead Code Elimination** | Remove unused code | Unreachable proofs eliminate more |
| **Common Subexpression Elimination** | Deduplicate computations | Pure function proofs enable more CSE |
| **Loop Invariant Code Motion** | Hoist loop-invariant code | Proof preservation across iterations |
| **Inlining** | Replace call with body | Pure function proofs enable aggressive inlining |
| **Tail Call Optimization** | Convert tail recursion to loop | Automatic for proven-terminating functions |

### 5.4 Optimization Pipeline

```rust
enum OptLevel {
    /// No optimization (for debugging)
    Debug,
    /// Standard optimization
    Release,
    /// Aggressive optimization (may increase compile time)
    Aggressive,
}

fn optimization_pipeline(level: OptLevel) -> Vec<Pass> {
    match level {
        Debug => vec![],

        Release => vec![
            // Contract-based (BMB-specific)
            BoundsCheckElimination,
            NullCheckElimination,
            ContractUnreachableElimination,

            // Traditional
            ConstantFolding,
            ConstantPropagation,
            DeadCodeElimination,
            CommonSubexpressionElimination,
            SimplifyBranches,

            // Repeated for fixed-point
            ConstantPropagation,
            DeadCodeElimination,
        ],

        Aggressive => vec![
            // All Release passes, plus:
            AggressiveInlining,
            LoopUnrolling,
            Vectorization,

            // Second round of contract-based
            BoundsCheckElimination,
            RangePropagation,

            // More aggressive traditional
            TailCallOptimization,
            PartialRedundancyElimination,
        ],
    }
}
```

---

## Phase 6: Code Emission

**Input**: Optimized MIR
**Output**: Target code (LLVM IR, WASM, or native)

### Purpose

Translate optimized MIR to executable form. By this phase, all semantic decisions have been made—emission is purely mechanical.

### 6.1 LLVM IR Generation

```
MIR                          LLVM IR
─────────────────────────────────────────────────────
MirType::I64            →    i64
MirType::F64            →    double
MirType::Bool           →    i1
MirType::String         →    %BmbString*
MirType::Ptr(T)         →    ptr

MirStmt::Assign         →    store
MirStmt::Call           →    call
MirStmt::Load           →    load

Terminator::Return      →    ret
Terminator::Branch      →    br
Terminator::Switch      →    switch
```

### 6.2 WASM Generation

```
MIR                          WASM
─────────────────────────────────────────────────────
MirType::I64            →    i64
MirType::F64            →    f64
MirType::Bool           →    i32 (0 or 1)
MirType::String         →    i32 (pointer)

MirStmt::Assign         →    local.set
MirStmt::Call           →    call
MirStmt::Load           →    i64.load

Terminator::Return      →    return
Terminator::Branch      →    br_if / br
```

### 6.3 Target-Specific Optimization

Some optimizations are target-specific:

| Target | Optimization |
|--------|--------------|
| x86-64 | SIMD vectorization (SSE/AVX) |
| ARM64 | NEON vectorization |
| WASM | Stack-based instruction selection |

---

## Comparison with Other Compilers

### BMB vs Rust

| Aspect | Rust | BMB |
|--------|------|-----|
| **Safety Mechanism** | Borrow checker (ownership) | Contract proofs (SMT) |
| **When Checked** | Compile time | Compile time |
| **Optimization Benefit** | Aliasing info | Proof facts |
| **Unique IRs** | HIR, THIR, MIR | CIR, PIR, MIR |
| **Runtime Cost** | Zero (mostly) | Zero (by proof) |

```
Rust:   Source → AST → HIR → THIR → MIR → LLVM IR
                        ↑     ↑
                   borrow  type
                   check   check

BMB:    Source → AST → TAST → CIR → PIR → MIR → LLVM IR
                        ↑      ↑     ↑
                      type  contract proof
                      check  extract propagate
```

### BMB vs C/C++

| Aspect | C/C++ | BMB |
|--------|-------|-----|
| **Bounds Checking** | None (UB) | Eliminated by proof |
| **Null Checking** | Programmer's job | Eliminated by proof |
| **Optimization** | Best-effort | Proof-guaranteed |
| **Safety** | Unsafe by default | Safe by proof |

### BMB vs Java/C#

| Aspect | Java/C# | BMB |
|--------|---------|-----|
| **Bounds Checking** | Runtime | Eliminated at compile time |
| **Null Checking** | Runtime (NPE) | Eliminated at compile time |
| **Performance Cost** | ~5-10% overhead | Zero overhead |

---

## Implementation Status

### Current State (v0.52)

| Phase | Status | Notes |
|-------|--------|-------|
| Lexical | ✅ Complete | logos-based |
| Syntactic | ✅ Complete | lalrpop, 60KB grammar |
| Semantic | ✅ Complete | H-M type inference |
| CIR | ✅ Implemented | `bmb/src/cir/` - ~106KB |
| Verification | ✅ Implemented | ProofDB, incremental verification |
| PIR | ✅ Implemented | `bmb/src/pir/` - ~55KB |
| Optimization | ✅ Complete | 15+ MIR passes (traditional) |
| Emission | ✅ Complete | LLVM IR + WASM |

### Integration Status

| Component | Code Status | Pipeline Integration |
|-----------|-------------|---------------------|
| CIR representation | ✅ Complete | ✅ **Integrated (v0.52)** |
| CIR → MIR facts | ✅ Complete | ✅ **Integrated (v0.52)** |
| Contract-based optimizations | ✅ Complete | ✅ **Integrated (v0.52)** |
| PIR representation | ✅ Complete | ✅ **Integrated (v0.55)** |
| PIR proof propagation | ✅ Complete | ✅ **Integrated (v0.55)** |
| PIR → MIR facts | ✅ Complete | ✅ **Integrated (v0.55)** |
| ProofDatabase | ✅ Complete | ✅ **Integrated (v0.55)** |
| FunctionSummary | ✅ Complete | ❌ Not used |
| IncrementalVerifier | ✅ Complete | ❌ Not used |
| PIR → MIR lowering | ⚠️ Stub only | ❌ Not needed (using fact extraction) |

### Roadmap (Updated v0.55)

| Version | Milestone | Status |
|---------|-----------|--------|
| v0.52 | Explicit CIR representation | ✅ Done |
| v0.52 | CIR → MIR facts bridge | ✅ Done |
| v0.52 | Contract-based optimizations (BCE, NCE, DCE, PUE) | ✅ Done |
| v0.55 | PIR proof propagation | ✅ **Done** |
| v0.55 | PIR → MIR facts extraction | ✅ **Done** |
| v0.55 | ProofDatabase caching | ✅ **Done** |
| v0.56 | SMT verification integration | ❌ Not started |
| v0.57 | Incremental verification | ❌ Not started |
| v1.0 | Full pipeline with incremental compilation | ❌ Not started |

### v0.55 Completed Integration

**Full CIR → PIR → MIR Pipeline**:

1. **CIR Integration** (`cir/to_mir_facts.rs`):
   - Extracts contract facts from explicit pre/post conditions
   - Converts CIR `Proposition` to MIR `ContractFact`

2. **PIR Integration** (`pir/propagate.rs`, `pir/to_mir_facts.rs`):
   - Propagates proofs through control flow (branch conditions, loops)
   - Extracts additional facts from:
     - If/else branch conditions
     - While/for loop invariants
     - Postconditions from function calls
   - Converts PIR `ProvenFact` to MIR `ContractFact`

3. **ProofDatabase Caching** (`verify/proof_db.rs`):
   - Saves proof cache to `.bmb.proofcache` file
   - Loads cached proofs for incremental compilation
   - Tracks cache hits/misses for optimization

4. **Build Pipeline** (`build/mod.rs`):
   - Full pipeline: AST → CIR → PIR → MIR
   - CIR + PIR facts merged with MIR's `ContractFact`
   - Proof-guided optimizations use augmented fact set
   - Proof cache saved after each build

### Future Work

```
1. SMT Verification Integration (v0.56)
   - Run CIR verification with Z3
   - Add verified proofs to ProofDatabase
   - Use verification results for stronger optimizations

2. Incremental Verification (v0.57)
   - Use IncrementalVerifier for changed-function-only verification
   - Leverage call graph for dependency tracking
   - Reduce verification time for large codebases
```

---

## Appendix: Mathematical Foundations

### Type System

BMB uses a Hindley-Milner type system extended with:
- **Refinement types**: `{x: i64 | x > 0}`
- **Dependent function types**: `(n: i64) -> [i64; n]`
- **Effect annotations**: `fn read() -> i64 @effect(Read)`

### Contract Logic

Contracts are expressed in first-order logic:

```
φ, ψ ::=
    | true | false           (constants)
    | e₁ = e₂                (equality)
    | e₁ < e₂ | e₁ ≤ e₂      (ordering)
    | ¬φ                      (negation)
    | φ ∧ ψ | φ ∨ ψ          (conjunction, disjunction)
    | φ → ψ                   (implication)
    | ∀x. φ | ∃x. φ          (quantification)
```

### Proof Rules

Key proof rules used in verification:

```
Γ ⊢ {P} S {Q}
─────────────────────────  (Precondition)
Γ ⊢ {P ∧ R} S {Q}

Γ ⊢ {P} S {Q}    Q → Q'
────────────────────────  (Postcondition Weakening)
Γ ⊢ {P} S {Q'}

Γ ⊢ {P ∧ B} S₁ {Q}    Γ ⊢ {P ∧ ¬B} S₂ {Q}
──────────────────────────────────────────  (Conditional)
Γ ⊢ {P} if B then S₁ else S₂ {Q}

Γ ⊢ {I ∧ B} S {I}
─────────────────────────  (Loop)
Γ ⊢ {I} while B do S {I ∧ ¬B}
```

---

## Summary

BMB's compiler architecture is built on one key insight: **proofs are optimization data, not just validation**.

By treating contract verification as a phase that produces reusable proof facts, BMB achieves what other languages cannot:

1. **Zero-overhead safety**: Bounds checks, null checks, and division checks are eliminated by proof, not runtime overhead
2. **Proof-guided optimization**: Optimizations that would be unsafe in C are proven safe in BMB
3. **Mathematical guarantees**: Not "probably works" but "mathematically proven"

This is the compiler architecture that makes BMB's philosophy—"Performance > Everything"—technically achievable.
