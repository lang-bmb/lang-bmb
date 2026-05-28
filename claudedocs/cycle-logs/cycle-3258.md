# Cycle 3258: M14 Phase 2 — gotgan verify (해시 검증)
Date: 2026-05-29

## Re-plan

Plan valid. M14 Phase 2: gotgan build시 해시 검증 → `gotgan verify` 명령 구현.

## Scope & Implementation

### 변경 (ecosystem/gotgan-bmb/gotgan.bmb)

1. `verify_single_dep()` — 단일 dep의 SHA-256 비교
2. `verify_dep_hashes_loop()` — 모든 dep 검증
3. `gotgan_verify()` — `gotgan.lock` 읽어 해시 검증
4. `main()` + `print_help()` 업데이트

### 테스트

- `gotgan verify` (hash 일치) → `All deps verified OK` ✅
- `gotgan verify` (파일 변경 후) → `MISMATCH: mylib`, exit 1 ✅

## Carry-Forward

- **Actionable**: M12 Phase 3 (Z3 effect constraint) 또는 새 기능
- **Next Recommendation**: 사이클 3259-3260으로 M15 Phase 1 시작 or 안정화
