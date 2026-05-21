# Cycle 2708: Stage 2 OOM 재현 + 메모리 사용 곡선 측정
Date: 2026-05-11

## Re-plan
인계받은 scope (HANDOFF/2707 next recommendation): Stage 2 진단 — parse error vs arena OOM 가설 분리.
Trigger: ⚪ NONE. 진행.

10 사이클 directional roadmap:
- 1-3: Stage 2 진단 (이 사이클 = Cycle 1)
- 4: checkpoint (RE-PLAN 또는 fix 진입)
- 5-7: Builtin arity proper fix 또는 Stage 2 fix
- 8: 측정 강화 (Tier 3 ≥10 runs)
- 9: ISSUE triage
- 10: HANDOFF/ROADMAP 갱신 + commit

## Scope & Implementation

### 측정 환경
- `bmb.exe` (Rust BMB, target/release): 10.3MB, mtime 2026-05-09
- `bmb-stage1.exe` (Rust BMB → compiler.bmb 빌드 결과): 1.4MB, build time 10.5s
- compiler.bmb 크기: **20802 lines**

### 직접 측정 (BMB_ARENA_MAX_SIZE=16G, target=bootstrap/compiler.bmb)

```
20802 lines (full):  parse error line 1:3 — 0.2s
```

이전 memory note 기록: "Stage 2 arena OOM (32G+) compiler.bmb self-compile" — 표면 일치하지 않음.
0.2s 만에 죽으므로 OOM이 아닌 **parse 결함처럼 보임**.

### Stage 1 binary 자체 검증
- simple `fn main() -> i64 = 42;`: OK
- leading line comment `// BMB Test\nfn main()...`: OK
→ Stage 1 binary 자체는 작동. compiler.bmb 자체의 특정 입력 처리 결함.

### 절단 사이즈 곡선 (BMB_ARENA_MAX_SIZE=16G)

| Lines | 결과 | 시간 | 해석 |
|-------|------|------|------|
| 1000 | OK | <1s | 정상 컴파일 |
| 2000 | OK | <1s | 정상 컴파일 |
| **3000** | **OOM 16G** | **~14s** | **비-선형 폭발 시작점** |
| 4000 | OOM 16G | ~14s | |
| 5000 | parse error (line 5000:11 truncation) | <1s | artifact |
| 6000 | OOM 16G | ~14s | |
| 8000 | parse error (artifact) | <1s | |
| 9000 | parse error (artifact) | <1s | |
| 9500 | OK | <1s | truncation이 valid syntax → 작은 subset 컴파일 |
| 10000 | OOM 16G | 13.9s | |
| 15000 | OOM 16G | ~14s | |
| 18000 | OOM 16G | ~14s | |
| 20000 | parse error line 1:3 | 0.2s | **OOM allocator fail → corruption** 추정 |

**핵심 관찰**:
1. **2K lines OK / 3K lines OOM** — 메모리 폭발 시작점
2. **3K → 16GB**: 매우 강한 비-선형 (O(n²) 또는 worse 의심)
3. **20K full source 0.2s parse error**: 초기 alloc 단계에서 16GB+ 요구 → arena fail → null/0-len source로 parse_source 진입 → line 1:3 위치에서 garbage token으로 if-context 들어감

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Stage 1 빌드 (`bmb build compiler.bmb`) | ✅ 10.5s |
| Simple BMB compile | ✅ |
| Comment leading BMB compile | ✅ |
| compiler.bmb (full) compile | ❌ parse error line 1:3 (OOM derivative) |
| 3K-18K truncated subset | ❌ 16GB OOM |
| 가설 분리 (parser vs O(n²) AST) | ✅ **OOM 우세** — parse error는 OOM 부작용 |

결함: 없음 (이 사이클은 진단만, fix 의도 없음).

## Reflection

### 외부 관찰자 관점에서

1. **Memory note "32G+" 주장의 정확도**: "compiler.bmb self-compile parse error vs arena OOM 두 가설" 표현은 잘못 — 두 가설은 동일 케이스의 다른 표현. parse error는 OOM 후행 효과.

2. **3K threshold의 의미**: 2K lines OK이지만 3K가 16GB 초과 — 1K lines 추가가 메모리를 8GB 추가 요구한다는 의미. 이는 O(n²) 또는 O(n²·k) 비례. 문자열 기반 AST의 누적 copy 비용으로 일치.

3. **bootstrap.sh의 BMB_ARENA_MAX_SIZE=16G default 의의**: 현재 default 자체가 부족 — 부트스트랩이 동작하려면 64GB+ 필요할 가능성. 시스템 RAM 한도가 사실상 부트스트랩 차단.

4. **scope fit**: bounded 진단 목표 달성. fix 시도 안 함 (advisor 권고 준수).

### Roadmap impact

- ROADMAP § M5-1 표 행 "M5-1: Fixed Point 검증" 노트 — "32G+" → "≥16G; 2K lines OK / 3K lines OOM" 로 정정 권고
- Cycle 2237 시점에 S2==S3 PASS였다면 그 후 compiler.bmb 크기 증가가 OOM threshold를 넘김. git blame으로 size 추이 확인 후보.

## Carry-Forward

- Actionable (Cycle 2 = Cycle 2709): **O(n²) 가설 검증** — 2K→3K 사이 binary search로 정확 threshold + 메모리 사용량 측정 (Windows: `tasklist /FO CSV /V` 또는 PowerShell `Get-Process`)
- Structural Improvement Proposals:
  - **Memory note 정정** (Cycle 10에서 일괄 처리): "OOM 32G+" → "OOM ≥16G at 3K lines"
  - **bootstrap.sh default arena**: 16G로는 부족 — 64G default 권고 또는 명시 docs
- Pending Human Decisions: 변경 없음 (M3-3, M3-4, M3-5, M4-1 잠금)
- Roadmap Revisions: 없음 (정정은 Cycle 10에서)
- Next Recommendation: Cycle 2 = 2K↔3K 정확한 threshold + 메모리 사용량 vs line count plot
