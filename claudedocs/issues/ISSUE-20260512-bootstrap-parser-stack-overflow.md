# ISSUE-20260512 — Bootstrap parser stack overflow on deeply nested ASTs

## 핵심 메타

**우선순위**: P3 (single bench affected, narrow scope, Rule 6 영향 별도)
**영역**: bootstrap/compiler.bmb (parser)
**상태**: Open — separate phase

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2767) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | `bootstrap/compiler.exe build ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` |
| `observed_rate` | returncode 3221225725 (0xC00000FD = STATUS_STACK_OVERFLOW Windows) |
| `scope` | hash_table bench BMB만 (다른 bench는 정상) |
| `env_hash` | win32 / MSYS2 UCRT64 / Python 3.12.10 |

## 문제

bootstrap/compiler.exe로 `ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` (~250 LOC) build 시 STATUS_STACK_OVERFLOW 발생 + stdout/stderr empty.

테스트한 BMB 기능 단독으로는 정상:
- raw pointer + index assignment (`*i64`, `entries[idx * 3] = val`): ✅
- bitwise operators (`bxor`, `band`): ✅
- calloc/free intrinsics: ✅

→ 특정 구조 (deeply nested `let _s1 = ...; let _s2 = ...; let _s3 = ...` 또는 cascading `else if`)가 bootstrap recursive descent parser 의 stack 한도 초과 가설.

## 영향 평가

| 영역 | 영향 |
|------|------|
| Rust compiler | ✅ 영향 없음 — Rust로는 정상 빌드 |
| bench 측정 | ⚠️ Rule 6/7 path: Rust 잔존 기능을 부트스트랩 port 시 deep nesting 한계로 차단 |
| **HashMap 진단** | ⚠️ Cycle 2767 분기 ① (bootstrap-built 측정) 불가, 우회 A/B로 진단 (결과: 1.020x ≈ parity) |
| 다른 bench | ✅ 영향 없음 (대다수 bench는 부트스트랩 빌드 가능) |

CLAUDE.md "Known Failure Patterns":
> | Stage 1 통과, Stage 2 스택오버플로 | 재귀 깊이 초과 | 부트스트랩 파서의 재귀 제한 |

→ 이미 알려진 패턴. 본 cycle은 외부 bench도 같은 한계 가능성 확인.

## 추정 root cause

bootstrap parser recursive descent. 깊은 nested AST (chained let / cascading if-else / nested expressions) 처리 시 stack frame 누적. Rust parser는 explicit stack로 우회 (LALR via lalrpop generates table-driven parser → no recursion).

## 해결 방안 (Decision Framework)

### Level 1 — 언어 스펙

별도 변경 불필요. BMB 소스는 정상 BMB.

### Level 2 — 컴파일러 구조

bootstrap parser를:
- a) 스택 사용량 감소 (재귀 종료 → 트램폴린)
- b) iterative 재작성 (parser combinators 또는 generated table)
- c) 스택 크기 explicit 증가 (Windows thread stack, registry 수준 또는 link flag `-Wl,--stack=N`)

옵션 c가 가장 저비용 단기 해법.

### Level 3 — 최적화 패스
- N/A (parsing 단계)

### Level 4 — 코드 생성
- N/A

## 구현 단계 (multi-cycle scope)

1. [ ] Windows linker `--stack` 옵션으로 bootstrap exe stack 크기 증가 (예: 8MB → 64MB)
2. [ ] 회귀 테스트: Stage 1 빌드 + 3-Stage Fixed Point 유지
3. [ ] hash_table bench bootstrap 빌드 재시도

향후 (장기):
4. [ ] parser iterative 재작성 또는 lalrpop equivalent in BMB

## 완료 기준

- [ ] bootstrap이 hash_table bench 빌드 성공
- [ ] Stage 2/3 Fixed Point 유지
- [ ] 다른 deeply-nested test 회귀 없음

## 메타

- 관련 ISSUE: `ISSUE-20260413-hashmap-perf.md` (Cycle 2767 분기 ① 차단 원인)
- 인용 cycle: cycle-2767.md (발견)
- CLAUDE.md known pattern: "Stage 2 스택오버플로 — 깊은 재귀"
