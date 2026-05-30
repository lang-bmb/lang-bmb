# Cycle 3332: bootstrap-bare-filename-sigsegv 수정
Date: 2026-05-30

## Re-plan
SCOPE ADJUST — 상속된 Actionable(L1 stack-allocated tuple ABI)은 Pending Human Decision 미결이므로 착수 불가.
P3 correctness bug `bootstrap-bare-filename-sigsegv`(ISSUE-Cycle 3249)로 전환: 근본 원인 분석 + 1-사이클 수정 가능성 평가.

## Scope & Implementation

**목표**: Bootstrap compiler가 bare filename 입력 시 SIGSEGV(STATUS_ACCESS_VIOLATION) 발생하는 버그 수정.

**근본 원인 분석**:
`include_dirname_scan`(line ~247)에 `pre i >= 0` contract가 있으나, 함수 body는 `i = -1`을 정상적으로 처리하도록 설계됨:
- `include_dirname("bare.bmb")` → `include_dirname_scan(path, 7)` → ... → `include_dirname_scan(path, -1)` → **`pre i >= 0` 위반**
- `include_dirname("./bare.bmb")` → separator(`.`) 발견 시 i=1에서 종료 → i=-1 호출 없음

`inject_contract_assumes_all`이 `pre i >= 0` → `llvm.assume(i >= 0)` 주입. i=-1 재귀 호출 시 `llvm.assume(false)` → LLVM UB → 비결정적 SIGSEGV. 이것이 정확히 CLAUDE.md § "메타순환 계약 위반" 패턴 (Cycle 3232 `post it >= 0` → `post it >= -1`의 계약 선행 버전).

**왜 비결정적인가**: LLVM optimizer가 `assume(false)`를 이용해 일부 branch를 DCE/변환하는 방식이 heap layout에 따라 다름 → 4/5 crash, 1/5 success (실측).

**왜 `include_dirname` → `"."` 반환이 workaround인가**: `src_dir`이 `"."` vs `""` 차이는 arena allocation size를 미미하게 바꿔 UB 발현을 억제할 뿐, `llvm.assume(false)` 자체는 남음. CLAUDE.md Principle 2 금지 항목.

**실제 수정 (1-char fix)**: `bootstrap/compiler.bmb` line 247:
```
pre i >= 0   →   pre i >= -1
```
이유: 함수 body가 `i = -1`을 정상 base case(`if i < 0 { -1 }`)로 처리. 재귀 호출 `include_dirname_scan(path, i - 1)`에서 `i = 0`일 때 `i - 1 = -1 >= -1` ✓. `post it >= -1`은 변경 없음.

**변경 파일**: `bootstrap/compiler.bmb` (line 247, 1문자: `0` → `-1`)

## Verification & Defect Resolution

- **Stage 1 빌드**: ✅ `compiler_s1.exe` 빌드 성공 (38s)
- **Bare filename 10회 반복**: ✅ 10/10 PASS, 0 CRASH (수정 전: ~4/5 crash)
- **경로 형식 전수 확인**: ✅ absolute / with-separator / `./` / bare — 모두 exit=0
- **Within-gen Fixed Point**: ✅ fp3332a.ll == fp3332b.ll
- **cargo test --release**: ✅ 6282 PASS, 0 FAILED (3800+47+22+2390+23)
- **lint warnings**: ✅ 180 pre-existing (변화 없음)

**결함 발견 없음.** ISSUE `bootstrap-bare-filename-sigsegv.md` → `closed/` 이동.

## Reflection

- **Scope fit**: P3 correctness bug 완결. `llvm.assume(false)` UB 제거 — 근본 수정.
- **Latent defects**: 없음.
- **Structural improvements**: 없음. 1-char fix, minimal footprint.
- **Philosophy drift**: CLAUDE.md Principle 2 준수 — workaround(`include_dirname` → `"."`) 거부, 근본 원인(잘못된 precondition) 수정.
- **Roadmap impact**: Active ISSUE 5→4. HANDOFF의 "bootstrap-bare-filename-sigsegv" 제거.
- **패턴 문서화**: 이 버그 클래스 (`pre i >= N` but recurse to N-1)는 메타순환 계약 위반과 동일 패턴. CLAUDE.md Known Failure Patterns에 추가 권장.

## Carry-Forward
- Actionable: L1 stack-allocated tuple ABI (csv 1.039× 근본 해결) — Human Decision 대기
- Structural Improvement Proposals: CLAUDE.md Known Failure Patterns에 "`pre i >= N` 재귀 하한 위반" 패턴 추가 (이번 버그 클래스 예방) — P4
- Pending Human Decisions: csv 1.039× 측정 노이즈 허용 여부 (결정 시 L1 ABI 착수)
- Roadmap Revisions: Active ISSUE 5→4 (bootstrap-bare-filename-sigsegv CLOSED)
- Next Recommendation: L1 stack-allocated tuple ABI Phase 1 (human decision 후), 또는 CLAUDE.md 패턴 추가 (P4 자율)
