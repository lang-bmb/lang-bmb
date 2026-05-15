# BMB Session Handoff — 2026-05-15 (Cycles 2851-2860 — 언어 갭 해소 3차)

> **HEAD**: TBD (Cycle 2860 commit 후 갱신)
> **이전 HEAD**: `ea584bab` (Cycles 2841-2850)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2822, 120790 lines) — 이번 세션 bootstrap 변경 없음
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2861

---

## 이번 세션 작업 요약 (Cycles 2851-2860)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2851 | str_hashmap_delete + str_hashmap_update | 키 삭제 + 값 덮어쓰기 (11종 완성) |
| 2852 | str_to_upper / str_to_lower / str_char_at | Unicode case변환 + 단문자 String 반환 |
| 2853 | vec_remove / vec_reverse / vec_fill | vec API 갭 해소 (19종 완성) |
| 2854 | svec_sort / svec_contains / svec_remove / svec_clear | svec API 완성 (10종) |
| 2855 | `{fn_call(args)}` 보간 | InterpMini primary() 확장 → Expr::Call 생성 |
| 2856 | pow_i64 / clamp_i64 / gcd_i64 | 정수 수학 builtins (free function) |
| 2857 | str_count / str_pad_left / str_pad_right | 문자열 포맷팅 유틸리티 |
| 2858 | dead_code 제거 + ROADMAP 갱신 | InterpMini::consume() 제거, M4-12 추가 |
| 2859 | HANDOFF 갱신 | 이번 문서 |
| 2860 | 최종 검증 + 커밋 | 전체 테스트 재확인 + git commit |

### 테스트 변화
2375 → **2382** (+7 integration tests)

---

## M4 ① 언어 갭 현황 (2860 기준)

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
| str_hashmap_delete + str_hashmap_update | ✅ Cycle 2851, interpreter-only |
| str_to_upper / str_to_lower / str_char_at | ✅ Cycle 2852, interpreter-only |
| vec_remove / vec_reverse / vec_fill | ✅ Cycle 2853, interpreter-only |
| svec_sort / svec_contains / svec_remove / svec_clear | ✅ Cycle 2854, interpreter-only |
| `{fn_call(args)}` 함수 호출 보간 | ✅ Cycle 2855, interpreter-only |
| pow_i64 / clamp_i64 / gcd_i64 | ✅ Cycle 2856, interpreter-only |
| str_count / str_pad_left / str_pad_right | ✅ Cycle 2857, interpreter-only |

---

## 변경 파일 (이번 세션 전체)

**Rust 소스**:
- `bmb/src/interp/eval.rs`: 22종 builtin 구현 + 등록 (Cycles 2851-2857)
- `bmb/src/ast/expr.rs`: InterpMini 확장 (call_args + primary 함수호출) + consume() 제거
- `bmb/src/types/mod.rs`: 22종 타입 서명 추가
- `bmb/tests/integration.rs`: 7개 test 함수 추가 (2375→2382)

**문서**:
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 전 사이클 신규 API 문서화
- `claudedocs/ROADMAP.md`: 최신화 (Cycle 2860까지) + M4-12 추가

---

## 다음 세션 우선순위

### Structural Improvement Proposals (Carry-Forward)
1. **`for x in svec {}`** — `Value::SvecHandle(usize)` 별도 값 타입 필요. 현재 svec/vec 모두 `i64`라 구분 불가.
2. **필드 복합 할당 native 지원** — `set obj.field += e`가 codegen에서도 동작하도록 확장.

### 다음 권장 작업
- **B축 재측정** (언어 갭 충분히 해소됨 — baseline 2026-08-13 stale 전에 재측정 고려)
- **`for x in svec {}`** (`Value::SvecHandle` 도입)
- 또는 새로운 M4 언어 갭 발굴

---

## 기술 인사이트 (다음 세션 참고)

1. **InterpMini 함수 호출 파싱**: `primary()` 내 ident 읽은 후 `(` 체크 → `call_args()` → `Expr::Call`. 재귀적으로 인수도 `expr()` 호출하므로 중첩 함수 호출 자동 지원.

2. **svec vs vec 핸들 구분 불가**: 두 핸들 모두 `i64`. svec 핸들은 SVEC_REGISTRY index, vec 핸들은 heap ptr. `Value::SvecHandle(usize)` 추가 시 해결 가능.

3. **str_hashmap raw ptr + SVEC_REGISTRY 혼합**: str_hashmap은 `Box<HashMap<String, i64>>` raw ptr, svec는 SVEC_REGISTRY index. 두 방식의 lifetime 관리가 다름 — free 순서 중요.

4. **InterpMini::consume() 패턴**: 초기 설계 시 skip+advance를 하나로 묶었으나 `expect()` 만으로 충분. dead_code 경고 시 즉시 제거.
