# BMB AI-Native 루프 사이클 기반 개선 가이드

> **"AI가 BMB 코드를 몇 번 만에 올바르게 작성하는가?"**
> 이 지표가 BMB의 AI-native 품질을 측정하는 핵심 척도다.

---

## 1. 핵심 지표: AI Loop Count

```
자연어 문제 → LLM 코드 생성 → 빌드 → 에러 → 수정 → 빌드 → ... → 성공
                                    ↑                ↑
                                  Loop 1           Loop 2
```

**Loop Count = LLM이 올바른 BMB 코드를 생성하기까지 시도한 횟수**

| Loop Count | 판정 | 의미 |
|-----------|------|------|
| 1 | EXCELLENT | 첫 시도에 성공. AI에게 완벽히 친화적 |
| 2 | GOOD | 1회 에러 + suggestion으로 즉시 수정 |
| 3-5 | NEEDS WORK | suggestion이 부족하거나 없음 |
| 6-10 | POOR | 도구 지원 부재. 패턴 추가 필요 |
| 11 (FAIL) | CRITICAL | AI가 해결 불가. 언어/도구 수준 변경 필요 |

---

## 2. 증명된 방법론

### 2.1 개선 사이클

```
┌──────────────────────────────────────────┐
│  1. 측정 — 실험 실행, Loop Count 수집     │
└────────────────┬─────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  2. 분석 — 에러 유형별 분류 (A/B/C/D)    │
│     어떤 에러에서 루프를 소진하는가?       │
└────────────────┬─────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  3. 진단 — 에러별 근본 원인 파악          │
│     suggestion 있는데 실패? → 힌트 부족   │
│     suggestion 없음? → 패턴 미등록        │
│     문법 자체가 문제? → 언어 스펙 변경     │
└────────────────┬─────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  4. 조치 — 수준별 대응                    │
│     Level 1: PatternBank 패턴 추가        │
│     Level 2: 에러 메시지 개선             │
│     Level 3: 컴파일러/언어 스펙 변경      │
│     Level 4: MCP 도구 추가               │
└────────────────┬─────────────────────────┘
                 ↓
┌──────────────────────────────────────────┐
│  5. 검증 — 동일 실험 재실행              │
│     Loop Count 감소 확인                  │
└──────────────────────────────────────────┘
```

### 2.2 실증 데이터

**Before (PatternBank 없음)**:
```
binary_search:  8 loops (Type C ×7)    → 90% 문법 에러에서 소진
quicksort:      10 loops (Type C ×7)   → FAIL 빈번
평균: 5.7 loops, 성공률 83%
```

**After (PatternBank + suggestion)**:
```
binary_search:  2 loops (Type C ×1)    → suggestion으로 1회 수정
quicksort:      2 loops (Type C ×1)    → suggestion으로 1회 수정
평균: 2.0 loops, 성공률 100%
```

**82% 루프 감소, 83%→100% 성공률.**

---

## 3. 에러 유형 분류 체계 (Loop Types)

모든 루프를 4가지 유형으로 분류:

| 유형 | 코드 | 발생 시점 | 원인 | 개선 방법 |
|------|------|----------|------|----------|
| **A — Contract** | `A` | 컴파일 | pre/post 계약 위반 | 이것이 BMB의 가치. 줄이지 않음 — 측정만 |
| **B — Syntax** | `B` | 파서 | BMB 문법 오류 | PatternBank 패턴 + bmb_reference 강화 |
| **C — Semantic** | `C` | 타입체커 | Rust-ism, 타입 에러 | PatternBank suggestion + 예제 |
| **D — Test Fail** | `D` | 런타임 | 로직 에러 | 문제 세트 설계, 테스트 피드백 개선 |

### 3.1 유형별 개선 전략

**Type A (Contract)** — 줄이지 않는다
- 이것은 BMB의 존재 이유. 계약이 런타임 에러를 컴파일타임으로 끌어올린 증거
- H1 실험에서 "BMB-contract의 Type A"가 "BMB-nocontract의 Type D"로 이동했는지 측정
- **목표**: Type A가 존재하고, Type D가 감소하는 것

**Type B (Syntax)** — PatternBank으로 해결
- LLM이 BMB 문법을 모르는 것이 원인
- 파일럿에서 `for`, `impl`, `use` 등 Rust 키워드 사용이 빈번
- **조치**: 파서 에러에 "BMB에서는 X를 사용하세요" suggestion 추가
- **목표**: Type B → 0

