# AI-Native Development with BMB

> How BMB's design maximizes synergy with AI code generation

## Why BMB is AI-Native

BMB was designed with AI code generation in mind. Its contract system provides explicit specifications that AI can target, while the compiler verifies correctness—creating a powerful human-AI-compiler feedback loop.

```
┌─────────────┐    Specification    ┌─────────────┐
│   Human     │ ─────────────────► │     AI      │
│ (Contracts) │                     │ (Generator) │
└─────────────┘                     └──────┬──────┘
       ▲                                   │
       │                                   │ Code
       │ Feedback                          ▼
       │                            ┌─────────────┐
       └─────────────────────────── │  Compiler   │
            Verification Result     │ (Verifier)  │
                                    └─────────────┘
```

## Specification-First Development

### 1. Write the Contract

Human provides the specification:

```bmb
// Human writes this - the "what"
fn binary_search(arr: Vec<i64>, target: i64) -> i64
  pre is_sorted(arr)
  post ret == -1 or (ret >= 0 and ret < vec_len(arr) and vec_get(arr, ret) == target)
= /* AI implements */;
```

### 2. AI Generates Implementation

AI fills in the "how":

```bmb
fn binary_search(arr: Vec<i64>, target: i64) -> i64
  pre is_sorted(arr)
  post ret == -1 or (ret >= 0 and ret < vec_len(arr) and vec_get(arr, ret) == target)
= binary_search_impl(arr, target, 0, vec_len(arr));

fn binary_search_impl(arr: Vec<i64>, target: i64, lo: i64, hi: i64) -> i64 =
    if lo >= hi { -1 }
    else {
        let mid = lo + (hi - lo) / 2;
        let v = vec_get(arr, mid);
        if v == target { mid }
        else if v < target { binary_search_impl(arr, target, mid + 1, hi) }
        else { binary_search_impl(arr, target, lo, mid) }
    };
```

### 3. Compiler Verifies

```bash
$ bmb verify search.bmb
Verifying binary_search... ✓
Verifying binary_search_impl... ✓
All contracts verified.
```

## AI Query System

BMB's built-in query system (`bmb q`) is designed for AI tools:

### Symbol Discovery

```bash
# Find functions with specific contracts
bmb q fn --has-pre "x > 0"
bmb q fn --has-post "ret >= 0"

# Find by pattern
bmb q sym "parse*"
bmb q type "Vec*"

# Get function signatures
bmb q fn binary_search --format json
```

### Codebase Understanding

```bash
# Generate project index for AI consumption
bmb index

# Get project metrics
bmb q metrics

# Query with AI-optimized output
bmb q fn --format llm
```

### Example AI Workflow

```bash
# 1. AI asks about the codebase
$ bmb q fn --accepts "String" --returns "i64"
parse_int: String -> i64
string_len: String -> i64
find_char: String, i64 -> i64

# 2. AI gets function details
$ bmb q fn parse_int --format json
{
  "name": "parse_int",
  "params": [{"name": "s", "type": "String"}],
  "returns": "i64",
  "preconditions": [],
  "postconditions": [],
  "body_loc": "stdlib/string/parse.bmb:42"
}

# 3. AI generates code using existing patterns
```

## Prompt Engineering for BMB

### Effective Prompts

```
Generate a BMB function that:
- Takes a Vec<i64> and returns the sum
- Has precondition: vec_len(v) > 0
- Has postcondition: ret >= min_element(v) * vec_len(v)
- Uses tail recursion for efficiency

Example style:
fn example(x: i64) -> i64
  pre x > 0
  post ret >= x
= x * 2;
```

### Contract Templates

AI can use these patterns:

```bmb
// Numeric function template
fn numeric_op(x: i64) -> i64
  pre /* input constraint */
  post /* output guarantee */
= /* implementation */;

// Collection function template
fn collection_op(v: Vec<i64>) -> i64
  pre vec_len(v) > 0
  post /* result property */
= /* implementation */;

// Nullable result template
fn find_op(data: Vec<i64>, key: i64) -> i64
  post ret == -1 or /* found condition */
= /* implementation */;
```

## Error Correction Loop

When AI-generated code fails verification:

