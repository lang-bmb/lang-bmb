# ISSUE-20260413 — match → jump table 컴파일

**우선순위**: P0
**영역**: codegen, mir
**상태**: ✅ **CLOSE — Cycle 2722 재진단 (false positive)**
**관련 벤치마크**: brainfuck (111%), lexer (109%)
**후속 ISSUE**: `ISSUE-20260511-or-chain-lowering.md` — lexer slowness 진짜 원인

## Cycle 2722 재진단 결과 (2026-05-11)

| 벤치마크 | jump table 작동? | 실제 원인 |
|---------|----------------|----------|
| brainfuck | ✅ 53 LJTI refs + 1 indirect jump | 별도 진단 필요 (string ops, function call overhead) |
| lexer | ❌ 0 LJTI refs | BMB `or` chain eager lowering — sequential branch 불가 |

→ 원진단 ("match → switch 매핑 필요")은 brainfuck에 대해 false positive. lexer 원인은 `or` lowering. **본 ISSUE close**, ISSUE-20260511-or-chain-lowering으로 대체.

## (이하 원본 보존 — close 표시)


## 재진단 결과 (2026-04-13, Cycle 362)

원래 진단("MIR Switch 명령 + LLVM switch 매핑 최적화 필요")은 **기 구현 상태 미확인**. 인프라는 이미 존재:
- MIR `Switch` terminator (`bmb/src/mir/mod.rs:797`)
- `IfElseToSwitch` optimization pass (`bmb/src/mir/optimize.rs:3541`)
- LLVM codegen emits `switch` instruction (`llvm.rs:5231`, `llvm_text.rs:7401`)
- Match lowering → Switch terminator for literal patterns (`lower.rs:1820`)

brainfuck/lexer slowness의 실제 원인은 다른 곳. 재조사 범위:
- byte-level dispatch: `char_at()` 오버헤드
- byte → op 매핑 루프 최적화
- LLVM이 실제로 jump table을 생성하는지 IR 수준 검증 (`--emit-ir` + objdump)

이 이슈는 **재스코프 후 재개**해야 함.

## 문제

BMB의 `match` 표현식이 긴 if-else 체인으로 컴파일되어 인터프리터 스타일 코드(brainfuck, lexer)에서 C 대비 9-11% 성능 저하. C의 `switch`는 LLVM이 jump table로 컴파일하지만, BMB `match`는 동등한 최적화를 받지 못함.

### 증거

| 벤치마크 | BMB | C | 비율 |
|----------|-----|---|------|
| brainfuck | 4.2ms | 3.8ms | 111% |
| lexer | 4.4ms | 4.0ms | 109% |

브랜치 디스패치 오버헤드가 핫 루프에 누적되면서 캐시 미스/분기 예측 실패 증가.

## 근본 원인 분석

1. MIR lowering에서 `match` → 연속된 `CondBranch` 명령으로 전개
2. LLVM은 이 패턴을 switch 명령으로 재구성하지 못하거나 jump table 생성 threshold 미달
3. 패턴이 연속된 정수 상수 (예: `0..255` 바이트)인데도 jump table 생성 안됨

## 해결 방안 (Decision Framework 순서)

### Level 1 — 언어 스펙 (불필요)
- `match` 시맨틱은 이미 switch와 동등. 스펙 변경 불필요.

### Level 2 — 컴파일러 구조 (MIR Switch 명령)
- MIR에 `Switch(scrutinee, cases: Vec<(Constant, BlockId)>, default: BlockId)` 명령 추가
- AST의 `Expr::Match`에서 모든 arm이 정수 상수 패턴이면 `Switch`로 lowering

### Level 3 — 최적화 패스
- 기존 MIR `IfElse → Switch` 패스가 있으나 동작 확인 필요 (`optimize.rs` 라인 grep)
- 연속 구간 감지 → jump table hint

### Level 4 — 코드 생성 (LLVM)
- LLVM IR `switch i64 %v, label %default [i64 0, label %b0 i64 1, label %b1 ...]`
- LLVM이 자동으로 dense switch → jump table 변환

## 구현 단계

1. [ ] `bmb/src/mir/mod.rs` — `Switch` 명령 정의
2. [ ] `bmb/src/mir/lower.rs` — `Expr::Match` → `Switch` lowering (정수 상수 패턴 탐지)
3. [ ] `bmb/src/codegen/llvm_text.rs` — `Switch` → LLVM `switch` 명령 방출
4. [ ] `bootstrap/compiler.bmb` — 동등한 구현 포팅
5. [ ] 골든 테스트: `tests/golden/test_golden_match_*.bmb` 검증
6. [ ] 벤치마크 재실행: brainfuck, lexer 목표 ≤ 100%

## 완료 기준

- brainfuck, lexer 벤치마크에서 **BMB ≤ C** 달성
- 기존 match 골든 테스트 회귀 없음
- 3-Stage 부트스트랩 통과 (Fixed Point)
- LLVM IR 확인: `switch` 명령 사용 확인
