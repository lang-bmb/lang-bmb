# BMB Session Handoff — 2026-05-15 (Cycles 2871-2876 — interpreter-only → native 포팅 1차)

> **HEAD**: `7eecdf8f` (이번 세션 커밋 예정)
> **이전 HEAD**: `7eecdf8f` (Cycles 2861-2870)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2822, 120790 lines) — 이번 세션 bootstrap 변경 없음
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2877

---

## 이번 세션 작업 요약 (Cycles 2871-2876)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2871 | str free-functions 12종 native 포팅 | str_len/contains/starts_with/ends_with/find/trim/to_upper/to_lower/replace/repeat/to_int + str_to_f64 |
| 2872 | vec 집계/변환 9종 native 포팅 | vec_sum/min/max/sort/contains/index_of/remove/reverse/fill |
| 2873 | str_trim_left/right/reverse + int_to_hex/bin native 포팅 | C 런타임 새 함수 4종 추가 |
| 2874 | str_substr/count/pad_left/pad_right native 포팅 | BMB↔C 시그니처 불일치 래퍼 패턴 |
| 2875 | f64 수학 free functions 11종 native 포팅 | log/log2/log10/exp/round/tan/atan/atan2/min_f64/max_f64/clamp_f64 |
| 2876 | 정수 수학 4종 native 포팅 + P0 버그 수정 | pow_i64/gcd_i64/clamp_i64/popcount + 음수 arg i32 버그 |

### 테스트 변화
2388 (이번 세션 새 integration test 없음 — native 포팅 테스트는 별도 `.bmb` 파일)

---

## native 포팅 현황 (2876 기준)

### ✅ interpreter + native (이번 세션 완료)

| 함수 | native 포팅 Cycle |
|------|-----------------|
| str_len, str_contains, str_starts_with, str_ends_with | 2871 |
| str_find, str_trim, str_to_upper, str_to_lower | 2871 |
| str_replace, str_repeat, str_to_int, str_to_f64 | 2871 |
| vec_sum, vec_min, vec_max, vec_sort | 2872 |
| vec_contains, vec_index_of, vec_remove, vec_reverse, vec_fill | 2872 |
| str_trim_left, str_trim_right, str_reverse | 2873 |
| int_to_hex, int_to_bin | 2873 |
| str_substr, str_count, str_pad_left, str_pad_right | 2874 |
| log, log2, log10, exp, round, tan, atan, atan2 | 2875 |
| min_f64, max_f64, clamp_f64 | 2875 |
| pow_i64, gcd_i64, clamp_i64, popcount | 2876 |

### ⚠️ 아직 interpreter-only

| 기능 | 상태 | 복잡도 |
|------|------|--------|
| str_split (→ svec) | interpreter-only | 高 — svec 핸들 native 표현 필요 |
| str_split_whitespace (→ svec) | interpreter-only | 高 |
| str_lines (→ svec) | interpreter-only | 高 |
| svec_* 전체 | interpreter-only | 高 — SVEC_REGISTRY native 표현 필요 |
| str_hashmap_* 전체 | interpreter-only | 高 |
| format() variadic | interpreter-only | 高 |
| str_char_at (→ String) | interpreter-only | 中 — 타입 불일치 |
| to_string\<T\> | interpreter-only | 中 |
| while let / for-in-svec | interpreter-only | 中 |
| str_is_empty | 미등록 | 低 |

---

## M4 ① 언어 갭 현황 (2876 기준)

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
| vec_sum/max/min/sort | ✅ Cycle 2836, interpreter+native (2872) |
| str_replace + str_repeat | ✅ Cycle 2837, interpreter+native (2871) |
| svec_join + vec_contains + vec_index_of | ✅ Cycle 2838, interpreter+native (2872) |
| for-in-vec | ✅ Cycle 2841, interpreter-only |
| String interpolation `"Hello {name}"` | ✅ Cycle 2842, interpreter-only |
| compound assignment `+=/-=/*=//=/%=` | ✅ Cycle 2844-2845, interpreter+native |
| str_hashmap 6종 (String→i64 HashMap) | ✅ Cycle 2846, interpreter-only |
| `set obj.field += e` 필드 복합 할당 | ✅ Cycle 2847, interpreter-only |
| `{expr}` 복잡 표현식 보간 | ✅ Cycle 2848, interpreter-only |
| str_hashmap_keys/sorted_keys | ✅ Cycle 2849, interpreter-only |
| svec_new/push + str_hashmap_inc | ✅ Cycle 2850, interpreter-only |
| str_hashmap_delete + str_hashmap_update | ✅ Cycle 2851, interpreter-only |
| str_to_upper / str_to_lower / str_char_at | ✅ Cycle 2852, str_to_upper/lower native (2871) |
| vec_remove / vec_reverse / vec_fill | ✅ Cycle 2853, interpreter+native (2872) |
| svec_sort / svec_contains / svec_remove / svec_clear | ✅ Cycle 2854, interpreter-only |
| `{fn_call(args)}` 함수 호출 보간 | ✅ Cycle 2855, interpreter-only |
| pow_i64 / clamp_i64 / gcd_i64 | ✅ Cycle 2856, interpreter+native (2876) |
| str_count / str_pad_left / str_pad_right | ✅ Cycle 2857, interpreter+native (2874) |
| Value::SvecHandle + for-in-svec | ✅ Cycle 2861-2862, interpreter-only |
| str_to_f64 / read_f64 / str_lines | ✅ Cycle 2863, str_to_f64+read_f64 native (2875) |
| f64 수학 free function 8종 (log~atan2) | ✅ Cycle 2865, interpreter+native (2875) |
| min_f64 / max_f64 / clamp_f64 | ✅ Cycle 2866, interpreter+native (2875) |
| str_trim_left / str_trim_right | ✅ Cycle 2866, interpreter+native (2873) |
| str_split_whitespace | ✅ Cycle 2867, interpreter-only |
| int_to_hex / int_to_bin | ✅ Cycle 2867, interpreter+native (2873) |
| str_reverse / popcount / svec_index_of | ✅ Cycle 2868, str_reverse+popcount native (2873/2876) |

