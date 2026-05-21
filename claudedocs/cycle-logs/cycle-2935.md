# Cycle 2935: Fixed Point 검증 + bootstrap_3stage.sh 인프라 수정
Date: 2026-05-19

## Re-plan
Cycle 2934 Carry-Forward:
- `is_fnref` 로버스트니스 수정 (HOF/i64 파라미터를 함수명으로 오인식하는 버그)
- Stage 2/3 Fixed Point 검증 (Rule 3 의무)
- 클로저 HOF 지원 탐색

## Scope & Implementation

### 1. `is_fnref` 로버스트니스 수정 (`bootstrap/compiler.bmb` ~line 14741)

HOF/i64 파라미터가 글로벌 함수와 같은 이름을 가질 경우 오인식 방지:
```bmb
// Before:
let is_fnref = src.len() > 1 and src.byte_at(0) == 37
    and find_pattern_at(registry, "@" + src.slice(1, src.len()) + ":", 0) >= 0;

// After:
let is_fnref = src.len() > 1 and src.byte_at(0) == 37
    and not (is_hof_param_sb(src, str_sb) or is_i64_param_sb(src, str_sb))
    and find_pattern_at(registry, "@" + src.slice(1, src.len()) + ":", 0) >= 0;
```

### 2. `bootstrap_3stage.sh` 인프라 수정 (3개 수정)

**수정 1 — Arena 크기 설정 누락**:
- Stage 2 IR 생성 시 `BMB_ARENA_MAX_SIZE` 미설정 → 4G OOM 크래시
- 수정: `BMB_ARENA_MAX_SIZE=${BMB_ARENA_MAX_SIZE:-32G}` 추가

**수정 2 — Windows 소켓 라이브러리 누락**:
- Stage 2 바이너리 링크 시 `-lws2_32` 미포함 → 링크 에러
- 수정: `-lws2_32 -no-pie 2>/dev/null ||` fallback 체인 추가

**수정 3 — 스택 크기 미지정**:
- Stage 2 바이너리의 기본 스택(1MB) < compiler.bmb 자기 컴파일 요구사항
- 수정: `-Wl,--stack,67108864` (64MB) 추가 — Rust bmb 링커와 동일

### 3. Stage 2/3 Fixed Point 검증 결과

```
Stage 1 (Rust BMB → Stage 1 Binary):    PASSED
Stage 2 (Stage 1 → LLVM IR):            PASSED (121393 lines)
Stage 3 (Stage 2 → LLVM IR):            PASSED (121393 lines)
Verification (S2 IR == S3 IR):          ✅ PASSED
```

### 4. 클로저 HOF 탐색

**발견**: 인터프리터는 이미 클로저 HOF를 지원함:
```bmb
fn apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x);
let mul = fn |x: i64| { x * 2 };
apply(mul, 21)  // → 42 ✅ 인터프리터
```

**Bootstrap codegen 한계**: 클로저와 named fn의 표현 불일치:
- Named fn: `ptrtoint ptr @fn to i64` (직접 fn ptr)
- Closure: `ptrtoint ptr {fn_ptr, env} to i64` (closure struct ptr)
- HOF call (`llvm_gen_hof_call`): `inttoptr → call` 직접 호출 → closure struct를 fn ptr로 오인 → 크래시

**근본 해결 방향** (별도 사이클 필요):
- Unified representation: `fn(T)->R` 값 = 항상 2-word struct `{fn_ptr, env}`
- Named fn도 `{ptrtoint @fn, 0}` struct로 래핑
- HOF call: 항상 closure protocol 사용 (load fn_ptr, load env, call fn_ptr(env, args))
- Named fn 시그니처: closure env 파라미터 수용 필요 (`@fn(i64 %_env, ...)`)

## Verification & Defect Resolution

### 검증 결과
```
HOF golden tests (test_hof_apply, test_hof_multi): ✅ 통과
골든 테스트 Quick Check: passed ✅
Stage 2/3 Fixed Point: ✅ S2 IR == S3 IR
```

### bootstrap_3stage.sh 수정 후 검증
- Stage 1→2: 121393 lines IR ✅
- Stage 2 binary 빌드: 64MB 스택 + ws2_32 ✅
- Stage 3 IR 생성: 121393 lines ✅
- S2 == S3: ✅ PASSED

## Reflection

### Scope fit
- ✅ Fixed Point 검증 완료 (Rule 3 의무 이행)
- ✅ bootstrap_3stage.sh 3가지 인프라 버그 수정
- ✅ `is_fnref` 로버스트니스 수정
- ⚠️ 클로저 HOF → 인터프리터 작동 확인, bootstrap codegen은 구조적 재설계 필요

### 인프라 의의
`bootstrap_3stage.sh`는 이전에 제대로 동작하지 않았던 스크립트. 이번 수정으로:
1. Windows 환경에서 완전히 동작하는 3-Stage 검증 가능
2. 앞으로 bootstrap 변경 시 Rule 3 이행이 간편해짐

### 클로저 HOF 설계 통찰
현재 HOF 설계는 `fn ptr as i64` 모델을 사용하므로 클로저 (struct ptr)와 호환되지 않음.
진정한 클로저 HOF 지원을 위해서는 `fn(T)->R` 값의 unified representation이 필요.
이는 Principle 3(복잡도는 기피 사유가 아님)에 따라 별도 사이클에서 proper fix 필요.

## Carry-Forward

- Actionable: 클로저 HOF 통합 표현 설계 (별도 사이클, 복잡도 높음)
- Structural Improvement Proposals:
  1. **Closure HOF unified representation**: `fn(T)->R` = 항상 `{fn_ptr, env}` 2-word struct. Named fn도 래핑. Cycle 2936+ 후보.
  2. **bootstrap_3stage.sh 개선**: Stage 2 바이너리 자체 테스트 (`./bmb-stage2 run simple.bmb`) 추가 권장
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2936 — 언어 갭 또는 B축 개선 (string match / more builtins)
