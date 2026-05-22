# Cycle 3060: M6-P3 gotgan — 골든 테스트 + ROADMAP 업데이트 + 완료 마킹
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3059):
- Cycle 3060 — gotgan.bmb 골든 테스트 작성 + ROADMAP M6-P3 ✅ 마킹

STEP 0 결과: 계획 유효. ⚪ NONE.

## Scope & Implementation

### 골든 테스트 작성

`tests/golden/test_golden_gotgan_bmb.bmb` 신규 작성 — 10개 테스트, 기대 출력 100:

| 테스트 | 내용 |
|--------|------|
| extract name | TOML에서 name 필드 추출 |
| extract version | TOML에서 version 필드 추출 |
| extract missing | 없는 필드 → "" 반환 |
| dep is_path true | `{ path = "..." }` → 1 |
| dep is_path false | `"1.0.0"` → 0 |
| toml template name | 생성된 TOML에 name 포함 |
| toml template version | 생성된 TOML에 version 포함 |
| toml template deps section | 생성된 TOML에 [dependencies] 포함 |
| path_join basic | `foo + bar → foo/bar` |
| path_join trailing slash | `foo/ + bar → foo/bar` |

실행 결과: `bmb run tests/golden/test_golden_gotgan_bmb.bmb` → `100` ✅

**참고**: `tests/golden/`는 수동 실행용. Bootstrap 테스트 시스템은 `tests/bootstrap/golden_tests.txt` 기반 별도 경로 — 통합은 M7 scope로 defer.

### ROADMAP 업데이트

`claudedocs/ROADMAP.md` M6 진행 상태:
```
M6  Full Dogfooding  ██████████████░░░░░░  🔄 — P1 ✅, P2 ✅, P3 MVP ✅ (interp모드)
```

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `test_golden_gotgan_bmb.bmb` 실행 | ✅ 100/100 |
| `cargo test --release` (Cycle 3058) | ✅ 6264/6264 |

## Reflection

- **Scope fit**: 100%
- **테스트 시스템 파악**: `tests/golden/`은 수동 전용, `tests/bootstrap/golden_tests.txt`가 공식 CI 테스트 시스템 — 이전에 작성된 `test_golden_gotgan_*.bmb`들도 미등록 상태로 동일한 패턴
- **M6-P3 완료 기준**: gotgan.bmb MVP (6 commands: new/init/build/check/clean/tree), TOML 파서, 의존성 트리 재귀 — 모두 완료. `bmb run`(인터프리터) 모드로 완전 동작

## Carry-Forward
- Actionable: Cycle 3061 — benchmark-bmb 상태 확인 (git status에서 `m ecosystem/benchmark-bmb` 표시)
- Structural Improvement Proposals:
  - `tests/golden/*.bmb` → `tests/bootstrap/` 이관 + `golden_tests.txt` 등록 (P3, 별도 정리 사이클)
  - `str_lines` / `svec_*` native codegen 지원으로 gotgan.bmb native 빌드 가능화 (M7 scope)
- Pending Human Decisions: 없음
- Roadmap Revisions: M6 진행바 업데이트 (P3 MVP ✅)
- Next Recommendation: Cycle 3061 — git status `m ecosystem/benchmark-bmb` 수정 사항 점검 + 나머지 M6 현황 정리
