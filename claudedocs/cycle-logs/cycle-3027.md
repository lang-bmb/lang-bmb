# Cycle 3027: MIR CSE 조사 — and/or 체인 이중 load 근본 원인
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3026): MIR CSE P2 구현 검토. 계획 유효.

## Scope & Implementation

### IR 분석 결과

`while cond and load_u8(ptr+pos) != 10 and load_u8(ptr+pos) != 13` 패턴에 대해:

**LLVM opt -O2 결과** (simple @export 함수에서):
```llvm
%_t8.u8.0 = load i8, ptr %gep_elem.0, align 1, !tbaa !0
switch i8 %_t8.u8.0 [i8 10 → exit, i8 13 → exit, default → body]
```

→ LLVM이 단일 load + switch로 최적화 성공 (GVN 패스).

**BUT**: 복잡한 함수 내부 (recursive benchmark, many locals, inlined calls)에서는 LLVM이 동일 최적화를 보장하지 못함. Cycle 3022에서 12.7pp 개선 확인.

### MIR 구조 이해

`BinOp::And` 단락 평가 lowering (mir/lower.rs:981):
```
entry → Branch(lhs, and_rhs_N, and_false_M)
and_rhs_N: [compute rhs; phi_result = cmp] → Goto(and_merge_K)
and_false_M: → Goto(and_merge_K)
and_merge_K: phi(rhs_result, false) → Branch(phi, and_rhs_P, and_false_Q)
and_rhs_P: [duplicate pure call!] → Goto(and_merge_R)
```

**근본 원인**: `and_rhs_P`는 `and_rhs_N`에 의해 CFG상으로 dominate되지 않음
(both `and_rhs_N`→merge와 `and_false_M`→merge→`and_rhs_P` 경로 존재).
따라서 LLVM GVN이 cross-block CSE를 보장하지 못함.

### 설계 결정

**구현 전략**: MIR post-lowering 최적화 패스 (mir/optimize.rs)
- `and_rhs_N`과 `and_rhs_P` 사이에서 순수 함수 호출 CSE 감지
- `and_merge_K`에 phi 노드 추가 (`result = phi(computed_in_N, undef from false)`)
- `and_rhs_P`의 중복 Call → Copy(phi_result)

**순수 함수 whitelist**: `load_u8` (우선), 필요 시 `byte_at` 추가

**값 동등성**: BinOp 결과 Value numbering (재귀적 canonical form 비교)

## Verification & Defect Resolution

조사 전용 사이클. 테스트 없음.

## Reflection

- **Scope fit**: 근본 원인 파악 + 구현 설계 완료.
- **Key finding**: LLVM CAN optimize simple cases (GVN). Complex code may not benefit. BMB MIR CSE가 일관성 있는 최적화를 보장하는 근본 해결책.
- **Roadmap impact**: 다음 2-3 사이클: MIR CSE 구현 → 자연 `and` 패턴 자동 최적화.

## Carry-Forward

- Actionable: Cycle 3028 = MIR CSE 값 동등성 인프라 + `and_rhs` 패턴 감지 구현
- Structural Improvement Proposals: 없음 (MIR CSE가 이 issue의 근본 fix)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3028 = MIR CSE 구현 시작
