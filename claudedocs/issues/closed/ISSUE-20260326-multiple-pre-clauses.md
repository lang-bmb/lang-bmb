# Multiple `pre` Clauses Not Supported (Parser Limitation)

**Status: OPEN** — Cycle 2730 양식 표준화. 언어 spec 변경 필요
**Priority: LOW**
**Category: Language Design / Parser**

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | n/a (parser limitation — feature spec) |
| `stale_after` | n/a (언어 spec 변경 전까지 유효) |
| `measurement_source` | `bmb/src/parser/grammar.lalrpop` (single `pre` rule) |
| `observed_rate` | n/a (workaround: `and` chain) |
| `scope` | 모든 BMB fn with multiple pre/contract |
| `env_hash` | n/a (parser 정적 제한) |

## Summary
BMB only supports a single `pre` clause per function. Multiple `pre` lines cause a parser error. Users must combine with `and`.

## Reproduction
```bmb
// FAILS: parser error
fn safe_get(v: i64, idx: i64, len: i64) -> i64
    pre idx >= 0
    pre idx < len
= vec_get(v, idx);

// WORKS: combined with 'and'
fn safe_get(v: i64, idx: i64, len: i64) -> i64
    pre idx >= 0 and idx < len
= vec_get(v, idx);
```

Error: `Unrecognized token 'pre' found... Expected one of "post", "and", "or", ...`

## Impact
- Low — `and` combinator is a clean workaround
- LLM-generated code sometimes uses multiple `pre` lines (natural pattern)
- 3 AI-Bench contract problems had to be fixed

## Proposed Fix
Either:
1. **Parser**: Allow multiple `pre` clauses, desugar to `and` conjunction
2. **Documentation**: Clearly state single `pre` + `and` pattern in reference

## Acceptance Criteria
- [ ] Either parser support or clear documentation
- [ ] BMB Reference updated with correct contract syntax

## Context
Discovered during AI-Bench problem 96-98 creation (Cycles 2286-2305).

---

## Close Resolution (Cycle 2756, 2026-05-12)

**Closed** via acceptance criterion (2) — documentation update.

근거:
- Rule 6 (CLAUDE.md) "Rust 새 기능 추가 ❌ 금지" — 파서 변경 회피
- 본 ISSUE 명시: priority LOW, impact "low (and combinator 깨끗한 workaround)"
- ROADMAP Drift C 3 갭 (let-tuple/static-method/Option-expr) 모두 해소됨 — multiple-pre는 그 셋과 별개의 부수적 갭
- 권장 대안 `where { }` block style이 이미 multiple named contracts를 지원 — 신규 코드의 정답

**Documentation 변경**:
- `docs/LANGUAGE_REFERENCE.md` § 10.4 (Legacy Pre/Post) — 다중 pre clauses 제약 + `and` 조합 + `where { }` 권장 예시 추가
- 본 변경으로 acceptance criteria "BMB Reference updated with correct contract syntax" 충족

**향후 재고 조건**:
- BMB가 Rust 컴파일러 졸업 (compiler.bmb 단독) → 그 시점 compiler.bmb 파서에 add 가능
- 외부 사용자가 multiple `pre`를 요구하는 use case 추가 발생 시
- LLM-generated code에서 빈도 높은 패턴 발견 시 (현 ai-bench 측정에서는 3 문제, 낮음)
