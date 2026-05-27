# BMB Session Handoff — 2026-05-27 (Cycles 3206-3209)

> **HEAD**: `(commit pending)`
> **이번 세션 작업**: Cycles 3206-3209 — **M11-A Phase 1-4: trivial postconditions 49개 교체**
> **M11-A 상태**: 🔵 진행 중 (358 → 309 trivials, -13.7%)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **M10 상태**: ✅ **COMPLETE** (이전 세션)
> **Stage 2 상태**: ✅ **RECOVERED** (Cycle 3202)
> **0-Warning 상태**: ✅ **유지** (lint 0 warnings, Cycle 3209까지)
> **M11-B 상태**: ✅ **COMPLETE** (Cycle 3205)

---

## 이번 세션 작업 요약 (Cycles 3206-3209)

### M11-A: trivial postcondition → semantic postcondition 교체

4개 사이클에 걸쳐 49개 trivial postcondition을 의미있는 계약으로 교체.

#### 변경 요약

| 사이클 | 내용 | 변경 수 |
|--------|------|---------|
| 3206 | cf_is_pow2 + cp_is_var_char (bool 2) + 14개 llvm_gen_* (String >= 1) | 16 |
| 3207 | bool scan 함수 7개 (pos < X.len() / pos < end / n > 0 / idx < argc) | 7 |
| 3208 | bool scan/check 함수 6개 (unique sig, semantic_duplication 안전) | 6 |
| 3209 | String LLVM codegen 13개 (항상 non-empty 확인) | 20 |
| **합계** | | **49** |

#### 발견된 제약사항

1. **semantic_duplication**: bool 함수에서 동일 (sig+pre+post) 조합 → lint 경고 발생.
   같은 스캔 패턴을 공유하는 함수군에서 대표 1개만 semantic post 교체 가능.
   **String 함수는 이 제약 없음** (empirical: 14개 동일 패턴 공존 후 0 warnings).

2. **"all" 함수**: 빈 컨테이너에서 `true` 반환하는 함수 (`ipr_all_calls_pure`, `match_bytes` 등)
   → `post not it or pos < X.len()` 성립 안 함 → skip.

3. **조건부 empty String**: `llvm_handle_mark_str_ptr_if` → `same_mapping("")` 항상 반환
   → `>= 0` 유지 필수.

#### skip 확정 목록 (변경 금지)

| 종류 | 수 |
|------|---|
| bool no-pre (skip 확정) | 7 |
| i64 `post it == it` | 7 |
| String no-pre (skip 확정) | 77 |

#### 현재 trivials 현황

| 종류 | 세션 전 | 현재 |
|------|---------|------|
| bool `post it or not it` | 49 | **27** |
| i64 `post it == it` | 7 | 7 |
| String `post it.len() >= 0` | 302 | **275** |
| **합계** | **358** | **309** |

---

## 다음 세션 시작점

### 가능한 다음 단계 (우선순위 순)

| 순위 | 작업 | 설명 |
|------|------|------|
| 1 | **M11-A Phase 5** | String 275개 중 198개 비-skip 추가 분석 (LLVM conc/channel 등) |
| 2 | **언어 갭 추가** | stack array / closure / generic 등 미지원 기능 |

### M11-A Phase 5 세부 계획

미분석 String 함수 중 확인 우선순위:
- `llvm_gen_conc_rhs`, `llvm_gen_conc_stmt`, `llvm_gen_channel_new` — LLVM concurrent ops
- `format_fn_params`, `gen_i32_param_sexts` — formatting (empty 반환 가능성)
- `gen_assumes_for_contracts_acc`, `gen_assumes_for_post_acc` — accumulator (acc="" 가능)

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| chained_comparison | **0** ✅ |
| missing_postcondition | **0** ✅ |
| 총 warnings | **0** ✅ |
| Stage 1 bootstrap | ✅ |
| Stage 2 bootstrap | ✅ |
| bootstrap.sh fixed_point | ✅ **true** |
| BMB-internal Fixed Point | ✅ |
| 테스트 | 3800+ passed ✅ |
| M11-A trivials | 358 → 309 (-49) 🔵 |

---

## 알려진 미결 사항

- **trivial postconditions**: 309개 잔여 (이전 358개 대비 -49). M11-A 계속.
- **ifs_flex_check_goto**: `post it >= 0` Z3 실패 (pre-existing) — `pre next_p >= 0` 누락.
- **BMB IR → opt 최적화 불가**: printf 기반 IR 방출 코드가 opt O1+ 적용 시 절단됨.
- **lint.exe 재빌드 불가**: `bootstrap/lint/lint.exe`의 PHI node IR 오류.
- **Unix 링크 스택 미설정**: `bootstrap.sh` Unix 브랜치 carry-forward.

---

## 핵심 기술 메모

### semantic_duplication 제약 (bool only)

bool 함수에서 동일 (sig+pre+post) 조합 → lint `semantic_duplication` 경고.
String 함수는 동일 조합도 경고 없음 (Cycle 3206 empirical 확인).

### `is_int_literal` 범위 주의 (핵심!)

```bmb
fn is_int_literal(kind: i64) -> bool =
  kind < 2000000000 + 100 or kind >= 2000000000 + 1000;
```

**2000000100–2000000999 범위는 제외!** (토큰 상수 범위)

### bootstrap.sh opt 제거 (핵심!)

Stage 2 바이너리 컴파일: `opt` 사용 금지, `llc -filetype=obj -O2`만 사용.
