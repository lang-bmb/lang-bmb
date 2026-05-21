# Cycle 2712: Builtin arity proper fix — 2-arg/3-arg + Cycle 2697 회수
Date: 2026-05-11

## Re-plan
인계 (Cycle 2711): Cycle 5-7은 builtin arity proper fix / lint 확장 / 측정 강화. 첫 자율 트랙으로 builtin arity 진입.
advisor 자문: 2-arg + 3-arg only (1-arg은 Cycle 6), set_cover source rename 회수가 go/no-go.
Trigger ⚪ NONE.

## Scope & Implementation

### 변경 패턴: rotate_left/right 확립 패턴 확장

기존 (Cycle 2697):
```bmb
} else if fn_name == "@bit_or" {
    // 2-arg builtin path (silent corruption on user-defined bit_or with !=2 args)
}
```

신규 (Cycle 2712 — rotate_left/right와 동일 패턴):
```bmb
} else if fn_name == "@bit_or" and call_has_two_args(line, paren_pos, close_pos) {
    // 2-arg builtin path
} else if fn_name == "@bit_or" {
    emit_regular_i64_call(line, paren_pos, close_pos, fn_name, dest)
}
```

### 변경 사이트 (총 16 분기, 2 lowering paths)

**lower_expr_sb** (line 7136-7211):
- `@bit_and`, `@bit_or`, `@bit_xor`, `@bit_shift_left`, `@bit_shift_right` (5)
- `@min`/`@bmb_min`, `@max`/`@bmb_max` (2)
- `@clamp`/`@bmb_clamp` (1) — 3-arg, `count_commas == 2` 가드

**step_expr** (line 7405-7491):
- 동일 8 분기 (mirror)

`count_commas` (line 14452) 재사용으로 3-arg 가드 신규 helper 없이 구현.

## Verification & Defect Resolution

### Gates

| 게이트 | 결과 |
|--------|------|
| Gate 1: Stage 1 빌드 | ✅ 10.4s |
| **Critical Gate: set_cover source rename 회수** | ✅ **`bit_or` user-fn 3-arg 정상 컴파일 + 실행 stdout "2"** |
| Gate 3: cargo test --release | ✅ 6210/6210 (no regression) |
| Gate 4: Stage 2 + Stage 3 + Fixed Point | ✅ S2 == S3 (32G arena, 28s) |

**Cycle 2697 workaround 회수 가능** — source의 `bits_or_n` → `bit_or` 되돌려도 동작. Cycle 5 결과로 source 회수 완료.

### 회귀 발견

**16G arena 한도 초과** — Cycle 2711 시점 26.7s 16G OK였으나 Cycle 2712 변경 후 16G OOM.

| 시점 | source bytes | Stage 2 16G | Stage 2 32G |
|------|-------------|-------------|-------------|
| Cycle 2711 | 1,036,359 | ✅ 26.7s | n/a |
| Cycle 2712 | 1,039,623 (+3,264) | ❌ OOM 16384MB | ✅ 28.2s |

3.3KB 증가가 메모리 한도를 ~+6GB 넘김. **O(n²) AST 메모리 트랙 부분 재인정** (Cycle 2711의 "OOM 가설 무효" 결론 정정).

올바른 모델:
1. **Token packing 1MB overflow** — primary blocker (Cycle 2711 fix)
2. **O(n²) AST memory growth** — secondary, source size 증가에 따라 비례 폭발

결함: 없음 (회귀는 진단 차원 — 32G로 회피 가능). **소스 회수 + 16 사이트 arity guard 적용 완료**.

## Reflection

### 외부 관찰자 관점

1. **Cycle 2711 vs Cycle 2712 OOM 모델 정정**: Cycle 2711에서 "OOM은 token packing 부작용"이라 결론냈으나, 실제로는 token packing fix 후에도 O(n²) 메모리가 살아있음. 16G 한도 초과 임계가 source 크기에 매우 sensitive — Cycle 2712의 +3KB가 6G 메모리 한도를 넘긴 사례.

2. **Cycle 2697 workaround 회수의 의미**: 단순 source rename → **컴파일러 차원 proper fix**. lint 11 (builtin_name_collision)에 더해, 이제 컴파일러 자체가 user-defined 함수 정상 처리. 이중 안전망.

3. **scope fit**: advisor 권고 (2-arg + 3-arg only) 정확히 따름. 16 사이트 mechanical 확장 + critical gate (set_cover) 검증.

### Roadmap impact

- **Bootstrap default arena**: bootstrap.sh의 `BMB_ARENA_MAX_SIZE=16G` → **32G 권고**. 또는 명시 문서화 (CLAUDE.md "Known Failure Patterns" 표 추가)
- **1-arg builtin guard**: Cycle 6 후보 (consistency). 위험 케이스 (user-fn `bit_not(x, mode)` 등)는 일관성 차원
- **O(n²) AST 트랙**: 별도 장기 트랙으로 명시 (string-based AST → binary 또는 shared arena, 수개월)

## Carry-Forward

- Actionable (Cycle 6 = 2713):
  - **bootstrap.sh 한도 16G → 32G 변경** (필수, 즉시) — Cycle 2711 회복이 향후 +3KB 추가에 fragile
  - **1-arg builtin arity guard 확장** (consistency, Cycle 6 또는 7)
- Structural Improvement Proposals:
  - **CI 게이트로 Fixed Point**: bootstrap_3stage.sh를 CI에 추가 (회귀 방지)
  - **O(n²) AST proper fix**: 별도 장기 트랙
  - **Token packing B안 (bit packing)**: 5M scale은 임시 — 장기 proper fix
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (정정은 Cycle 10)
- Next Recommendation: Cycle 6 = bootstrap.sh 32G 적용 + 1-arg builtin guard (compact 사이클)
