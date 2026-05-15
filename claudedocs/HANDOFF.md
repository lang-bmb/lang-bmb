# BMB Session Handoff — 2026-05-15 (Cycles 2861-2870 — 언어 갭 해소 4차)

> **HEAD**: `(commit 후 갱신 예정)`
> **이전 HEAD**: `21e91395` (Cycles 2851-2860)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2822, 120790 lines) — 이번 세션 bootstrap 변경 없음
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2871

---

## 이번 세션 작업 요약 (Cycles 2861-2870)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2861 | SvecHandle 타입 + for-in-svec | Value::SvecHandle(usize) + SVEC_REGISTRY 반복 |
| 2862 | 타입 체커 for-in svec String 추론 | Type::Named("SvecHandle") + for-in 타입 서명 13종 |
| 2863 | str_to_f64 / read_f64 / str_lines | float 파싱 + 라인 분할 builtins |
| 2864 | bmb_reference 정비 (stale 수정) | {fn_call()} 정정 + v0.98.7 API 문서화 |
| 2865 | f64 수학 free function 8종 | log/log2/log10/exp/round/tan/atan/atan2 |
| 2866 | min_f64/max_f64/clamp_f64 + str_trim_left/right | f64 min/max/clamp + 방향별 trim |
| 2867 | str_split_whitespace + int_to_hex/bin | 공백 분리 + 진수 변환 |
| 2868 | str_reverse + popcount + svec_index_of | 회문 체크 + 비트 count + svec 위치 검색 |
| 2869 | bmb_reference 비트 연산 + 알고리즘 패턴 | band/bor/bxor 문서화 + Palindrome/Tokenized 패턴 |
| 2870 | HANDOFF/ROADMAP 정리 + 커밋 | 이번 문서 |

### 테스트 변화
2382 → **2388** (+6 integration tests)

---

## M4 ① 언어 갭 현황 (2870 기준)

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
| Value::SvecHandle + for-in-svec | ✅ Cycle 2861-2862, interpreter-only |
| str_to_f64 / read_f64 / str_lines | ✅ Cycle 2863, interpreter-only |
| f64 수학 free function 8종 (log~atan2) | ✅ Cycle 2865, interpreter-only |
| min_f64 / max_f64 / clamp_f64 | ✅ Cycle 2866, interpreter-only |
| str_trim_left / str_trim_right | ✅ Cycle 2866, interpreter-only |
| str_split_whitespace | ✅ Cycle 2867, interpreter-only |
| int_to_hex / int_to_bin | ✅ Cycle 2867, interpreter-only |
| str_reverse / popcount / svec_index_of | ✅ Cycle 2868, interpreter-only |

---

## 변경 파일 (이번 세션 전체)

**Rust 소스**:
- `bmb/src/interp/eval.rs`: 27종 builtin 구현 + 등록 (Cycles 2861-2868) + Value::SvecHandle 활용
- `bmb/src/interp/value.rs`: Value::SvecHandle(usize) 추가 (Cycle 2861)
- `bmb/src/types/mod.rs`: 27종 타입 서명 추가 (Cycles 2862-2868)
- `bmb/tests/integration.rs`: 6개 test 함수 추가 (2382→2388)

**문서**:
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 전체 v0.98.7 API 문서화 + stale 수정 + 패턴 2종 추가
- `claudedocs/ROADMAP.md`: 최신화

---

## 다음 세션 우선순위

### Carry-Forward (Actionable)
없음 — 모든 계획된 작업 완료.

### Structural Improvement Proposals
1. **필드 복합 할당 native 지원** — `set obj.field += e`가 codegen(llvm_text.rs)에서도 동작하도록 확장. 현재 interpreter-only.
2. **bmb_reference 예시 통합 테스트** — 패턴 코드 예시를 실제 integration test로 연결하는 체계 구축 (stale 방지)

### Pending Human Decisions
- **B축 재측정**: `BMB_BENCH_API_KEY` 환경변수 필요. 언어 갭 충분히 해소됨 — baseline 2026-08-13 stale 기한 이전에 재측정 권장.
- **tier3-spawn-overhead**: ISSUE-20260512 Option A/B/C 선택 (HUMAN 결정 필요).

### 다음 권장 작업 (우선순위 순)
1. **B축 재측정** — 이번 4차 세션으로 언어 갭 ~35종 해소. "AI가 BMB로 자연스럽게 코드 작성 가능"을 주장할 수 있는 수준.
2. **새로운 M4 언어 갭 발굴** — B축 측정에서 실패하는 패턴 분석 후 추가 갭 식별.
3. **interpreter-only → native 포팅** — 주요 str/svec/hashmap builtins의 LLVM 코드젠 지원.

---

## 기술 인사이트 (다음 세션 참고)

1. **Value::SvecHandle(usize) 도입 이유**: 기존 svec/vec 핸들이 모두 `Value::Int(i64)`였기 때문에 for-in dispatch 불가능. `Value::SvecHandle(usize)`로 구분 → SVEC_REGISTRY 반복 가능.

2. **svec_t scope 제약**: `types/mod.rs`에서 `svec_t`는 L425에서 정의됨. svec_t를 인자로 사용하는 타입 서명은 반드시 L425 이후에 배치해야 함 (Cycle 2868에서 발견).

3. **f64 수학 macro 패턴**: `f64_unary_builtin!(fn_name, "fn_name", method_name)` 매크로로 단항 f64 builtin 7종을 간결하게 구현. 향후 추가 f64 함수에도 재사용 가능.

4. **str_split vs str_split_whitespace**: `str_split(s, " ")`은 연속 공백에 빈 토큰 생성. `str_split_whitespace(s)`는 Rust의 `split_whitespace()`로 빈 토큰 자동 제거 — 경쟁 프로그래밍 입력 파싱에 권장.

5. **bmb_reference stale 관리**: 이번 세션에서 {fn_call()} 지원이 3 사이클(2855→2858→2864) 동안 stale 상태로 있었음. 기능 추가 사이클에서 bmb_reference도 즉시 업데이트하는 규칙 필요.
