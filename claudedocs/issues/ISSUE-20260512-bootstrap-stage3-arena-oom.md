# ISSUE-20260512 — Bootstrap Stage 3 arena OOM (32G 이하)

## 핵심 메타

**우선순위**: P1 (bootstrap correctness — Fixed Point 검증 불가)
**영역**: bootstrap / arena / compiler.bmb self-compile
**상태**: Open — Cycle 2777 발견. Stage 2 통과, Stage 3 실패.
**estimated_cycles**: 2-4 (hypothesis)

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2777) |
| `stale_after` | 2026-09-12 |
| `measurement_source` | `./bootstrap/bmb_stage2 bootstrap/compiler.bmb -o bootstrap/bmb_stage3` |
| `observed_rate` | BMB_ARENA_MAX_SIZE=32G → OOM. 64G → 시스템 OOM (실제 RAM 부족) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 |

## 문제

Stage 3 빌드 (Stage 2 바이너리 = bootstrap binary로 compiler.bmb 컴파일) 시 32G arena 소진.
이전 Fixed Point 달성 (Cycle 2711-2714, 32G)과 비교하면 regression 가능성 있음.

```
$env:BMB_ARENA_MAX_SIZE = "32G"
./bootstrap/bmb_stage2 bootstrap/compiler.bmb -o bootstrap/bmb_stage3
# [bmb] FATAL: arena memory limit exceeded (32768 MB / 32768 MB max)
```

## 분석

1. Stage 2 빌드 (Stage 1 = Rust binary → compiler.bmb): 32G로 성공 ✅
2. Stage 3 빌드 (Stage 2 = bootstrap binary → compiler.bmb): 32G 실패 ❌

Stage 1 (Rust-built, memory-efficient) vs Stage 2 (BMB-compiled, potentially less efficient) 차이.
Stage 2 바이너리가 같은 소스 컴파일 시 더 많은 arena를 사용하는 것으로 추정.

**Cycle 2777 수정(`llvm_text.rs param_set`)과의 관련성**:
`compiler.bmb`에 직접 `store_u8/load_u8` 호출 없음 → 수정이 Stage 2 동작에 영향 없음.
Stage 3 OOM은 이번 수정의 side effect가 아니라 선행 regression 또는 known issue.

Known failure pattern (CLAUDE.md): "Stage 2 arena OOM (32G+) | 문자열 기반 AST O(n²) 성장".

## 조사 항목

- [ ] Cycle 2711-2714 이후 `compiler.bmb` 변경 사항 확인 (`git log bootstrap/compiler.bmb`)
- [ ] Stage 2 바이너리 arena 사용량 프로파일링
- [ ] `BMB_ARENA_MAX_SIZE=48G` 시도 (시스템 여건에 따라)
- [ ] 소스 내 O(n²) arena 성장 hotspot 식별

## 종결 기준

- [ ] Stage 3 빌드 32G 이하에서 성공
- [ ] Stage 2 == Stage 3 Fixed Point 확인 (`diff`)
