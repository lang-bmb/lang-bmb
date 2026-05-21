# Cycle 2723: `or` chain lowering 분석 (RE-PLAN from compare-inline)
Date: 2026-05-11

## Re-plan (🟡 SCOPE ADJUST)
인계 (Cycle 2722): match-jump-table false positive 확정. 실제 원인은 `or` chain eager lowering. compare-inline 대신 `or` lowering 분석.

## Scope & Implementation

### 검증: mem2reg 작동 여부

raw IR (lexer_2722.ll) vs opt IR (lexer_2722_opt.ll):
- alloca i1: **173 → 0** (mem2reg가 모두 정리)
- switch instructions: 1 → 0 (lowered)
- icmp eq: 80 (raw)

**결론**: alloca/store/load는 opt가 처리. 진짜 원인은 **eager OR**.

### Eager OR vs Short-circuit OR

**Eager (현재 BMB)**:
```llvm
%b0 = icmp eq c, 43
%b1 = icmp eq c, 45
%b01 = or i1 %b0, %b1
%b2 = icmp eq c, 42
%b02 = or i1 %b01, %b2
...
br i1 %final, %then, %else
```

**Short-circuit (target)**:
```llvm
%b0 = icmp eq c, 43
br i1 %b0, %then, %check1
%check1: %b1 = icmp eq c, 45
br i1 %b1, %then, %check2
...
```

LLVM SimplifyCFG는 후자를 switch + jump table로 변환. 전자는 변환 불가.

### `or` lowering site (bootstrap/compiler.bmb)

| Line | 내용 |
|------|------|
| 529 | tokenizer: `"or"` → `TK_OR` |
| 14766 | `bor` → `llvm_gen_binop("or", ...)` |
| 14778 | `or` → `llvm_gen_binop("or", ...)` |
| 6658 | `llvm_gen_binop("or", ...)` → `or i64 lhs, rhs` (i64 binary OR) |

BMB의 `or`은 i64 binary OR로 lowering된다. 그런데 lexer IR엔 `or i1`이 있다. 다른 path가 boolean OR를 처리.

→ 추정: `if a or b { ... }` 조건식의 `or`는 boolean context로 추론되어 `or i1` lowering. 그 이전에 alloca i1 + store + load 패턴이 추가됨 — MIR/IR generator가 모든 SSA temp를 메모리로 처리.

### Proper fix scope (Decision Framework)

| 수준 | 변경 | 사이클 비용 |
|------|------|-----------|
| 1. 언어 스펙 | `or`/`and` short-circuit 명세 검토 | 1 cycle (spec read) |
| 2. 컴파일러 구조 | AST/MIR lowering: `or` → sequential branch | **3-5 cycles** (proper fix) |
| 3. 최적화 패스 | (BMB IR 정정 시 자동) | — |

**결론**: 단일 cycle 부적합. ISSUE 등록 + multi-cycle phase로 carry-forward.

### 산출물

1. **새 ISSUE 등록**: `ISSUE-20260511-or-chain-lowering.md` — P1, multi-cycle scope
2. **기존 ISSUE close**: `ISSUE-20260413-match-jump-table.md` → `closed/`
   - Close 사유 명시 (Cycle 2722 재진단)
   - 후속 ISSUE 링크
3. **active 25 → 25** (1 close + 1 new = 동일)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| mem2reg 작동 확인 (alloca i1 173→0) | ✅ |
| Eager OR 패턴 라이브러리 식별 | ✅ |
| `or` lowering site grep | ✅ (compiler.bmb 4 사이트) |
| Proper fix scope estimation | ✅ multi-cycle (3-5) |
| 새 ISSUE 등록 | ✅ ISSUE-20260511-or-chain-lowering.md |
| 기존 ISSUE close + 이동 | ✅ closed/ISSUE-20260413-match-jump-table.md |

결함: 없음.

## Reflection

### 외부 관찰자 관점

1. **proper fix 인식의 정확성**: 단일 cycle 압박 속에서 "compiler IR generator 변경"이라는 본질적 범위 파악. Workaround 유혹 (사용자 코드 lookup table) 거부 (CLAUDE.md Principle 2).

2. **Multi-cycle phase 분리의 가치**: 1 cycle에서 다른 다중 cycle 작업 (HashMap, Alloc, `or` lowering) 모두 처리하려는 무리한 시도 회피. 별도 phase 또는 next session.

3. **재진단 → 새 ISSUE 등록 → 기존 close 사이클의 학습**: 잘못된 진단을 빠르게 close, 정확한 진단으로 새 ISSUE. 백로그 품질 유지.

4. **Decision Framework 적용**: Level 2 (컴파일러 구조) 답이 명확하지만 비용 측면에서 multi-cycle. "복잡도는 기피 사유가 아니다"이지만 1 cycle 안에 무리하게 시도하는 것은 다른 문제 — phase 분리가 정상.

### Roadmap impact

- ISSUE 백로그: 25 → 25 (1 close + 1 new)
- Cycle 2724-2725 plan: compare-inline + StringBuilder SSO 유지 (단일 cycle 가능)
- `or` chain proper fix는 next session multi-cycle phase 후보

## Carry-Forward

- Actionable (Cycle 2724):
  - **StringBuilder SSO** — single cycle, fasta 108% 대상 (Cycle 2721 plan 유지)
- Structural Improvement Proposals:
  - **`or`/`and` lowering proper fix phase** — next session, 3-5 cycles
  - **HashMap / Alloc multi-cycle phase** — next session
  - **compare-inline (MIR threshold)** — Cycle 2725 후보 또는 carry-forward
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (Cycle 2722 결과 반영 — Phase 2가 정상 진행)
- Next Recommendation: Cycle 2724 = StringBuilder SSO 분석 + 단일 cycle 가능 여부 확인
