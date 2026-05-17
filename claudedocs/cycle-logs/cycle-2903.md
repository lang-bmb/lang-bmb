# Cycle 2903: bmb_runtime.c CI 자동 재빌드 스크립트
Date: 2026-05-17

## Re-plan
Carry-Forward 없음. HANDOFF "Structural Improvement: `scripts/rebuild-runtime.sh` — bmb_runtime.c 변경 시 자동 재빌드" 항목 선택.

## Scope & Implementation

**목표**: `bmb/runtime/bmb_runtime.c` 또는 `bmb_event_loop.c` 변경 시 `libbmb_runtime.a` 자동 재빌드 스크립트 작성.

**파일 생성**: `scripts/rebuild-runtime.sh`

### 스크립트 설계

**템플릿**: `scripts/rebuild-bootstrap-exe.sh` 패턴 적용 (`--force`, `--json`, `--check-only`/`--ci`).

**staleness 검사**: `-nt` 파일 타임스탬프 비교 — 두 소스 파일 중 하나라도 `.a`보다 새로우면 stale.

**컴파일러 선택**: clang 우선, 없으면 gcc 폴백 (`pick_cc()`).

**컴파일 플래그**: `bootstrap_3stage.sh`와 동일 플래그 사용:
```
-O2 -ffunction-sections -fdata-sections
```

**이중 출력**: 기본 `bmb/runtime/libbmb_runtime.a` 재빌드 후 `runtime/libbmb_runtime.a` 동기화 (디렉토리 존재 시).

**사용법**:
```bash
bash scripts/rebuild-runtime.sh             # 타임스탬프 기반 자동 재빌드
bash scripts/rebuild-runtime.sh --force     # 강제 재빌드
bash scripts/rebuild-runtime.sh --json      # 머신 출력
bash scripts/rebuild-runtime.sh --ci        # exit 1 on stale (CI 게이트)
```

### 부수 발견 사항

실행 중 git 트래킹 바이너리(124530 bytes, ~540 symbols)와 소스 재빌드 결과(224310 bytes, ~1695 symbols)의 심볼 수 격차 발견.

**원인**: Cycles 2871-2876에서 추가된 35종+ native builtin이 `bmb_runtime.c`에 추가됐으나, git 커밋 바이너리는 업데이트되지 않음. 스크립트가 이 stale 상태를 첫 실행에서 자동 감지·수정.

## Verification & Defect Resolution

**스크립트 테스트**:
```
# current 상태:
bmb/runtime/libbmb_runtime.a is current (224310 bytes)

# --force 강제 재빌드:
libbmb_runtime.a is stale — rebuilding with clang ...
  → synced to runtime/libbmb_runtime.a
OK: libbmb_runtime.a rebuilt in 13507 ms (224310 bytes, compiler: clang)

# --json:
{"status":"current","rebuilt":false,"lib":"...","bytes":224310}

# --ci exit 0 확인
```

`cargo test --release`: **2388/2388 PASS** ✓ (재빌드된 runtime으로 정상 동작)

## Reflection
- **Scope fit**: `rebuild-bootstrap-exe.sh`와 일관된 패턴 완성.
- **부수 효과**: git stale 바이너리 → 소스 기준 재빌드로 자동 동기화. 이번 실행으로 `bmb/runtime/libbmb_runtime.a`와 `runtime/libbmb_runtime.a` 양쪽 최신화.
- **Limitation**: git에 커밋된 `.a` 바이너리 자체가 stale — 다음 커밋에서 갱신 필요.
- **Philosophy drift**: 없음.
- **Roadmap impact**: Structural Improvement 완료. `rebuild-runtime.sh --ci`를 `quick-check.sh` / `full-cycle.sh`에 통합하면 Rule 7 위반 + runtime stale 두 문제를 CI에서 동시 방지 가능.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - `quick-check.sh`/`full-cycle.sh`에 `rebuild-runtime.sh --ci` 통합
  - `check_backend_parity.py --ci` 통합
  - git 커밋된 `libbmb_runtime.a` 갱신 (소스 기준 최신 바이너리)
- Pending Human Decisions:
  - B축 재측정 실행
  - Java 바인딩 계속 개발 여부
- Roadmap Revisions: None
- Next Recommendation: Cycle 2904 — Java scaffold batch (bmb-json/compute/crypto/text 나머지 4개)
