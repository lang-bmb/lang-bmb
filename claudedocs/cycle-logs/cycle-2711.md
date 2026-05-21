# Cycle 2711: A안 5M scale 적용 → **Fixed Point 회복** 🎉
Date: 2026-05-11

## Re-plan
인계: Cycle 4 checkpoint — A안 적용 시도 결정.
Trigger 🟡 SCOPE ADJUST: Bounded 진단 끝났고 A안이 10 LOC 단순 변경이므로 사이클 1개로 시도 가능. advisor 권고 5M scale 채택, 게이트 4개 명시, 커밋 보류.

## Scope & Implementation

### Token packing 1M → 5M scale-up

`bootstrap/compiler.bmb` 변경 (11 사이트, 약 14 LOC):

**Token pack/unpack (4 LOC)**:
- L506: `tok_val(r) = r / 5000000` (was 1000000)
- L507: `tok_end(r) = r - (r / 5000000) * 5000000`
- L509: `make_tok(kind, endpos) = kind * 5000000 + endpos`
- L623: `tok_end(tok) - (tok_val(tok) / 5000000)` (dead-code 식이지만 일관성)

**pack_int_tok + scan_* (5 사이트, 각 임계+sentinel)**:
- L399-400: 임계 `9e12 → 1.8e12`, sentinel `9.2e12 → 1.84e12`, scale `1e6 → 5e6`
- L392 (scan_int): 임계+sentinel 동일 축소
- L407 (scan_hex_int): 동일
- L417 (scan_bin_int): 동일
- L428 (scan_oct_int): 동일

**Comment 갱신**:
- L204 "Token packing: kind * 5000000 + pos" (1000000 → 5000000)
- L388 "Cycle 2711 (M5-bootstrap): scale 1e6 -> 5e6 to support source > 1MB"

### 산술 정합성 검증

```
새 scale = 5_000_000
i64_max ≈ 9.22e18
max_kind = (9.22e18 - 5e6) / 5e6 ≈ 1.844e12
임계 1.8e12 → 한 자릿수 더 추가 후 1.84e13 미만 sentinel 사용 (saturate)
sentinel 1.84e12 * 5e6 = 9.2e18 < i64_max 9.22e18 ✅
```

사용자 정수 literal 한도: **1.84e12 (1.84 trillion)** — 컴파일러/도구 도메인 (ROADMAP § 1.4)에 충분.

### 변경 직접 영향 없는 1M 사이트 (보존)

- `pack_ids` (L3510-12): temp_id/block_id 분리 (별개 인코딩)
- `time_ns() / 1000000` (다수): 나노초→밀리초 변환 (도메인 무관)
- 행 12300 comment: line_start 인코딩 (별개)

## Verification & Defect Resolution

### 게이트 4개 + 회귀 검증

| 게이트 | 결과 | 시간/시그널 |
|--------|------|-----------|
| Gate 1: Stage 1 빌드 | ✅ | 12.1s (was 10.5s — 큰 차이 없음) |
| Gate 2: Simple BMB compile | ✅ | EXIT 0, valid IR |
| Gate 2b: integer literal compile | ✅ | EXIT 0 |
| **Gate 3: Full compiler.bmb (1.036MB)** | ✅ | **26.7s, 4.95MB output (114279 lines)** |
| **Gate 4: Stage 2 binary 생성 + Stage 3 IR + Fixed Point** | ✅ | **S2 == S3 (diff -q)** |

추가 회귀 검증:

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ **6210/6210 passed** (HANDOFF 수치 일치) |
| Sample golden 10/10 compile | ✅ 모두 EXIT 0 |
| Stage 2 binary 실행 (compiler.bmb 처리) | ✅ 26.1s, IR 동일 생성 |

결함: 없음. **Cycle 2237 회복** — Fixed Point 다시 PASS.

### 가설 정정 (Cycle 2709 → Cycle 2711)

| 가설 (Cycle 2709) | Cycle 2711 실측 |
|------------------|-----------------|
| Token packing overflow (>1MB) | ✅ 확정 — 단일 변경으로 해소 |
| O(n²) AST 메모리 폭발 (<1MB) | ❌ **무효** — token packing 정상화 후 OOM 발생 안 함. 16GB로 26.7s 정상 완료. |
| 두 결함 공존 | ❌ **무효** — token packing이 단일 차단 |