**Type C (Semantic)** — 가장 큰 개선 영역
- `expected i64, got ()`, `unknown function Vec::new()` 등
- PatternBank suggestion이 가장 효과적 (82% 감소 달성)
- **조치**: 새로운 Type C 에러 발견 시 PatternBank 패턴 즉시 추가
- **목표**: Type C ≤ 1 per problem

**Type D (Test Fail)** — 문제 설계로 해결
- 코드는 컴파일되지만 로직이 틀림
- BMB 특유 문제 아님 — 모든 언어에서 발생
- **조치**: 테스트 실패 피드백에 "어떤 입력에서 실패했는지" 명확히 포함
- **목표**: Type D ≤ 1 per problem

---

## 4. PatternBank 패턴 추가 절차

### 4.1 새 패턴 발견

실험 결과에서 **suggestion=NO**인 에러를 추출:

```bash
cd ecosystem/ai-proof
python -c "
import json, glob
for f in sorted(glob.glob('results/raw/*/*/*.json')):
    d = json.loads(open(f).read())
    for a in d.get('attempts', []):
        if a.get('error') and not a['error'].get('suggestion'):
            print(f'{a[\"loop_type\"]} | {a[\"error\"][\"normalized\"][:80]}')
"
```

### 4.2 패턴 작성

`bmb/src/diagnostics/patterns.rs`에 추가:

```rust
DiagPattern {
    id: "descriptive_id",
    kind: "type",  // "parser", "type", "resolve", or "" for any
    triggers: &["specific substring in error message"],
    suggestion: "BMB에서는 X를 사용합니다",
    example_wrong: "AI가 쓸 법한 잘못된 코드",
    example_correct: "올바른 BMB 코드",
},
```

**트리거 규칙**:
- 에러 메시지에 나타나는 **구체적** 문자열 사용
- `"for "` (너무 넓음) → `"\`for\`"` (정확함)
- kind 필터로 false positive 방지
- 대소문자 무시됨 (lowercase 비교)

### 4.3 검증

```bash
# 1. 컴파일
cargo build --release

# 2. 단위 테스트 (선택)
cargo test --test diagnostics_test --release

# 3. 전체 테스트 (회귀 방지)
cargo test --release

# 4. 실험 재실행
cd ecosystem/ai-proof && python scripts/run_experiment.py --pilot --runs 1
```

### 4.4 효과 측정

**패턴 적중률 = suggestion이 포함된 에러 / 전체 에러**

```bash
python -c "
import json, glob
total = 0; with_sug = 0
for f in sorted(glob.glob('results/raw/*/*/*.json')):
    d = json.loads(open(f).read())
    for a in d.get('attempts', []):
        if a.get('error'):
            total += 1
            if a['error'].get('suggestion'): with_sug += 1
print(f'Pattern hit rate: {with_sug}/{total} = {with_sug/total*100:.0f}%')
"
```

**목표: ≥ 80% 적중률**

---

## 5. 개선 수준 (Escalation Ladder)

문제가 PatternBank만으로 해결되지 않을 때 상위 수준으로 에스컬레이션:

```
Level 1: PatternBank 패턴 추가        ← 대부분 여기서 해결 (분 단위)
    ↓ 해결 안 됨
Level 2: 에러 메시지 자체 개선         ← 컴파일러 에러 텍스트 수정 (시간 단위)
    ↓ 해결 안 됨
Level 3: 컴파일러/언어 스펙 변경       ← 새 문법, 새 내장 함수 (일 단위)
    ↓ 해결 안 됨
Level 4: MCP 도구 추가                ← bmb-mcp에 새 도구 (주 단위)
```

### 수준별 예시

| 문제 | 수준 | 조치 |
|------|------|------|
| `Vec::new()` 사용 | L1 | `vec_generic` 패턴 추가 |
| "unknown token" 에러가 모호함 | L2 | 에러 메시지에 가능한 토큰 나열 |
| while 루프가 장황함 | L3 | for 루프 문법 추가 검토 |
| LLM이 stdlib API를 모름 | L4 | bmb_example MCP 도구 구현 |

### 수준 판단 기준

