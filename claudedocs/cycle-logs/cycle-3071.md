# Cycle 3071: gotgan PATH 개선 — BMB_PATH env var 지원
Date: 2026-05-23

## Re-plan
이전 Carry-Forward: gotgan build/check PATH 개선 (bmb_exe_path() 확장).
이번 사이클 범위: `BMB_PATH` 환경변수 지원 추가 → PATH 의존성 opt-out 가능화.

## Scope & Implementation

### 변경 파일

**ecosystem/gotgan-bmb/gotgan.bmb** — `bmb_exe_path()` 함수 확장:

```
이전:
fn bmb_exe_path() -> String = {
    let cwd = getcwd();
    let win_path = path_join(cwd, "target/release/bmb.exe");
    if file_exists(win_path) > 0 { win_path }
    else { ... }

이후:
fn bmb_exe_path() -> String = {
    let env_path = getenv("BMB_PATH");
    if env_path.len() > 0 { env_path }   // 환경변수 우선
    else {
        let cwd = getcwd();
        ...  (기존 로직)
    }
```

### 탐색 순서 (변경 후)

1. `BMB_PATH` 환경변수 (신규, 최우선)
2. `cwd/target/release/bmb.exe` (Windows, 기존)
3. `cwd/target/release/bmb` (Linux, 기존)
4. `"bmb"` (PATH fallback, 기존)

### 사용 예시

```
BMB_PATH=/usr/local/bin/bmb gotgan build
```

또는 CI 환경:
```
export BMB_PATH=$(which bmb)
```

## Verification & Defect Resolution

- `cargo test --release`: 3782 + 47 + 22 + 2390 = 전체 PASS ✅
- gotgan 골든 테스트 7종 모두 통과 (test_golden_gotgan_*) ✅
- bootstrap/compiler.bmb 변경 없음 → Fixed Point 유지

## Reflection
- **Scope fit**: 100%
- **Philosophy drift**: 없음
- **User-facing quality**: `BMB_PATH` env var은 단순하고 Unix 관례에 부합하는 확장
- **Roadmap impact**: Known Issue "gotgan build/check: PATH에 bmb binary 필요" 해소

## Carry-Forward
- Actionable: 없음 (Known Issues 전부 해소)
- Structural Improvement Proposals:
  - src/main.bmb도 동일하게 개선 가능 (단, 현재 주 개발 대상 아님)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3072 — 추가 구조 개선 탐색 또는 조기 종료
