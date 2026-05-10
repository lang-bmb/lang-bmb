# Cycle 2670-2673: M5-5c 구현 — fn() -> Array<String> dispatch ✅
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2669): M5-5c 옵션 A 구현. ⚪ Plan 유효, scope 그대로.

CLAUDE.md Rule 1 (구현 우선): 진단/계획 짧게, 즉시 구현 진입. 한 사이클에 진단 + 구현 + 첫 골든 + 변형 검증까지 완료.

## Scope & Implementation

### 1. 근본 원인 (Cycle 2670 진단)

`fn make_strs() -> Array<String>`이 MIR에서 `i64`로 ret_type 변환됨.

**계층별 원인 분리**:
| 위치 | 원인 |
|------|------|
| **`parse_return_type` (compiler.bmb 2617)** | TK_IDENT generic ret 케이스 누락 → 무조건 `"i64"` fallback |
| **`get_fn_return_scan` (compiler.bmb 6505)** | sexp 형식 `Array<String>`이 `<`에서 ident 끊김 (read_sexp_at) |
| **`collect_string_fns_from_mir`** | ret_type=="String"만 수집, "Array<String>" 미지원 |
| **`llvm_gen_call_with_string_tracking_sb_reg`** | array 반환 fn dispatch 없음 (push_str_ptr_marker 미발행) |

### 2. 4-점 수정 (Cycle 2671)

**(a) parse_return_type — `Array<String>` 인식**:
```bmb
else if tok_kind(t4) == TK_IDENT() {
    let name = get_ident_text(src, pos, t4);
    let t5 = next_token_raw(src, ne);
    if name == "Array" and tok_kind(t5) == TK_LT() {
        let t6 = next_token_raw(src, tok_end(t5));
        if tok_kind(t6) == TK_STRING_TYPE() {
            let t7 = next_token_raw(src, tok_end(t6));
            if tok_kind(t7) == TK_GT() {
                pack_result(skip_nullable(src, tok_end(t7)), "Array<String>")
            } else { ... fallback i64 ... }
```

**(b) get_fn_return_scan — 두 토큰 합성**:
```bmb
else if child == "Array" {
    let next_sexp = read_sexp_at(content, after);
    if next_sexp == "<String>" { "Array<String>" }
    else { recurse }
}
```

**(c) collect_string_fns_acc — 통합 registry (`A:` prefix)**:
```bmb
let entry = if ret_type == "String" { fn_name }
    else if ret_type == "Array<String>" { "A:" + fn_name }
    else { "" };
```

**(d) is_dynamic_string_array_fn + llvm_gen_call dispatch**:
```bmb
fn is_dynamic_string_array_fn(fn_name, string_fns) -> bool =
    if list 비었으면 false else { check_array_fn_in_list ... };
// llvm_gen_call_with_string_tracking_sb_reg
let is_str_array_fn = is_dynamic_string_array_fn(fn_name, string_fns);
let w = if ... is_str_array_fn { push_str_ptr_marker(str_sb, dest) } ...
```

**핵심 설계 결정**: signature 확장 대신 기존 `string_fns` 리스트에 `A:` prefix로 임베드. caller chain ~20 함수 signature 변경 회피.

### 3. 골든 테스트 (Cycle 2672-2673)

| 골든 | 시나리오 | 결과 |
|------|---------|------|
| `test_golden_arr_str_fn_return.bmb` | `let arr = fn(); println(arr[i])` | exit 42, "foo\nbar\n" ✅ |
| `test_golden_arr_str_fn_return_alias.bmb` | `let arr2 = arr` alias propagation | exit 42, "apple\nbanana\n" ✅ |
| `test_golden_arr_str_fn_return_loop.bmb` | `for i in 0..3 { println(arr[i]) }` | exit 42, "one\ntwo\nthree\n" ✅ |

Golden count: 2851 → **2854**.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (회귀 없음) |
| Stage 1 빌드 | ✅ OK (10.5s) |
| M5-5 골든 5개 (이전) | ✅ 모두 PASS |
| M5-5c 골든 3개 (신규) | ✅ 모두 PASS |
| 변형 시나리오 (alias, for-loop, alloca) | ✅ 모두 PASS |

## Reflection

**Scope fit**: 의도 = M5-5c 구현. 결과 = 한 사이클 클러스터에 진단 + 구현 + 변형 검증까지 완료. CLAUDE.md Rule 1 (구현 우선) 효과 ✅.

**Latent defects**:
- (제로) — 모든 M5-5 회귀 통과
- M5-5d (`p.field[i]` struct field array) 여전히 미지원 — Cycle 2674에서 진단

**Structural improvement opportunities**:
- `parse_return_type`이 다른 generic ret types (`Array<i64>`, `Result<T,E>` 등)도 fallback `i64` — 필요 시 동일 패턴 확장
- `read_sexp_at`이 `<`에서 끊기는 문제 — `Array<String>` 외 generic은 두 토큰 매핑 필요 (현재는 한 케이스만 처리)
- `string_fns` 리스트가 단일 카테고리에서 카테고리화 (`A:`) — 추가 카테고리 (`R:` Result, `M:` Map 등) 확장 시 깨끗
- `lookup_fn_ret_raw`(raw BMB type) 신규 — 큰 변경이지만 장기적으로 더 깨끗 (M6 후보)

**Philosophy drift 점검**:
- "Workaround 없는 근본 해결" — parse_return_type / get_fn_return_scan 계층별 정확 위치 수정 (4-point fix) ✅
- "복잡도는 기피 사유 아니다" — 4-point fix가 각각 다른 계층, 모두 처리 ✅
- "출력 디폴트 = AI 친화" — string_fns의 `A:` prefix는 기계 파싱 친화 ✅
- "AI-native 언어 확장" — 자연 패턴 `let arr = make_strs(); println(arr[0])` 작동 ✅

**Roadmap impact**:
- M5-5 매트릭스 5/7 → **6/7** (fn-return 추가)
- M5-5d (struct field) 잔여 — 같은 패턴 (struct field type registry) 적용 검토
- M3 ~96% 유지

**User-facing quality**:
- LLM이 가장 자연스럽게 작성할 패턴 `["a", "b"]` 반환 함수 동작 ✅
- 명시적 type annotation (`Array<String>`) 필요 — BMB의 explicit 철학 일관

## Carry-Forward
- Actionable:
  - Cycle 2674: M5-5d 진단 (`struct S { tags: Array<String> }` field dispatch)
  - Cycle 2675: M5-5d 구현 시도 (struct field type registry — 동일 `A:` 패턴 미러 가능?)
- Structural Improvement Proposals:
  - `parse_return_type` 다른 generic 패턴 확장 (`Result<T,E>`, `Array<i64>`) — 필요 시
  - `string_fns` 카테고리화 일반화 (M6 후보)
  - `read_sexp_at` `<`/`>` 처리 일반화 (모든 generic AST 통합)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 매트릭스 6/7 (fn-return 추가) — Cycle 2676/2677에서 ROADMAP/HANDOFF 갱신
- Next Recommendation: Cycle 2674 — M5-5d 진단
