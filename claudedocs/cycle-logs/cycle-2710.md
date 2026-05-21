# Cycle 2710: Token packing fix 범위 추정 + 옵션 산출
Date: 2026-05-11

## Re-plan
인계: Token packing 영향 범위 + go/defer 결정. Trigger ⚪ NONE.

## Scope & Implementation

### Token packing 영향 사이트 (6 LOC)

`bootstrap/compiler.bmb`:
- line 399, 400: `pack_int_tok` — `* 1000000 + pos`, 임계 9e12
- line 506: `tok_val(r) = r / 1000000`
- line 507: `tok_end(r) = r - (r / 1000000) * 1000000`
- line 509: `make_tok(kind, endpos) = kind * 1000000 + endpos`
- line 623: `let start = tok_end(tok) - (tok_val(tok) / 1000000)` (next_token_raw 내 위치)

추가 영향 (정수 literal 임계):
- line 392, 407, 417, 428: scan_int/scan_hex_int/scan_bin_int/scan_oct_int 임계 9e12

### 충돌 조건 (smoking gun)

| 필드 | 값 |
|------|------|
| Source byte size | 1,036,359 (compiler.bmb full) |
| Token kind base | 2_000_000_000 (TK_FN=2e9+100, ...) |
| Pack scale | 1_000_000 (`kind * 1M + pos`) |
| Pos limit | < 1,000,000 |

source > 1MB → pos가 kind 영역 침범 → 다른 kind로 디코딩 → parse 실패 위치 line 1:3 (의미: position 정보 상실로 source 시작점으로 되감김).

### Fix 옵션

| 안 | 변경 | 비용 | 사용자 영향 |
|----|------|------|------------|
| **A** | 1M → 10M scale (`* 10000000 + pos`) | 6 LOC + 임계 9e12→9e11 | 정수 리터럴 한도 9.22e11 (922 billion). 미래 source 한도 10MB. |
| **B** | Bit pack `kind << 32 \| pos` | 6 LOC + 비트 연산 검증 (`@bit_shl`, `@bit_or`, `@bit_and`) | 정수 리터럴 한도 4.29e9 (4.29 billion). |
| **C** | 별도 토큰 배열 (kinds[], positions[]) | 모든 토큰 핸들러 (수십 LOC) | 영향 없음, but **대규모 재작성** |
| **D** | Defer — Stage 1만 사용, Stage 2 fixed point는 명시 차단 | 0 LOC | 부트스트랩 검증 불가 (Trusted Trust 약화) |

#### compiler.bmb 자체의 정수 literal 사용 확인

- 9.22e12 이상 literal: 없음 (있는 `9223372036854775808`은 LLVM IR 문자열 안)
- pack_int_tok의 9e12 임계: scan_int용으로 사용자 코드 정수 literal max bound. compiler.bmb 자체에 영향 없음

**A안이 compiler.bmb 자체에 영향 없이 source 한도만 10x 확장** — 가장 저비용 path.

### O(n²) AST 메모리 결함 (별개 트랙)

Cycle 2709에서 분리 확정. ≤1M source가 16GB OOM 발생.
- AST 메모리/source byte ≈ 16,000x 증폭
- compiler.bmb 985KB source → 16GB OOM
- proper fix: 문자열 기반 AST → binary or shared arena (대규모 재작성, 수개월)

이 트랙은 A안 적용 후 Stage 2가 통과해야 가설 자체가 검증 가능 — A안이 선행.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Token packing 영향 사이트 grep | ✅ 6 LOC + 4 LOC 정수 임계 |
| compiler.bmb 자체 9e12 literal 사용 | ✅ 없음 (LLVM IR 문자열만) |
| A안 변경 영향 평가 | ✅ compiler 자체 영향 없음, 사용자 literal 한도만 축소 |
| BMB 비트 연산자 지원 확인 | ⚠️ B안은 별도 검증 필요 (지금은 deferred) |

결함: 없음 (이 사이클은 진단 + 옵션 산출).

## Reflection

### 외부 관찰자 관점

1. **A안 변경량의 매력**: 6 LOC + 4 LOC 정수 임계 = 10 LOC 변경으로 부트스트랩 차단 해소. ROI 매우 높음.

2. **literal 축소 영향의 실용성**: 9.22e12 → 9.22e11. 9220억까지 가능. 일반 컴파일러 코드/일반 사용자 코드에 충분 (수치 계산 도메인 외).
   - BMB는 시스템 프로그래밍 + 컴파일러/언어 도구 1차 도메인 (ROADMAP § 1.4) — 큰 정수 literal 사용 일반적이지 않음.

3. **A안의 임시성**: 1M → 10M scale은 같은 문제의 시간 연장. 진짜 proper fix는 토큰 인코딩 자체 분리 (B 또는 C).
   - 단, 부트스트랩 회복이 우선 — A안 + B안 추후 추진 hybrid path 합리적.

4. **O(n²) 트랙의 dependency**: A안 적용 → Stage 2 token packing OK → 그 후 O(n²) AST가 다음 차단 (16G OOM). A안만으로 부트스트랩 완료 안 됨, but Stage 2 token packing 검증은 가능해짐.

### Roadmap impact

- 단기 (10 cycle 안): A안 적용 → Stage 2 token packing 부분 통과 → O(n²) OOM 차단 도달
- 장기: B안 (bit packing) + O(n²) proper fix (binary AST)
- v0.100 선언 전제 (CLAUDE.md/ROADMAP): 부트스트랩 fixed point 회복 필요 — A안만으로는 불충분, but 첫 단계

## Carry-Forward

- Actionable (Cycle 4 = 2711, **checkpoint**):
  - **결정**: A안 fix 시도 (Cycle 5-7) vs defer (Cycle 5-7 다른 자율 작업)
  - 결정 근거: (1) A안 자체는 저비용 (10 LOC), (2) O(n²) 트랙이 별개 차단 — A안 단독 효과는 진단 가치만, (3) Stage 2 token packing 검증 자체는 사용자 가치 (compiler.bmb 크기 한도 인지)
- Structural Improvement Proposals:
  - **A안 적용 후 Stage 2 token packing 통과 진단 사이클 1개** — O(n²) OOM이 진짜 다음 차단인지 검증
  - **B안 (bit packing)**: BMB 비트 연산자 지원 시 proper-fix 후보
  - **O(n²) AST 트랙**: 별도 장기 트랙으로 명시
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (정정은 Cycle 10)
- Next Recommendation: Cycle 4 checkpoint — **A안 적용 시도 권고** (저비용, 단계적 진단 가치)
