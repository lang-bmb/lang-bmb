# Cycle 2663: M5-5b/c/d 근본 원인 진단 — get_node_type vs inferred type
Date: 2026-05-11

## Re-plan
Cycle 2662 carry-forward: M5-5 잔여 (M5-5b/c/d) 또는 M6 type registry 설계.
SCOPE: M5-5b (`[s; N]` var-repeat) 근본 진단으로 시작 — 가장 간단한 케이스.

## Scope & Implementation

### 1. M5-5b 테스트 케이스 작성
```bmb
fn main() -> i64 = {
    let s = "hello";
    let arr = [s; 3];
    let _p0 = println(arr[0]);  // 기대: hello, 실제: 140697513627664 (포인터)
    42
};
```

### 2. 디버그 트레이스 추가 (compiler.bmb)
- `lower_array_repeat_sb` (recursive, line 5119)에 println_str 디버그
- `step_array_repeat` (iterative, line 5395)에 println_str 디버그

### 3. 핵심 발견 — `val_type == "var"`
- `step_array_repeat` 경로 사용 (`let arr = ...` 블록 내부)
- `get_node_type(val_ast)` = **"var"** (syntactic AST node type, NOT inferred semantic type)
- 기존 분기 `val_type == "string"` 은 literal 또는 string_fns 결과 expression일 때만 true
- → var node의 경우 inferred type을 알 방법이 없음 (lowering 단계)

### 4. 근본 원인 = lowering 단계 type registry 부재
- type checker는 var의 inferred type을 추론
- 하지만 lowering 단계에서는 `get_node_type`이 syntactic 정보만 노출
- `is_string_var_sb` (str_sb registry)는 LLVM 코드젠 단계에만 사용 — lowering 단계 무관

### 5. 해결 옵션 비교
| 옵션 | 변경 범위 | 영향 |
|------|---------|------|
| A: Type-checker에서 var node에 inferred type attach | type checker + AST | 모든 lowering이 즉시 정확한 type 알 수 있음 — best |
| B: Lowering 단계 var-type registry (M6) | lowering + 새 인프라 | M5-5b/c/d + tuple String + struct array field 동시 해결 가능 |
| C: var-name 추출 후 enclosing function의 let scan | 매 lowering call마다 | O(n²) — 비효율 |
| D: 부분 fix만 (특정 컨텍스트) | 좁은 범위 | M5-5 잔여는 부분만 해결, 인프라 부재 잔존 |

→ **추천 = A** (type checker가 이미 inferred type 갖고 있음 + AST에 attach만 추가하면 단일 fix)
→ B는 더 큰 작업 (재발견한 정보를 다시 만드는 셈) — A가 실패 시 fallback

### 6. 디버그 코드 제거 + stage1 정상 복구
- 모든 println_str 디버그 제거
- `target/release/bmb.exe build bootstrap/compiler.bmb -o target/bootstrap/bmb-stage1.exe --release` 정상 빌드 ✅
- 골든 테스트 검증 (`arr_str_println`, `arr_str_alias`) ✅

## Verification & Defect Resolution

**측정 일관성**:
- bmb-stage1.exe 정상 빌드 ✅
- 기존 골든 테스트 통과 ✅ (regression 없음)
- M5-5b 케이스 = 빌드 성공하지만 dispatch 실패 (포인터 정수 출력) — 의도된 진단

**테스트 영향**:
- compiler.bmb 본 사이클 변경 없음 (디버그만 추가/제거)
- cargo test 영향 없음

## Reflection

**Scope fit**:
- 의도 = M5-5 잔여 분석 → 근본 원인 식별 ✅
- 추가 발견 = 옵션 A (type-checker 기반) > 옵션 B (M6 lower-time registry) — 실제로 더 단순한 fix 가능성

**Latent defects**:
- 디버그 코드 추가/제거 과정에서 stage1 재빌드 — 의도된 작업
- M5-5c, M5-5d는 미진단 (M5-5b와 동일 원인 추정 — 다음 사이클 검증)

**Structural improvement opportunities**:
- **AST에 inferred type attach (옵션 A)** — type-checker가 이미 정보 보유
- BMB compiler.bmb의 type-checker 출력이 실제로 어떤 정보를 포함하는지 추가 조사 필요
- 만약 type-checker가 var-name → type map을 별도 자료구조로 가지면 lowering이 lookup 가능

**Philosophy drift 점검**:
- "Workaround 없는 근본 해결" 원칙 — 옵션 A 채택이 맞음 (옵션 D 부분 fix는 거부)
- 진단 우선 → 처방 비교 → 최선 선택 ✅
- "복잡도는 기피 사유 아니다" — type-checker 변경은 더 큰 변경이지만 근본 해결

**Roadmap impact**:
- M5-5 잔여 처방 명확화 = "옵션 A 시도 → 실패 시 옵션 B"
- M6 명명 재고 가능 — "lower-time type registry"가 아니라 "type-attached AST"로 변경 검토
- 다음 사이클 (2664+): 옵션 A 가능성 조사 → 구현

**User-facing quality**:
- 진단 결과 = "포인터 정수가 출력되는 이유" 명확 설명 — 외부 reader 이해 가능
- 옵션 비교 표 = HUMAN 결정 input (큰 변경 vs 작은 변경)

## Carry-Forward
- Actionable:
  - Cycle 2664: type-checker 출력 분석 — var node에 inferred type attach 가능성 조사
  - Cycle 2665+: 옵션 A 구현 (성공 시) 또는 옵션 B fallback
- Structural Improvement Proposals:
  - AST node에 inferred type attach (type-checker 결과 보존)
  - lowering 단계의 var-type lookup 인프라 (옵션 B fallback)
  - M5-5c (`fn() -> Array<String>`), M5-5d (`p.field[i]`) 동일 원인 검증 (다음 사이클)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M6 design 명세 변경 가능성 — Cycle 2664 결과 후 결정
- Next Recommendation: Cycle 2664 — type-checker가 var에 대해 어떤 정보를 추론하는지 조사 (compiler.bmb의 typecheck path 확인)