---

## 변경 파일 (이번 세션 전체)

**Rust 소스**:
- `bmb/src/codegen/llvm_text.rs`: 35종+ builtin native 포팅 (IR선언 + name mapping + return type + param type)
- `bmb/runtime/bmb_runtime.c`: C 래퍼 함수 추가 (trim_left/right, int_to_hex/bin, substr, pad_left/right, pow_i64, gcd_i64, clamp_i64, read_f64)

**테스트 파일**:
- `tests/native_str_builtins.bmb`: str 12종 native 테스트
- `tests/native_vec_builtins.bmb`: vec 9종 native 테스트
- `tests/native_str2_builtins.bmb`: str_trim/reverse/hex/bin 테스트
- `tests/native_str3_builtins.bmb`: substr/count/pad 테스트
- `tests/native_f64_builtins.bmb`: f64 수학 11종 테스트
- `tests/native_int_math_builtins.bmb`: 정수 수학 4종 테스트

**사이클 로그**:
- `claudedocs/cycle-logs/cycle-2871.md` ~ `cycle-2876.md`

---

## 다음 세션 우선순위

### Carry-Forward (Actionable)
없음.

### Structural Improvement Proposals
1. **음수 리터럴 i32 narrow 근본 수정** — 음수 arg가 alloca i32로 생성되는 codegen 버그의 root cause 파악 및 fix (현재 runtime_param_type 등록으로 우회). 관련: `build_place_type_map`에서 place_type이 i32로 설정되는 경로.
2. **필드 복합 할당 native 지원** — `set obj.field += e`가 llvm_text.rs에서도 동작하도록. 현재 interpreter-only.
3. **bmb_reference interpreter-only 경고 해제** — 이번 세션에서 많은 함수가 native 포팅됨 → bmb_reference.md의 "(interpreter-only)" 경고 제거 필요.

### Pending Human Decisions
- **B축 재측정**: `BMB_BENCH_API_KEY` 환경변수 필요. 언어 갭 + native 포팅 진전 — baseline 2026-08-13 stale 기한 이전에 재측정 권장.
- **tier3-spawn-overhead**: ISSUE-20260512 Option A/B/C 선택 (HUMAN 결정 필요).

### 다음 권장 작업 (우선순위 순)
1. **B축 재측정** — native 포팅 35종 완료로 `bmb build`에서도 실행 가능한 프로그램 범위 대폭 확대.
2. **bmb_reference 업데이트** — interpreter-only 경고 해제 + native 포팅 표시.
3. **str_is_empty native 포팅** — 미등록 함수, types/mod.rs + llvm_text.rs 소규모 추가.
4. **음수 리터럴 i32 narrow 근본 수정** — `build_place_type_map` 버그 추적.

---

## 기술 인사이트 (다음 세션 참고)

1. **native 포팅 3-레이어 패턴**: IR declare → name mapping → infer_call_return_type. 세 레이어 모두 필요.

2. **runtime_param_type 등록 필수**: 음수 리터럴 등 i32로 narrow된 temp가 함수 인수로 전달될 때 sext가 필요. runtime_param_type에 등록 안 하면 i32 arg → i64 param 타입 불일치 버그 발생.

3. **C 래퍼 패턴**: BMB API와 C 런타임 시그니처 불일치 시 얇은 C 래퍼 추가:
   - `str_substr(s, start, len)` vs `bmb_string_slice(s, start, end)` → `bmb_string_substr(s, start, start+len)`
   - `str_pad_left(s, w, pad: String)` vs `bmb_string_pad_left(s, w, ch: i64)` → `bmb_str_pad_left(s, w, pad_str)` (첫 바이트 추출)

4. **void 반환 함수 주의**: `vec_sort/reverse/fill` 같이 void 반환하는 함수는 `infer_call_return_type`에 명시적 "void" 등록 필요. 미등록 시 undefined behavior.

5. **clamp_f64 인라인**: LLVM `llvm.clamp.f64` intrinsic 없음 → `max(min(x, hi), lo)` 인라인 구현 (llvm.minnum.f64 + llvm.maxnum.f64).

6. **str_to_f64/read_f64 double 반환**: f64 반환 함수는 `infer_call_return_type`에 "double" 등록 필요. 미등록 시 "ptr"로 오판.
