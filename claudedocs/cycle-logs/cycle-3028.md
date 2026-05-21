# Cycle 3028: MIR AndChainCSE 구현
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3027): MIR CSE 값 동등성 인프라 + `and_rhs` 패턴 감지 구현.
계획 유효. 설계도 완성됐으므로 phi 삽입·Copy 교체까지 한 번에 구현.

## Scope & Implementation

### 구현 내용

`bmb/src/mir/optimize.rs`에 `AndChainCSE` optimization pass 신규 추가:

**핵심 알고리즘:**
1. **Phase 1**: 각 `and_rhs_*` 블록에서 pure load call(`load_u8` 등)과 canonical 주소 키 수집
2. **Phase 2**: `and_merge_*` 블록 탐색 — phi 소스 레이블에서 `and_rhs_N`과 `and_false_N` 확인 후, branch `then_label`이 `and_rhs_P`인 구조 감지. 두 `and_rhs` 블록의 canonical load key가 일치하면 변환 대상으로 등록
3. **Phase 3**: 각 변환 대상마다:
   - `and_merge_*` 블록에 phi 삽입: `%_and_cse_N = phi(%t1 from and_rhs_N, 0 from and_false_N)`
   - `and_rhs_P` 블록에서 중복 Call → Copy 교체: `%t4 = Copy(%_and_cse_N)`
   - `func.locals`에 `MirType::I64` 타입으로 phi dest 등록

**Canonical form 기반 값 동등성:**
- `param:{name}` — 함수 파라미터
- `var:{name}` — 도미네이팅 블록의 외부 변수 (루프 phi 등)
- `const:{debug_repr}` — 상수
- `binop:{op:?}:{lhs_canon}:{rhs_canon}` — 이항 연산 (재귀적)

두 `and_rhs` 블록이 `binop:Add:param:ptr:var:pos_phi` 같은 동일 canonical key를 가지면 같은 주소에 대한 load임을 판단.

**Safety**: `and_rhs_P`는 `and_merge`의 phi가 true일 때만 도달 가능. phi=true는 `and_rhs_N`에서만 제공됨 → `%t1` 항상 사용 가능.

**변경 파일:**
- `bmb/src/mir/optimize.rs`: `AndChainCSE` struct + impl + 4개 unit test 추가
- `bmb/src/mir/mod.rs`: `AndChainCSE` export 추가
- Release/Aggressive 파이프라인: `GlobalFieldAccessCSE` 직후 `AndChainCSE` 추가

### IR 검증 결과

```bmb
fn count_non_newline(ptr: i64, len: i64) -> i64 = {
    while pos < len and load_u8(ptr + pos) != 10 and load_u8(ptr + pos) != 13 {
        count = count + 1; pos = pos + 1
    }; count
};
```

**변환 전** (이론적): `load i8` 2회
**변환 후** (실제): `grep -c "load i8"` = **1** ✅

LLVM IR 확인:
```llvm
bb_and_rhs_3:
  %_t5.u8.0 = load i8, ptr %gep_elem.0   ; ← 첫 번째 load (유지)
  %_t6 = icmp ne i64 ..., 10

bb_and_merge_5:
  %_t3 = phi i1 [%_t6.phi.and_rhs_3, ...], [0, ...]
  %_and_cse_1 = phi i64 [%_t5.phi.and_rhs_3, bb_and_rhs_3], [0, bb_and_false_4]  ← CSE phi

bb_and_rhs_6:
  %_t10 = icmp ne i64 %_and_cse_1, 13   ; ← load i8 없음! phi 재사용
```

## Verification & Defect Resolution

- `cargo test --release --lib -- test_and_chain_cse`: **4/4 PASS** ✅
  - `test_and_chain_cse_eliminates_duplicate_load` ✅
  - `test_and_chain_cse_adds_phi_to_merge_block` ✅
  - `test_and_chain_cse_different_loads_no_change` ✅
  - `test_and_chain_cse_name` ✅
- `cargo test --release`: **3782+2390+22+23 PASS, 0 FAIL** ✅

## Reflection

- **Scope fit**: Cycle 3027에서 설계한 모든 내용을 한 사이클에 구현 완료 (phi 삽입 포함).
- **성능 기대치**: and-chain 패턴에서 `load_u8` 2회 → 1회. Cycles 3022-3023 실측(csv -12.7pp, http -2.9pp)과 동등한 효과를 컴파일러가 자동으로 달성.
- **Philosophy fit**: workaround(break-based 패턴 수동 적용) 없이 컴파일러가 자동으로 최적화. Principle 2 (Workaround 금지) 준수.
- **Roadmap impact**: ISSUE-20260521-mir-cse-and-chain P2 구현 완료. 다음 사이클에서 P-track 벤치마크로 효과 확인 필요.
- **Rule 7 고려**: inkwell backend (`codegen/llvm.rs`)는 현재 text backend 경유하지 않음 — MIR CSE는 MIR 수준 변환이므로 양 backend에 자동 적용됨. Rule 7 별도 조치 불필요.

## Carry-Forward

- Actionable: Cycle 3029 = P-track 벤치마크 검증 (double-load 패턴 원본 파일로 성능 측정) + ISSUE close
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3029 = 벤치마크 검증 (원본 double-load 패턴 vs break-based vs CSE 자동 최적화 비교)
