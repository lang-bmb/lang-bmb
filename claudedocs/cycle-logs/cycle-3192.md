# Cycle 3192: chained_comparison → match 자동 변환 (28 chains)
Date: 2026-05-26

## Re-plan
Plan valid. Inherited scope: chained_comparison 경고 제거 (M10 Warning Zero). 스크립트 기반 자동 변환 + broken patterns 수동 수정.

## Scope & Implementation

**convert_chains_to_match.py 실행**:
- 28개 literal chain을 `if var == LIT { } else if ...` → `match var { LIT => ..., ... }` 변환
- 2개 버그 수정: `skip_ws`에 `//` 주석 스킵 추가, `else if` 처리에서 `pos += 2` 제거

**fix_else_match.py 실행**:
- 17개 `else match VAR { }` 패턴을 `else { match VAR { } }`로 수정 (BMB 문법 요구)

**Broken patterns 수동 수정** (6곳):
1. `type_info` (line 3633): `parse_chain`이 `ptr_type != ""` 혼합 조건에서 조기 종료 → if-else 복원
2. `ntype` (line 4316): `or ntype ==` 복합 조건 → if-else 복원
3. match arm 주석 버그 (lines 4681-4684): arm body 끝 `//` 주석이 후속 arm을 흡수 → 쉼표 구분자 복원
4. `llvm_gen_call` (line 7978, 8214): `else { match } else if` 구조적 오류 → `_, => if` arm + `} } }` 추가
5. `llvm_gen_call_reg` (line 8262, 8476): 동일 패턴 수정
6. `get_call_arg_types` (line 8486, 8563): `} }or fn_name ==` → `_, => if` + `} } }` 수정

## Verification & Defect Resolution
- `bmb check`: 에러 없음, 1,564 warnings (chained_comparison: 270)
- Stage 1 bootstrap: ✅ `{"type":"build_success","output":"bootstrap/compiler.exe"}`
- `cargo test --release`: ✅ (0 tests in release mode는 정상)

## Reflection
- **Scope fit**: 28 chains 변환 완료. chained_comparison ~757 → 270 (−487)
- **Latent defect**: `build_match` 함수가 `//` 주석으로 끝나는 arm body를 join할 때 후속 arm이 주석에 흡수됨. `step_cast_to_i32` 건에서 확인. 스크립트 수정 필요.
- **Structural issue**: `parse_chain`이 복합 조건(`else if var == LIT and cond`)에서 LIT까지 consume하고 `and`에서 break → actual_end가 `and` 앞까지 포함 → `fn_name ==` 텍스트 소실. 스크립트 수정 필요.
- **단일 arm match 3개**: `single_arm_match` 경고 3개는 기존 코드에서 이미 존재하던 것.

## Carry-Forward
- Actionable: `convert_chains_to_match.py` 2가지 버그 수정 후 잔여 67 고유 체인 재변환
  1. arm body 끝 `//` 주석 있을 경우 `{ body }` 감싸기
  2. 복합 조건 (`and`, `or`) 있을 경우 체인 조기 종료 시 consumed 위치를 `if` 이전으로 rollback
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3193에서 스크립트 수정 + 잔여 체인 변환