```
Q: suggestion을 추가하면 해결되는가?
   YES → Level 1

Q: 에러 메시지가 근본적으로 불명확한가?
   YES → Level 2

Q: BMB 문법 자체가 AI에게 불필요하게 어려운가?
   YES → Level 3 (언어 스펙 변경 검토)

Q: 실시간 문맥이 필요한가? (stdlib 탐색, 코드 분석 등)
   YES → Level 4 (MCP)
```

---

## 6. 실험 확장 로드맵

### Phase 1: 파일럿 완료 ✅

```
3 문제 × 2 조건 × 1 회 = 6 실험
결과: 100% 성공, 2.0 loops, Type C 82% 감소
```

### Phase 2: 전체 문제 세트 — 인프라 준비 완료 ✅

```
30 문제 × 4 조건 × 3 회 = 360 실험
현재 상태:
  ✅ 27개 문제 추가 완료 (알고리즘 10 + 시스템 10 + 계약 10)
  ✅ 모든 BMB 솔루션 build+test 통과 (30/30, 390 테스트)
  ✅ PatternBank 27 패턴 (검증 완료, 7개 오류 패턴 제거)
  ✅ bmb_reference.md 강화 (현대 BMB 문법 반영)
  ✅ 솔루션 현대화 (for/break/return/type inference)
  ✅ E2E 파이프라인 mock 검증 (30/30 PASS)
  ⬜ 실험 실행 (LLM API 필요)
  ⬜ H1 + H2 실행
  ⬜ 통계적 유의성 검증 (Wilcoxon signed-rank)
```

**문제 카테고리**:
- 알고리즘 (01-10): binary_search, quicksort, merge_sort, fibonacci, gcd, matrix_multiply, max_subarray, two_sum, insertion_sort, reverse_array
- 시스템 (11-20): stack, queue, linked_list_sum, count_frequency, min_max, digit_sum, histogram, running_average, prefix_sum, matrix_transpose
- 계약 (21-30): bounded_array, safe_divide, bounded_sum, sorted_insert, range_clamp, safe_sqrt, matrix_safe_access, positive_factorial, bounded_stack, contract_chain

**예상 발견**: 새 문제에서 새로운 Type C 에러 패턴 발견 → PatternBank 확장

### Phase 3: 다중 LLM

```
30 문제 × 4 조건 × 3 회 × 3 LLM = 1,080 실험
목표: "BMB의 AI 친화성은 특정 LLM에 의존하지 않음" 증명
```

### Phase 4: bmb-mcp 통합

```
동일 실험을 MCP 도구 유무로 비교
목표: MCP가 Loop Count를 추가 감소시키는지 측정
```

---

## 7. 지속적 측정 체계

### 7.1 CI 통합 (향후)

```yaml
# .github/workflows/ai-native.yml
- name: AI Loop Regression Check
  run: |
    python ecosystem/ai-proof/scripts/run_experiment.py --pilot --runs 1
    python -c "
    import json
    data = json.load(open('ecosystem/ai-proof/results/summary.json'))
    # Check no regression from baseline
    "
```

### 7.2 대시보드 지표

| 지표 | Pilot 결과 | Phase 2 목표 |
|------|-----------|-------------|
| 평균 Loop Count | 2.0 | ≤ 2.0 |
| 성공률 | 100% (3/3) | ≥ 95% (30 문제) |
| Type C 비율 | 100% (1/1) | ≤ 50% |
| PatternBank 패턴 수 | 27 | 유지 + 실험 후 추가 |
| PatternBank 적중률 | 100% (6/6) | ≥ 80% |
| Type A (contract) | 0% | 측정 (줄이지 않음) |
| 문제 세트 | 3 | 30 (10+10+10) ✅ |
| 테스트 케이스 | 45 | 390 ✅ |
| 파이프라인 검증 | - | 30/30 mock PASS ✅ |

### 7.3 회귀 감지

Loop Count가 이전보다 증가하면:
1. 새 에러 메시지가 추가/변경되었는가? → 트리거 업데이트
2. LLM이 업데이트되었는가? → 새 실수 패턴 발견 필요
3. 문제가 더 어려워졌는가? → 예상된 증가, 수용

---

## 8. bmb-mcp 승격 경로

현재 컴파일러에 내장된 기능이 bmb-mcp의 기반이 됩니다:

