# BMB Session Handoff — 2026-05-15 (Cycles 2841-2850 — 언어 갭 해소 2차)

> **HEAD**: `ea584bab` (Cycle 2850 session close)
> **이전 HEAD**: `38f84ebd` (Cycles 2834-2840)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2822, 120790 lines) — 이번 세션 bootstrap 변경 없음
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2851

---

## 이번 세션 작업 요약 (Cycles 2841-2850)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2841 | for-in-vec 구현 (M4-10) | `for x in vec_handle {}` — interpreter-only, types + interp + grammar |
| 2842 | String interpolation | `"Hello {name}"` → `format("Hello {0}", name)` desugar (grammar + ast/expr.rs) |
| 2843 | `{0}` numeric placeholder | format-arg passthrough + 기존 numeric interp 보존 |
| 2844 | compound assignment +=/-=/*=//= | 4종 연산자. grammar.lalrpop BlockStmt desugar |
| 2845 | %=, {{ escape + flaky fix | %=5번째 연산자 + `{{`/`}}` literal brace + test_index_write_and_read 경쟁 조건 수정 |
| 2846 | str_hashmap 6종 완성 | 기존 v0.90.83 스텁 → 완전 구현. type signature 수정 (String key) |
| 2847 | `set obj.field += e` 필드 복합 할당 | BlockExpr desugar 방식으로 LR(1) 충돌 해결 |
| 2848 | `{expr}` 보간 | 산술/필드접근/단항 지원. InterpMini 미니파서 내장 |
| 2849 | str_hashmap_keys/sorted_keys | SVEC_REGISTRY 재활용 → svec handle 반환 |
| 2850 | svec_new/push + str_hashmap_inc | svec API 완성 + atomic word-freq increment |

### 테스트 변화
2370 → **2375** (+5 integration tests)

---

## M4 ① 언어 갭 현황 (2850 기준)

| 기능 | 상태 |
|------|------|
| let-tuple | ✅ Cycle 2621 |
| static method | ✅ Cycle 2620 |
| Option::Some expr | ✅ Cycle 2633 |
| if-without-else | ✅ Cycle 2822 |
| else-if-chain | ✅ Cycle 2823 |
| 7종 string builtins | ✅ Cycle 2828, interpreter-only |
| to_string\<T\> | ✅ Cycle 2830, interpreter-only |
| str_split + svec_\* | ✅ Cycle 2833, interpreter-only |
| while let PAT = expr {} | ✅ Cycle 2834, interpreter-only |
| format(template, ...args) | ✅ Cycle 2835, interpreter-only |
| vec_sum/max/min/sort | ✅ Cycle 2836, interpreter-only |
| str_replace + str_repeat | ✅ Cycle 2837, interpreter-only |
| svec_join + vec_contains + vec_index_of | ✅ Cycle 2838, interpreter-only |
| for-in-vec | ✅ Cycle 2841, interpreter-only |
| String interpolation `"Hello {name}"` | ✅ Cycle 2842, interpreter-only |
| compound assignment `+=/-=/*=//=/%=` | ✅ Cycle 2844-2845, interpreter+native |
| str_hashmap 6종 (String→i64 HashMap) | ✅ Cycle 2846, interpreter-only |
| `set obj.field += e` 필드 복합 할당 | ✅ Cycle 2847, interpreter-only |
| `{expr}` 복잡 표현식 보간 | ✅ Cycle 2848, interpreter-only |
| str_hashmap_keys/sorted_keys | ✅ Cycle 2849, interpreter-only |
| svec_new/push + str_hashmap_inc | ✅ Cycle 2850, interpreter-only |

---

## 변경 파일 (이번 세션 전체)

**Rust 소스**:
- `bmb/src/grammar.lalrpop`: for-in-vec grammar + compound assignment BlockStmt 5종 + `{expr}` interp이 이미 Expr에 desugar됨 + BlockExpr 필드복합할당 5종
- `bmb/src/ast/expr.rs`: `desugar_string_interp` (ident→complex expr 확장) + `InterpMini` 미니파서 (~120줄)
- `bmb/src/interp/eval.rs`: for-in-vec eval + str_hashmap 6종 + svec_new/push + str_hashmap_keys/sorted_keys + str_hashmap_inc
- `bmb/src/types/mod.rs`: 위 모든 빌트인 타입 서명 등록
- `bmb/tests/integration.rs`: 5개 test 함수 추가 (field_compound, str_interp_expr, str_hashmap_keys, svec_new_push, str_hashmap_inc)

**문서**:
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 필드복합할당 패턴 + {expr} 보간 패턴 + str_hashmap_keys 순회 패턴 + svec_new/push + str_hashmap_inc 갱신
- `claudedocs/ROADMAP.md`: 최신화 (Cycle 2850까지)

---

## 다음 세션 우선순위

### Structural Improvement Proposals (Carry-Forward)
1. **`for x in svec {}`** — `Value::SvecHandle(usize)` 별도 값 타입 필요. 현재 svec/vec 모두 `i64`라 구분 불가.
2. **`{fn_call(args)}` 보간** — InterpMini에 함수 호출 파싱 추가. args 재귀적으로 복잡.
3. **필드 복합 할당 native 지원** — `set obj.field += e`가 codegen에서도 동작하도록 확장.
4. **`str_hashmap_delete(map, key)`** — 키 삭제 기능.

### 다음 권장 작업
- **B축 재측정** (언어 갭 충분히 해소됨 — baseline 2026-08-13 stale 전에 재측정 고려)
- 또는 계속 언어 갭 해소 (위 Structural Proposals)

---

## 기술 인사이트 (다음 세션 참고)

1. **LR(1) 충돌 패턴**: `"set" RawIdent "." Ident "+="` 형태를 BlockStmt에 직접 추가 시 shift-reduce 충돌. 해결책: `BlockExpr`에서 `SpannedUnaryExpr` 전체 파싱 후 연산자 토큰으로 분기.

2. **desugar_string_interp는 파싱 타임 호출**: grammar action에서 직접 호출됨 → 서브-파서 내장 필요. `InterpMini` 재귀 하강 파서가 분리된 구조체로 깔끔하게 동작.

3. **svec vs vec 핸들 구분 불가**: 두 핸들 모두 `i64`. svec 핸들은 작은 정수(registry index), vec 핸들은 큰 포인터값이지만 타입 시스템상 구분 없음. `Value::SvecHandle(usize)` 추가 시 해결 가능.

4. **str_hashmap raw ptr + SVEC_REGISTRY 혼합**: str_hashmap은 `Box<HashMap<String, i64>>` raw ptr (heap), svec는 `thread_local` SVEC_REGISTRY index. 두 방식의 lifetime 관리가 다름 — free 순서 중요.
