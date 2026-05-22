# Cycle 3059: M6-P3 gotgan — Stage 1 Bootstrap 검증 + Bootstrap 동기화 확인
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3058):
- Cycle 3059 — 통합 테스트 + `bootstrap/compiler.bmb` `getcwd` 동기화 + Stage 1 검증

STEP 0 결과: 계획 유효. ⚪ NONE.

## Scope & Implementation

### Bootstrap 동기화 확인 (`getcwd`)

`bootstrap/compiler.bmb` 내 `getcwd` 등록 상태 점검:

| 위치 | 상태 |
|------|------|
| `get_fn_return_type` (line 6879): `@bmb_getcwd` | ✅ 이미 등록됨 |
| `mangle_fn_name` (line 7040): `@getcwd → @bmb_getcwd` | ✅ 이미 등록됨 |
| LLVM declare (line 13866) | ✅ 이미 등록됨 |

Cycle 3058의 P0 수정 (`bmb/src/types/mod.rs`)은 Rust 인터프리터/타입체커와 bootstrap 간 불일치를 해소. Bootstrap은 이미 정상이었음.

### Stage 1 Bootstrap 검증

```
target/release/bmb build bootstrap/compiler.bmb -o bootstrap/compiler_stage1.exe
→ {"type":"build_success","output":"bootstrap/compiler_stage1.exe"}

bootstrap/compiler_stage1.exe check ecosystem/gotgan-bmb/gotgan.bmb
→ OK: ecosystem/gotgan-bmb/gotgan.bmb
```

Stage 1 bootstrap이 gotgan.bmb를 정상 type-check ✅

### Native 빌드 시도 결과

```
bootstrap/compiler_stage1.exe build ecosystem/gotgan-bmb/gotgan.bmb -o gotgan.exe
→ Error: opt failed. Falling back to direct compilation.
→ Error: linking failed
```

**원인**: gotgan.bmb는 인터프리터 전용 builtins 사용:
- `str_lines` → `SvecHandle` 반환: native codegen 미지원
- `svec_*` 함수군: native codegen 미지원
- `getcwd()`, `str_trim()`, etc: native 미지원 가능성

이는 M6 MVP 범위를 벗어남 — gotgan.bmb는 `bmb run`(인터프리터 모드) 전용 구현. P3 등록.

## Verification & Defect Resolution

| 검증 항목 | 결과 |
|---------|------|
| `bmb check gotgan.bmb` | ✅ success (33 warnings) |
| `bmb run gotgan.bmb help` | ✅ 6개 명령 출력 |
| `bmb run gotgan.bmb new` | ✅ 프로젝트 생성 |
| `bmb run gotgan.bmb tree` (dep-chain fixture) | ✅ 3단계 재귀 |
| `bmb run gotgan.bmb clean` | ✅ target/ 정리 |
| Stage 1 bootstrap check | ✅ OK |
| `cargo test --release` | ✅ 6264/6264 (Cycle 3058에서 검증) |

## Reflection

- **Scope fit**: 100% — Bootstrap 동기화 확인 + Stage 1 검증 완료
- **발견**: `getcwd`는 bootstrap에 이미 있었음 — Cycle 3058에서 `types/mod.rs`에만 추가가 필요했음
- **Native 빌드 한계**: gotgan.bmb는 인터프리터 전용 builtins(`str_lines`, `svec_*`) 사용 → native 빌드 불가. 이는 M6 자체구현의 현재 한계이며 language gap이 아닌 implementation gap
- **Philosophy drift**: 없음 — `bmb run` 인터프리터 실행이 M6 dogfooding의 1차 목표

## Carry-Forward
- Actionable: Cycle 3060 — gotgan.bmb 골든 테스트 작성 + ROADMAP M6-P3 ✅ 마킹
- Structural Improvement Proposals:
  - `str_lines` / `svec_*` native codegen 지원 추가 (P2 — M7 scope)
  - `path_join` 언어 내장 추가 (P3 제안)
- Pending Human Decisions: gotgan.bmb native 빌드 지원 여부 (M7+ 결정)
- Roadmap Revisions: 없음 (M6-P3 범위 내 진행 중)
- Next Recommendation: Cycle 3060 — gotgan.bmb 골든 테스트 + M6-P3 완료 마킹
