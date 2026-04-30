# Cycle 2505: -lm 누락 결함 + CI fixed-point 마스킹 fix
Date: 2026-04-30

## Re-plan
Plan invalid. HANDOFF은 "CI Fixed Point preserved" 주장했으나, CI artifact
재검증에서 정반대 사실 확인:

1. `bmb_runtime.c`의 `bmb_f64_floor/ceil/round/sqrt` + `pow_f64`가
   `floor/ceil/round/sqrt/pow`(libm)을 참조.
2. `bmb/src/build/mod.rs`의 두 link path(`link_native`, `link_with_runtime`)가
   Linux에서 `-lm`을 빠뜨림. Windows MinGW은 자동 포함, macOS는 libSystem
   에 포함되므로 무사.
3. CI Bootstrap이 매 push마다 Stage 1 link error로 실패해 왔음
   (`undefined reference to floor` 등). Cycle 2498 artifact에서 직접 확인.
4. `.github/workflows/bootstrap-benchmark.yml` 144줄 `|| true`가
   exit 1을 삼켜 워크플로 status를 success로 위장. 후속 step의
   `::warning::Bootstrap fixed point not reached` annotation만 남음.
5. HANDOFF은 워크플로 status(success)를 "Fixed Point preserved"로 잘못 해석
   → 1년+ 동안 fake green CI. 다른 회귀까지 가릴 위험 존재.

회귀가 아니라 **장기 잠복 결함**. 즉시 fix 필요.

## Scope & Implementation

### Fix 1 — `bmb/src/build/mod.rs` link 양 경로

**`link_native` (LLVM inkwell backend)**, line ~1362:
```rust
#[cfg(target_os = "linux")]
{
    if !is_shared { cmd.arg("-no-pie"); }
    cmd.arg("-lc");
    cmd.arg("-lpthread");
    cmd.arg("-lm");                       // Cycle 2505 추가
}
```

**`link_with_runtime` (text backend default)**, line ~1115:
```rust
#[cfg(target_os = "windows")]
cmd.arg("-lws2_32");

#[cfg(target_os = "linux")]              // Cycle 2505 추가
cmd.arg("-lm");
```

macOS `libSystem`은 math 함수 자체 포함 → 추가 불필요.

### Fix 2 — `.github/workflows/bootstrap-benchmark.yml`

`|| true` 제거하고 exit code를 명시적으로 보존하여 실패를 진짜 실패로 surface:

```yaml
set +e
./scripts/bootstrap.sh --json > bootstrap_results.json 2>&1
BOOTSTRAP_EXIT=$?
set -e
# ... parse JSON for telemetry ...
if [ "$BOOTSTRAP_EXIT" -ne 0 ]; then
  echo "::error::3-Stage Bootstrap failed (exit $BOOTSTRAP_EXIT)"
  exit "$BOOTSTRAP_EXIT"
fi
```

`Verify Bootstrap Success` step의 `::warning::`도 `::error::` + `exit 1`
로 강화 (defensive — JSON과 exit code 불일치 대비).

### Decision Framework

| 수준 | 판단 |
|------|------|
| 1. 언어 스펙 | 무관. |
| 2. 컴파일러 구조 | ✅ 정직한 link command — 의존성 누락은 결함. |
| 3. 최적화 패스 | 무관. |
| 4. 코드젠 | 코드젠 자체는 정확. link만 결함. |
| 5. 런타임 | 런타임은 OK. 소비자(컴파일러 link)가 누락. |

Level 2 fix. workaround 회피(예: bmb_runtime.c에서 math.h 호출 제거 등)
가 아니라 정확한 의존성 명시 — Performance > Everything의 **Honest
Cost** 원칙.

### Rule 6 (Rust frozen) 예외 정당화

CLAUDE.md Rule 6 기준 "Rust 버그 수정: ⚠️ 부트스트래핑 차단 시에만".
이 fix가 정확히 그 케이스 — Linux Bootstrap이 영영 실패 중. 정당한 예외.

## Verification

### 로컬 (Windows MinGW UCRT, HEAD pre-commit)

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 5m 09s |
| `cargo clippy --all-targets -- -D warnings` | ✅ clean |
| `cargo test --release --lib` | ✅ 3,772 pass / 0 fail |
| `bash scripts/bootstrap.sh --stage1-only` | ✅ Stage 1 OK (21.9s) |
| `bash scripts/bootstrap.sh` (full 3-Stage) | ✅ Fixed Point S2 == S3 (119s) |

Windows에서는 -lm `cfg(target_os = "linux")` gate되어 영향 없음 — 회귀 0.

### CI (post-commit)

push로 트리거되는 Bootstrap + Benchmark Cycle이 이번엔 **진짜로** 실패하거나
성공하게 됨. Linux 부트스트랩이 -lm으로 link 통과해야 Stage 2/3 IR 생성
가능. Fixed Point 검증을 처음으로 empirical 수행.

## Defect Resolution

| 결함 | 심각도 | 해결 |
|------|--------|------|
| Linux `-lm` 누락 → bootstrap.sh 영구 실패 | High | ✅ link_native + link_with_runtime fix |
| `\|\| true` masking | High | ✅ workflow 정직화 (set +e + exit code 보존) |
| HANDOFF "Fixed Point preserved via CI" 오류 | Med | 본 cycle log에 정정 기록 |

## Reflection

### Scope fit
✅ Cycle 1개로 fix + verification 완결. 로컬 회귀 0, CI에서 empirical
Linux 검증 가능 상태로 push.

### Latent defects discovered
- **HANDOFF/cycle log 신뢰도 회의**: HANDOFF는 "Fixed Point preserved"
  단언했지만 artifact 직접 확인 결과 fake. workflow status(success)만
  보고 단언했음 → HANDOFF 작성 시 artifact JSON `fixed_point` 필드를
  반드시 직접 검증하는 절차 필요. (다음 사이클의 HANDOFF에서 적용)

### Philosophy alignment
- **Performance > Everything**: 무관 (build infra fix).
- **No Workaround**: ✅ Level 2 정직 fix. bmb_runtime.c 우회 등 회피 안 함.
- **Honest Measurement**: ✅ `|| true` 제거가 핵심. CI status가 실제 검증
  결과를 반영하지 않으면 모든 회귀 검사가 무의미.

### Roadmap impact
- **B'.1**: Cycle 2492 + 2500 + 2505 조합으로 **진짜로** 검증 가능. 이전
  CI green은 Linux에서는 fake였음 (windows-latest만 진짜 검증). post-fix
  CI run에서 ubuntu-latest Bootstrap이 처음으로 정상 통과 예상.
- **G.1 (Z3)**: 환경 셋업 필요 — winget으로 Z3 설치 시도 가능.
- **D' (Golden 정책)**: 자율 권장안 작성 가능.

## Carry-Forward

### Actionable (이번 세션 진행 중)
- Push 후 CI 모니터링: Bootstrap + Benchmark Cycle ubuntu-latest가
  진짜로 통과해야 함.
- G.1 Z3 설치 시도 (winget).
- D' Golden 정책 권장안.

### Pending Human Decisions
- TestPyPI token 등록 (B'.2 — org admin만 가능).
- C' WSL2 환경 (Class not registered — admin install 필요).

### Roadmap Revisions
- ROADMAP B'.1 entry 정정 필요 — "linux 검증 미수행" 명시 후 Cycle 2505
  이후로 갱신.
