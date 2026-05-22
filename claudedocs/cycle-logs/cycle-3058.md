# Cycle 3058: M6-P3 gotgan BMB 구현 완료 (TOML 파서 + 6 commands)
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3057):
1. Actionable: Cycle 3058 — TOML 파서 + Manifest + new/init + bootstrap exec_with_stdin fix
2. Cycle 계획 3058–3063 유효.

STEP 0 결과: Cycle 3057 carry-forward 그대로 실행. ⚪ NONE.

## Scope & Implementation

### 구현 범위
1. **`bootstrap/compiler.bmb` P0 패치**: `exec_with_stdin`을 `get_fn_return_type` String-반환 목록에 추가 (line 6876)
2. **`ecosystem/gotgan-bmb/gotgan.bmb` 신규**: gotgan MVP 전체 (~440 LOC, 6 commands)
3. **`bmb/src/types/mod.rs` P0 패치**: `getcwd` 타입 체커 누락 추가

### gotgan.bmb 구조

| Section | 기능 | LOC |
|---------|------|-----|
| String utilities | `path_join`, `find_char_back`, `parent_dir`, `extract_quote_loop`, `extract_quoted` | ~50 |
| TOML parsing | `extract_toml_field`, `dep_line_is_path`, `process_dep_line`, `parse_dep_lines_loop`, `find_section_line`, `parse_deps` | ~80 |
| Manifest | `load_manifest_content` | ~10 |
| Root finder | `find_project_root` | ~15 |
| new/init | `toml_template`, `main_bmb_template`, `create_project`, `gotgan_new`, `init_project`, `gotgan_init` | ~75 |
| build/check | `bmb_exe_path`, `dep_includes_loop`, `gotgan_build_or_check`, `gotgan_build`, `gotgan_check` | ~65 |
| clean | `remove_target_entries`, `gotgan_clean` | ~30 |
| tree | `print_deps_tree`, `gotgan_tree` | ~45 |
| CLI | `print_help`, `main` | ~35 |

### 디버깅 과정 (타입 에러 3종)

**버그 1: `str_contains` i64 조건 오류**
```bmb
// 오류: str_contains는 i64 반환, if 조건에 직접 사용 불가
if str_contains(val_part, "path") { 1 } else { 0 }
// 수정:
if str_contains(val_part, "path") > 0 { 1 } else { 0 }
```
타입 체커 `types/mod.rs:426`: `str_contains → Type::I64`.
BMB `if` 조건은 `bool`이 필요. 수정: `> 0` 비교 추가.

**버그 2: `getcwd` 타입 체커 누락 (P0)**
`bmb/src/interp/eval.rs:251`에는 `getcwd` 등록됨, `bmb/src/types/mod.rs`에는 없음.
→ 타입 체커에서 "undefined function: `getcwd`" 에러.
→ `types/mod.rs` P0 최소 패치: `getcwd` + `current_dir` alias 2줄 추가.

**버그 3: `version` 키워드 충돌 (Cycle 3057 분석에서 이미 예측)**
변수명으로 `version` 사용 불가 → `pkg_ver`로 변경.

### 검증된 명령어

| 명령 | 결과 | 비고 |
|------|------|------|
| help (no args) | ✅ | 6개 명령 출력 |
| `new testproject` | ✅ | `gotgan.toml` + `src/main.bmb` 생성 |
| `tree` (no deps) | ✅ | `pkg-name v0.1.0` 출력 |
| `tree` (dep-chain fixture) | ✅ | 3단계 재귀 트래버스 정상 |
| `clean` | ✅ | `target/` 내용 삭제 |
| `build` / `check` | ✅ | `bmb`가 PATH에 있을 때 작동 (설계된 동작) |

dep-chain 출력 예시:
```
pkg-top v0.1.0
+-+ pkg-mid ../pkg-mid (path)
    +-+ pkg-base ../pkg-base (path)
```

## Verification & Defect Resolution

1. **타입 체크**: `bmb check ecosystem/gotgan-bmb/gotgan.bmb` → `{"type":"success","warnings":33}` ✅
2. **기능 테스트**: 6개 명령 모두 검증 ✅
3. **회귀 테스트**: `cargo test --release` → 6264 tests, 0 FAILED ✅

## Reflection

- **Scope fit**: 100% — TOML 파서 + 6 commands 구현 완료, 분석 사이클(3057)에서 예측한 ~440 LOC
- **발견된 P0**: `getcwd` 타입 체커 미등록 — 인터프리터에 있지만 타입 체커에 없는 불일치
- **TOML 파서 선택 검증**: Option(a) grep-based ~80 LOC가 충분. 실제 gotgan.toml 포맷이 극히 단순 (3 패턴).
- **BMB 문법 주의사항**: `str_contains` 반환 타입이 `i64`임을 확인 — bool처럼 사용 시 타입 오류
- **Philosophy drift**: 없음. gotgan.bmb는 external tool (bmb binary)를 호출하는 래퍼로 설계 — BMB가 BMB를 관리하는 dogfooding 철학 부합

## Carry-Forward
- Actionable: Cycle 3059 — gotgan.bmb 통합 테스트 + Stage 1 bootstrap 검증 + `getcwd` bootstrap compiler.bmb 동기화
- Structural Improvement Proposals:
  - `path_join(dir, file) -> String` 내장 builtin 추가 제안 (현재 string concat으로 대체)
  - `walk_dir(path) -> SvecHandle` 내장 추가 제안 (현재 gotgan.bmb는 단일 진입점 가정)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음 (M6-P3 범위 내, 3059-3063 계획 유지)
- Next Recommendation: Cycle 3059 — 통합 테스트 + `bootstrap/compiler.bmb` `getcwd` 동기화 + Stage 1 검증