```
현재 (컴파일러 내장)                    bmb-mcp (MCP 서버)
────────────────────────────────────────────────────────
PatternBank                            bmb_example tool
  find_patterns(kind, msg)      →        "BMB에서 정렬은 어떻게?"
  23개 패턴, suggestion+example           카테고리별 예제 검색

report_error_machine                    bmb_check tool
  enriched JSONL                →        코드 제출 → JSON 에러+힌트 반환
  suggestion 포함                         MCP resource로 노출

bmb_reference.md                        bmb_spec_lookup tool
  문법 치트시트               →        "BMB의 while 문법은?"
                                         자연어 쿼리 → 관련 스펙 반환

ai-proof orchestrator                   MCP 클라이언트 레퍼런스
  check → build → test loop    →        다른 AI 도구가 참조할 구현
```

**전환 비용 최소**: 컴파일러 기능을 MCP 프로토콜로 래핑만 하면 됩니다.

---

## 9. Decision Framework — AI Loop 관점

문제 발견 시 **AI Loop Count 영향도**로 우선순위 판단:

| 순위 | 질문 | 예시 |
|------|------|------|
| 1 | **이 변경이 Loop Count를 줄이는가?** | PatternBank 패턴 → 직접 감소 |
| 2 | **이 변경이 FAIL을 PASS로 바꾸는가?** | 에러 메시지 개선 → 간접 감소 |
| 3 | **이 변경이 Type A를 증가시키는가?** | 계약 검증 강화 → BMB 가치 증가 |
| 4 | **이 변경이 성능을 개선하는가?** | 코드젠 최적화 → 복합 스코어 향상 |

**낮은 순위에서 해결하려는 유혹을 경계하라.**
Loop Count가 높은 문제를 성능 최적화로 해결하려 하지 마라.

---

## 부록: 현재 PatternBank 목록 (27 패턴)

> **2026-03-25 검증 완료**: BMB 컴파일러 현재 상태 기준으로 모든 패턴 검증.
> 7개 패턴 제거됨 — BMB가 이제 for/break/continue/return/-1/x=5/type inference 지원.

### 활성 패턴 (27개)

| ID | Kind | 대상 | Trigger 예시 |
|----|------|------|-------------|
| option_type | any | Option → T? | `Option<`, `Some(` |
| vec_generic | any | Vec → vec_new | `Vec<`, `Vec::new` |
| method_call | any | .method() → func() | `.len()`, `.push(` |
| println_macro | any | println! → println | `println!` |
| string_type | any | String → &str | `String::from` |
| type_annotation | type | 타입 추론 실패 시 | `cannot infer` |
| fn_return_expr | parser | { expr } → = expr; | `expected \`=\`` |
| bitwise_ops | any | & → band | `bitwise` |
| impl_block | parser | impl → free fn | `` `impl` `` |
| trait_def | parser | trait syntax | `` `trait` `` |
| tuple_destruct | any | let (a,b) → separate | `let (` |
| match_wildcard | any | _ => → else | `_ =>` |
| static_method | any | ::new() → func() | `::new(` |
| io_functions | any | std::io → read_int() | `io::stdin` |
| array_syntax | any | [T;N] → vec_new | `[i64;` |
| use_import | parser | use → import | `` `use` `` |
| void_return_used | type | () → i64 wrapper | `expected i64, got ()` |
| unit_to_value | type | () return | `expected (), got i64` |
| underscore_pattern | parser | _ → named var | `` `_` `` |
| missing_semicolon | parser | } → }; | `expected \`}\`` |
| missing_else | parser | if without else | `Expected one of "else"` |
| closure_lambda | parser | \|x\| → fn | `` `\|` `` |
| mutable_param | any | &mut → local copy | `&mut` |
| print_string_fn | type | println(str) → println_str | `expected &str, got i64` |
| if_without_else_unit | type | if {} → if {} else {()} | `branch types do not match` |
| iterator_methods | any | .iter()/.map() → for loop | `.iter()`, `.map(` |
| type_cast | any | as usize → i64 | `as usize`, `as i64` |

### 제거된 패턴 (7개 — BMB가 이제 지원)

| 제거된 ID | 이유 |
|-----------|------|
| for_loop | BMB가 `for i in 0..n { }` 지원 |
| break_continue | BMB가 `break`, `continue` 지원 |
| return_keyword | BMB가 `return expr` 지원 |
| reassign_set | BMB가 `x = 5` (set 없이) 지원 |
| negative_literal | BMB가 `-1` 지원 |
| range_syntax | BMB가 `0..n` 범위 지원 |
| bool_literal | BMB가 `true`/`false` 지원 |
