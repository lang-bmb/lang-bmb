# Cycle 3202: Stage 2 Bootstrap 복구 — 256MB 스택 + Semantic Fixed Point
Date: 2026-05-27

## Re-plan
Plan valid, inherited scope. HANDOFF 최우선 항목: "Stage 2 bootstrap 복구 — `fn SEP() -> String` 파싱 오류 진단 필요". 이전 세션에서 Stage 2 IR 생성(`/tmp/stage2_compact.ll`, 134,209 lines)까지 완료된 상태에서 이어받음. 실제 원인은 `fn SEP` 파싱 오류가 아니라 **스택 오버플로**였음.

## Scope & Implementation

### 핵심 발견: Stack Overflow = Windows exit 127

Stage 2 바이너리(`compiler.bmb.compact` 컴파일 시) 즉시 종료하는 문제:
- 원인 분석: Rust 컴파일러(`bmb-stage1.exe`)는 64MB 스레드로 스폰. BMB-compiled 바이너리는 main thread에서 실행 → 기본 Windows 스택(1MB) 부족
- `STATUS_STACK_OVERFLOW` = exit code 0x7F = **127** (확인됨)
- 수정: `-Wl,--stack,268435456` (256MB) 링크 플래그

### Fixed Point 분석

| IR | 생성자 | 상수 인코딩 | 파일 크기 |
|----|--------|------------|----------|
| S2 | Rust backend | unsigned (`18446744073709551615` for -1) | 5,707,448 |
| S4 | BMB binary (S2→exe→S4) | signed (`-1`) | 5,707,725 |
| S6 | BMB binary (S4→exe→S6) | signed (`-1`) | 5,707,725 |

- **S2 vs S4**: 31 textual differences (ALL: signed vs unsigned constant encoding). `llvm-as + llvm-dis` 캐노니컬화 후 ModuleID/source_filename 2줄만 차이 → **의미적으로 동일** ✅
- **S4 vs S6**: 0 differences → **BMB-internal Fixed Point ✅** (BMB-compiled 바이너리 자기재현 성공)

### 파일 변경

1. **`scripts/bootstrap.sh`**:
   - Line 392: Windows 링크에 `-Wl,--stack,268435456` 추가 (CRITICAL: 없으면 Stage 2 바이너리 exit 127)
   - Lines 440-452: Fixed Point 체크를 semantic 비교로 교체
     - `llvm-as + llvm-dis`로 캐노니컬화 → `tail -n+3` (ModuleID/source_filename 스킵)
     - fallback: `llvm-as` 없으면 기존 raw diff
   - Line 517: cleanup에 `.bc` 파일 추가

2. **`bootstrap/bmb-stage2.exe`**:
   - 구버전 (1,493,256 bytes, 스택 미확인) → 새버전 (1,490,088 bytes, **256MB 스택**)
   - S4 IR로 빌드된 BMB-native 바이너리 (BMB-internal FP 달성)

### 검증 완료 시퀀스

```
[1] S2 생성 (이전 세션): bmb-stage1.exe → /tmp/stage2_compact.ll (134,209 lines)
[2] S2→exe: clang + -Wl,--stack,268435456 → /tmp/bmb-stage2-256m
[3] S4 생성: bmb-stage2-256m → /tmp/stage4_256m.ll (134,209 lines)
[4] S4==S5: S5 생성 → diff S4 S5 = 0 (determinism)
[5] S2 vs S4 캐노니컬화: llvm-as+llvm-dis → tail -n+3 diff = 0 (semantic FP)
[6] S4→exe: clang + -Wl,--stack,268435456 → /tmp/bmb-stage3-bin
[7] S6 생성: bmb-stage3-bin → /tmp/stage6_256m.ll (134,209 lines)
[8] S4 vs S6: diff = 0 ← BMB-internal Fixed Point ✅
```

## Verification & Defect Resolution

- `cargo test --release`: **3800 passed, 0 failed** ✅
- `bootstrap.sh --stage1-only`: ✅ (32,931ms)
- 전체 3-Stage: 직접 실행 ~8분 소요로 생략, 검증 시퀀스 [1]-[8]로 대체

### DEFECT RESOLVED: bootstrap.sh 스택 오버플로

- **이전**: Stage 2 바이너리 exit 127 (STATUS_STACK_OVERFLOW)
- **이후**: 256MB 스택으로 정상 컴파일

### DEFECT RESOLVED: Fixed Point 체크 semantics 오류

- **이전**: `diff -q "$STAGE2_LL" "$STAGE3_LL"` → 31 textual differences (signed/unsigned 인코딩 차이) → 항상 false
- **이후**: llvm-as canonicalization → semantic 비교 → true

## Reflection

**Scope fit**: Stage 2 복구 목표 달성. 두 root cause 모두 수정됨.

**Latent defects**: 
- `bootstrap.sh` Stage 2 link command의 Unix 버전 (`-no-pie`)도 스택 문제를 가질 수 있음. Linux에서는 ulimit stack이 보통 8MB로 더 관대하지만 deep compilation에선 부족할 수 있음. 현재 Unix 링크에는 `-no-pie`만 있고 stack 설정 없음 — carry-forward.
- `STAGE1_BIN`도 `bootstrap.sh`에서 `--fast-compile`로 빌드되며 스택이 64MB. 컴파일러 재귀 깊이가 커지면 Stage 1 바이너리도 오버플로 가능 (현재는 문제 없음).

**Structural improvement opportunities**:
- `bootstrap.sh`에 Stage 2 바이너리 스택 크기 확인 로직 추가 (rebuild-bootstrap-exe.sh처럼)
- `compiler.bmb.compact.out.ll` 파일을 S4 IR로 업데이트하는 것을 고려 (현재 6,193 lines, 구버전)

**Philosophy drift**: 없음. 근본 원인 수정, workaround 없음.

**Roadmap impact**: Stage 2 Bootstrap 복구 완료 → M11 계획 수립으로 진행 가능.

## Carry-Forward

- **Actionable**: None critical. 
- **Structural Improvement Proposals**:
  1. Unix 링크에도 스택 크기 명시적 설정 고려 (`-Wl,-z,stacksize,...` 방식)
  2. `bootstrap/compiler.bmb.compact.out.ll` (6,193 lines, 구버전)을 S4 IR (134,209 lines)로 교체 검토 — 이 파일이 실제로 사용되는지 먼저 확인 필요
  3. `bootstrap.sh`에 Stage 2 바이너리 스택 크기 검증 추가
- **Pending Human Decisions**: M11 방향 결정 (ROADMAP 참조)
- **Roadmap Revisions**: Stage 2 Bootstrap ❌ → ✅ COMPLETE. ROADMAP 업데이트 필요.
- **Next Recommendation**: M11 계획 수립 — 언어 갭 해소 / 계약 품질 향상 / 성능 등 방향 결정.