**올바른 모델**: Truncated 19500-line OOM은 token-packing 오작동 후 malformed token stream이 parser 재귀를 무한 루프/메모리 leak 트리거. Full compiler.bmb는 byte 1M 초과로 즉시 parse-1:3로 fail (재귀 안 들어감). 둘 다 동일 root cause.

## Reflection

### 외부 관찰자 관점

1. **단일 root cause의 강력함**: 10 LOC 변경 + 26초 stage 2 시간으로 부트스트랩 회귀 회복. Cycle 2237 시점 이후 한 자릿수 사이클 동안 차단되어 있었던 것이 단일 변경으로 해소.

2. **OOM 가설을 너무 빨리 받아들였음**: Cycle 2708에서 "OOM 우세" 결론을 내렸으나 사실 OOM은 token packing 오작동의 부작용. 한 자릿수 측정 데이터(Phase 3 byte threshold)가 가설 분리에 결정적. **bounded 진단의 가치 입증**.

3. **scope fit**: Cycle 4 checkpoint에서 A안 진입 결정이 옳았음. fix 시도가 진단보다 빠르게 결론 도출.

4. **advisor 권고 정확도**:
   - ✅ 5M scale (10M 대신) — i64 마진 안전
   - ✅ 게이트 4개 명시 — fix 검증 절차 결정
   - ❌ "O(n²) 트랙이 다음 차단" — 실측에서 무효, 부수적 가설이 었음
   - ✅ 커밋 보류 — Cycle 10 종합 commit 합리적

### Roadmap impact (대규모)

| 항목 | Before Cycle 2711 | After |
|------|-------------------|-------|
| Bootstrap Fixed Point | ❌ Cycle 2237 이후 회귀 | ✅ **회복** |
| M1 Self-Validated | ✅ COMPLETE | ✅ COMPLETE + 부트스트랩 검증 회복 |
| M5-1 OOM 노트 | "32G+ 초과, O(n²) AST" | **정정 필요**: "단일 token packing 결함, Cycle 2711 5M scale로 해소" |
| 다음 10 사이클 트랙 | Stage 2 진단 + O(n²) 트랙 | **남은 사이클에서 다른 자율 작업 가능** (Builtin arity, lint 확장, 측정 강화, ISSUE triage) |

### Sub-discovery

- bootstrap.sh default BMB_ARENA_MAX_SIZE=16G — 5M scale 후 Stage 2가 4-5GB 정도 사용 추정. 16G 여유. default 보존 가능.
- Stage 1 binary 빌드 시간 큰 변화 없음 (10.5s → 12.1s). 정수 literal max value 축소는 컴파일 시간에 무영향.

## Carry-Forward

- Actionable (Cycle 5 = 2712): **directional roadmap 재구성**
  - 원래 Cycle 5-7은 Stage 2 fix 또는 builtin arity. Stage 2 fix가 Cycle 4에서 완료된 셈 → builtin arity / lint 확장 / 측정 강화 / ISSUE triage로 전환
  - 남은 사이클 6개 (Cycle 5-10) 재배분 후보:
    - Cycle 5: HANDOFF/ROADMAP의 OOM 가설 + memory note 정정 (가벼움)
    - Cycle 6-7: Builtin arity proper fix (Cycle 2697/2700 source rename 회수)
    - Cycle 8: Tier 3 ≥10 runs 측정 강화 (knapsack outlier 재확인)
    - Cycle 9: ISSUE 40+ triage
    - Cycle 10: HANDOFF/ROADMAP 갱신 + 종합 commit
- Structural Improvement Proposals:
  - **Token packing proper fix (B안)**: 5M는 임시. 장기적으로 비트 분리 (`kind << 32 | pos`) 또는 별도 배열로 인코딩
  - **3-Stage 검증 CI 게이트**: Cycle 2711 회복은 다시 회귀 가능. CI에 부트스트랩 fixed point 추가 권고
- Pending Human Decisions: 변경 없음 (M3-3, M3-4, M3-5, M4-1 잠금)
- Roadmap Revisions: M5-1 표 행 OOM 노트 정정 (Cycle 10에서 일괄)
- Next Recommendation: Cycle 5 = HANDOFF/ROADMAP 정정 + 다음 자율 트랙 결정
