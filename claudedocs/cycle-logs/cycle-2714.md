# Cycle 2714: 1-arg builtin arity guard 확장 (7 i64 builtins × 2 paths)
Date: 2026-05-11

## Re-plan
인계 (Cycle 2713): 1-arg builtin guard 7개 핵심 i64. Trigger ⚪ NONE.

## Scope & Implementation

### 신규 helper

`call_has_one_arg` (line 7042 인근):
```bmb
fn call_has_one_arg(line: String, paren_pos: i64, close_pos: i64) -> bool =
    let args = line.slice(paren_pos + 1, close_pos);
    if args.len() == 0 { false }
    else { find_char(args, 0, 44) >= args.len() };
```

### 변경 사이트 (14 분기, 2 paths)

| Builtin | lower_expr_sb | step_expr |
|---------|---------------|-----------|
| `@popcount` | ✅ | ✅ |
| `@clz` | ✅ | ✅ |
| `@ctz` | ✅ | ✅ |
| `@bit_reverse` | ✅ | ✅ |
| `@bit_not` | ✅ | ✅ |
| `@abs`/`@bmb_abs` | ✅ | ✅ |
| `@bswap` | ✅ | ✅ |

각 builtin: 2개 분기 (arity-guarded builtin + fallback to `emit_regular_i64_call`).

**FP 1-arg builtin defer**: `@fabs`, `@floor`, `@ceil`, `@round`, `@sqrt`, `@sin`/`@cos`/etc. — 사용자 충돌 가능성 낮음, 사이클 부담 줄임 (Cycle 2714 Carry-Forward).

### compiler.bmb 크기 추이

| Cycle | bytes | delta |
|-------|-------|-------|
| 2711 | 1,036,359 | (token packing) |
| 2712 | 1,039,623 | +3,264 (2-arg/3-arg guard) |
| 2714 | 1,042,127 | +2,504 (1-arg guard) |

총 5,768 bytes 증가. 32G arena 한도 안 유지.

## Verification & Defect Resolution

| 게이트 | 결과 |
|--------|------|
| Gate 1: Stage 1 빌드 | ✅ 10.4s |
| Gate 2: Simple BMB compile | ✅ |
| **Gate 3: 1-arg builtin 회귀** (popcount(255)+abs(-7)+bit_not(0)) | ✅ stdout `14` (8+7-1=14) |
| **Gate 4: User-defined popcount(x,y) 2-arg fallback** | ✅ stdout `42` (10+32=42) |
| Gate 5: cargo test --release | ✅ **6210/6210** |
| Gate 6: Stage 2 + Stage 3 + Fixed Point | ✅ S2 == S3 (32G, 28s) |

결함: 없음. proper-fix 적용 완료.

## Reflection

### 외부 관찰자 관점

1. **Gate 4의 critical 의미**: user-defined `popcount(x, y)` 2-arg fallback이 정상 작동. Cycle 5 (set_cover) + Cycle 7 (popcount user-fn) 두 케이스 검증으로 fallback path가 robust함을 증명.

2. **mechanical 확장의 안전성**: 16 + 14 = 30 사이트 분기 추가됐고 모두 동일 패턴 (rotate_left/right 확립 패턴). compiler.bmb +5,768 bytes 증가했으나 32G 한도 안 안정.

3. **lint 11 + arity guard = defense-in-depth**:
   - lint 단: 사용자 코드 정적 감지 (Cycle 2703)
   - 컴파일러 단: arity 가드 + fallback (Cycle 2712 + 2714)
   - 사용자가 lint 무시해도 silent corruption 안 일어남

### Roadmap impact

- Cycle 2697 workaround (source rename) **완전 회수** — proper fix 확립
- 1-arg FP builtin은 Cycle 2714 Carry-Forward (낮은 우선순위)
- Cycle 2712 Carry-Forward의 "1-arg builtin guard" 완료

## Carry-Forward

- Actionable (Cycle 8 = 2715):
  - **측정 강화** — Tier 3 ≥10 runs (knapsack clang outlier 재확인) or 골든 sample 풀 검증
- Structural Improvement Proposals:
  - **FP 1-arg builtin guard** (consistency): `@fabs`, `@floor`, `@ceil`, `@round`, `@sqrt`, `@sin`/`@cos`/`@tan`/etc. — 우선순위 낮음
  - **2-arg FP builtin guard**: `@f64_min`/`@f64_max`, `@pow_f64`, `@atan2`, `@fmod` — 같은 카테고리
  - **CI 게이트로 Fixed Point + builtin arity 회귀 테스트**
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (정정 Cycle 10)
- Next Recommendation: Cycle 8 = 측정 강화 (Tier 3 ≥10 runs) 또는 골든 sample 풀 검증