```bash
$ bmb verify ai_generated.bmb
Error: Postcondition violation in sort_array
  post is_sorted(ret) and vec_len(ret) == vec_len(arr)

  Counter-example found:
    arr = [3, 1, 2]
    ret = [1, 3, 2]  # Not sorted!
```

AI receives this feedback and regenerates:

```bmb
// AI's corrected version
fn sort_array(arr: Vec<i64>) -> Vec<i64>
  post is_sorted(ret) and vec_len(ret) == vec_len(arr)
= /* fixed implementation using proper sorting algorithm */;
```

## IDE Integration

BMB's LSP provides AI assistants with:

1. **Real-time diagnostics** - Contract violations as you type
2. **Completion suggestions** - Context-aware function recommendations
3. **Hover information** - Contract details for any symbol
4. **Refactoring support** - Safe rename with contract preservation

```json
// LSP response for AI consumption
{
  "diagnostics": [
    {
      "range": {"start": {"line": 10, "character": 0}},
      "message": "Precondition 'n > 0' may not hold",
      "severity": "error",
      "source": "bmb-verify"
    }
  ]
}
```

## Best Practices for AI-Assisted Development

### 1. Start with Contracts

```bmb
// Write this first
fn merge_sorted(a: Vec<i64>, b: Vec<i64>) -> Vec<i64>
  pre is_sorted(a) and is_sorted(b)
  post is_sorted(ret)
  post vec_len(ret) == vec_len(a) + vec_len(b)
= /* let AI implement */;
```

### 2. Use Descriptive Names

```bmb
// Good - AI understands intent
fn calculate_monthly_payment(principal: i64, rate: i64, months: i64) -> i64
  pre principal > 0
  pre rate >= 0
  pre months > 0
= /* AI generates financial calculation */;

// Bad - AI guesses
fn calc(p: i64, r: i64, m: i64) -> i64 = /* unclear intent */;
```

### 3. Provide Examples in Comments

```bmb
// Calculate nth Fibonacci number
// Examples: fib(0) = 0, fib(1) = 1, fib(10) = 55
fn fib(n: i64) -> i64
  pre n >= 0
= /* AI implements with examples as test cases */;
```

### 4. Iterative Refinement

```bmb
// v1: Basic contract
fn sort(arr: Vec<i64>) -> Vec<i64>
  post is_sorted(ret)
= /* AI generates */;

// v2: Add stability requirement
fn sort(arr: Vec<i64>) -> Vec<i64>
  post is_sorted(ret)
  post is_permutation(arr, ret)
= /* AI regenerates with stricter contract */;

// v3: Add performance hint
fn sort(arr: Vec<i64>) -> Vec<i64>
  post is_sorted(ret)
  post is_permutation(arr, ret)
= merge_sort(arr);  // Human hints algorithm choice
```

## Comparison with Other Approaches

| Approach | Specification | Verification | AI Fit |
|----------|---------------|--------------|--------|
| Unit Tests | Implicit (examples) | Runtime | Medium |
| Type Systems | Types only | Compile-time | Medium |
| Doc Comments | Natural language | None | Low |
| **BMB Contracts** | Formal, precise | Compile-time | **High** |

## Future: AI-Native Ecosystem

### Planned Features

1. **`bmb q ctx`** - Generate AI context from codebase
2. **`bmb q sig`** - Search by signature pattern
3. **`--format llm`** - LLM-optimized output
4. **`bmb q batch`** - Bulk queries for AI agents

### Vision

```
Human Intent → BMB Contract → AI Implementation → Compiler Verification → Correct Code
      ↑                                                      │
      └──────────────────── Feedback ◄───────────────────────┘
```

## Getting Started with AI + BMB

1. **Write contracts first** - Define what you want
2. **Use bmb query** - Help AI understand your codebase
3. **Let AI implement** - Generate the code
4. **Trust the verifier** - Compiler catches AI mistakes
5. **Iterate quickly** - Fast feedback loop

## Next Steps

- [Contracts](CONTRACTS.md) - Master specification writing
- [Performance](PERFORMANCE.md) - Ensure AI code is fast
- [From Rust](FROM_RUST.md) - Patterns AI can learn from
