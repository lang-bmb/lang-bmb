# ISSUE-20260511 — `or`/`and` chain lowering eager → short-circuit

**우선순위**: P2 (Cycle 2751 가설 검증: 10-run 1.000x — 회귀 기각, 원 우선순위 유지)
**영역**: codegen, bootstrap
**상태**: Open — lowering 결함 자체는 유효, 다른 use case 영향 가능

## 측정 stamp (Cycle 2751 검증 후)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (tier3_10run_c2751 10-run, noise-gate) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | `target/benchmarks/tier3_10run_2026_05_12_c2751.json` |
| `observed_rate` | lexer **1.000x parity** ✅ (10-run noise-gate로 회귀 가설 기각) |
| `scope` | codegen `or`/`and` chain lowering 전역 (다른 character class fn 영향 가능) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 |

**측정 추이**:

| date | source | observed | 변화 |
|------|--------|----------|------|
| 2026-05-12 | tier3_10run_c2751.json (Cycle 2751) | **1.000x** ✅ | -31 pp (c2729 노이즈 정상화) |
| 2026-05-11 | tier_all_c2729.json (Cycle 2750) | 1.310x ⚠️ | +31 pp (5-run 노이즈, 기각됨) |
| 2026-05-01 | v098-tier3-10runs.json | 1.000x | -9 pp |
| 2026-04-13 | (구) tier 3 | 1.090x | (baseline) |

**Cycle 2751 검증 결과**:
- 10-run noise-gate (36s wall time, 7 Tier 3 benches) — c2729 5-run 노이즈 가설 ✅ 확인
- **시나리오 A** (회귀 재현 안 함) — alloc-optimization 등 P-track 기준 충족 후보 추적
- 다른 검증 row (json_serialize 1.120→0.870, json_parse 1.210→1.070) 동반 정상화

**Cycle 2750 진단 — 환경 변동성 가설 우위**:

Tier 3 전반 absolute 시간이 ~50% 증가 (이전 28-30ms → 현재 42-55ms). lexer 단독 회귀가 아니라 Tier 3 패턴 광범위 영향.

| bench (Tier 3) | hist bmb (ms) | new bmb (ms) | hist c (ms) | new c (ms) | bmb 증가 | c 증가 |
|---|---|---|---|---|---|---|
| brainfuck | 29 | 42 | 28 | 45 | +45% | **+61%** |
| csv_parse | 30 | 46 | 31 | 56 | +53% | **+81%** |
| http_parse | 33 | 45 | 32 | 47 | +36% | **+47%** |
| **lexer** | 28 | 51 | 28 | 39 | **+82%** | +39% |
| **json_serialize** | 28 | 55 | 34 | 49 | **+96%** | +44% |
| sorting | 121 | 157 | 133 | 166 | +30% | +25% |

**비대칭 패턴**: brainfuck/csv_parse/http_parse는 C가 BMB보다 더 슬로다운 → "환경 변동". lexer/json_serialize는 BMB가 C보다 ~2배 슬로다운 → "실제 회귀 후보 또는 BMB의 환경 변동성이 더 큰 패턴".

**가설 검증 필요 (Cycle 2751+)**:
1. 10-run noise-gate 재측정 (Tier 3) — variance 정량화
2. lexer만 isolation 측정 (다른 bench 없이) — 시스템 부하 효과 격리
3. lexer 의심 시 LLVM IR diff (historic build vs current build) — `or` chain pattern 변화 검사

**현 시점 결론**: 단일 5-run으로 P2 → 회귀 단정 불가. 우선순위 P2 유지, **재측정 carry-forward**.

**원본 ISSUE**: ISSUE-20260413-match-jump-table — 재진단 후 close 권고

## 문제

BMB의 `a or b or c ...` (또는 `and` chain)이 **eager evaluation**으로 lowering. 모든 operand를 평가 후 `or i1` 합산.

```llvm
%_t0 = icmp eq i64 %c, 43
%_t1 = icmp eq i64 %c, 45
%_t2 = or i1 %_t0, %_t1
%_t3 = icmp eq i64 %c, 42
%_t4 = or i1 %_t2, %_t3
... (등)
%_t11 = or i1 ..., ...
br i1 %_t11, %then, %else
```

LLVM `SimplifyCFG`는 **sequential branch** 패턴을 switch로 묶을 수 있으나, eager OR 패턴은 변환 대상 외. 결과: `is_operator_char` 같은 dense ASCII 비교 (11개 byte) 가 jump table 미생성.

## 검증 (Cycle 2722)

| 항목 | brainfuck (if-else if) | lexer (or chain) |
|------|----------------------|-----------------|
| BMB IR pattern | sequential `br i1` | `or i1` cascade |
| opt -O2 switch | 3 | 0 |
| asm indirect jump | 1 | 0 |
| asm LJTI refs | 53 | 0 |
| jump table 작동 | ✅ | ❌ |

LJTI 53 vs 0 — 같은 ASCII byte 비교 패턴인데 lowering 차이로 인한 결과 차이.

## 해결 방안 (Decision Framework)

### Level 1 — 언어 스펙
- `or`/`and`는 short-circuit이어야 함이 spec 명세 확인 필요
- 현재 eager lowering은 short-circuit semantics 위반 — `a or expensive_fn()` 에서 `a == true`라도 `expensive_fn` 평가됨

### Level 2 — 컴파일러 구조 (proper fix)
- AST `BinOp::Or(a, b)` → MIR으로 lowering 시 sequential branch chain 생성
- 또는 codegen에서 `or i1` 다중 chain 패턴 감지 후 sequential branch로 발행

### Level 3 — 최적화 패스
- LLVM SimplifyCFG가 BMB IR 정정 후 자동 처리 (변경 불필요)

### Level 4 — 코드 생성
- BMB IR 정정 시 자동

## 영향 범위

- **lexer 109%**: 직접 영향, 패턴 정확히 매칭
- **brainfuck 111%**: 영향 없음 (이미 if-else if pattern)
- **기타**: `is_*_char`, `is_*_alpha`, validation functions 등 character class 함수 다수

## 구현 단계 (multi-cycle scope)

1. [ ] BMB spec 검토: `or`/`and` short-circuit 명세 여부
2. [ ] AST/MIR lowering 위치 식별 (`bootstrap/compiler.bmb`)
3. [ ] `or`/`and` MIR lowering을 sequential branch로 변경
4. [ ] 회귀 테스트: 기존 boolean expression 골든 테스트
5. [ ] 부트스트랩 Stage 2/3 Fixed Point 검증
6. [ ] lexer 벤치마크 재측정

## 완료 기준

- BMB IR이 `or` chain을 sequential `br i1` chain으로 발행
- LLVM SimplifyCFG가 switch + jump table 생성 (lexer)
- lexer 벤치마크 BMB ≤ C
- Stage 2/3 Fixed Point 유지
