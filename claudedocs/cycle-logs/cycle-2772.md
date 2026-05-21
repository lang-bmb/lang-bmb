# Cycle 2772: json_serialize char bug — P0 silent correctness bug 발견

Date: 2026-05-12

## Re-plan

진입 — cycle 2771 carry-forward (json_serialize bug fix, 추정 1 cycle light). Trigger ⚪ NONE.

## Scope & Implementation

### Step 1: source 추적

`benches/real_world/json_serialize/bmb/main.bmb`:
- `buf_char(buf, 0, 91)` 호출 — '[' (91) 쓰기 (정확한 ASCII 코드)
- 동일 buf를 serialize_person → serialize_int_array에 재사용 (buf[0..N])
- C 동등 코드 정상 작동 → BMB 측 codegen 문제

### Step 2: IR diagnosis

`--emit-ir` 결과:
```llvm
@buf_char (inlinehint):
  %sgep_base.0 = inttoptr i64 %pos to ptr           <- pos를 base로 선택 (잘못)
  %sgep_elem.0 = getelementptr inbounds i8, ptr %sgep_base.0, i64 %buf
  store i8 %store_u8_trunc.0, ptr %sgep_elem.0
```

**Smoking gun**: pos가 base로 선택됨. pos=0 시 `inttoptr 0 to ptr` = null. GEP from null = UB. LLVM이 store 제거.

### Step 3: Minimal repro

```bmb
@inline fn buf_char_a(buf: i64, pos: i64, c: i64) -> i64 = {
    store_u8(buf + pos, c);
    pos + 1
};

fn main() -> i64 = {
    let buf = calloc(20, 1);
    let _p1 = buf_char_a(buf, 0, 91);   -- pos=0 fails
    let _p2 = buf_char_a(buf, 1, 49);   -- writes? actually also fails
    let _p3 = buf_char_a(buf, 2, 93);
    store_u8(buf + 3, 0);
    puts_cstr(buf);  -- output: empty (expected "[1]")
};
```

→ 함수 인자 buf + pos 패턴이 silent UB 진입. pos=0 / pos>0 둘 다 영향.

### Step 4: Workaround 검증 (실패)

`let addr = buf + pos; store_u8(addr, c)` — 동일 UB 발생.
→ source-level workaround 불가.

### Step 5: ISSUE 등록 (P0)

`claudedocs/issues/ISSUE-20260512-store_u8-null-ptr-base.md`:
- **P0** (silent correctness — compile/run success + 잘못된 출력)
- estimated_cycles: 3-5 (hypothesis)
- Option A (proper fix), B (workaround 불가), C (source 분해)
- HUMAN 결정 필요 (Rule 6 vs P0)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| BMB IR diagnosis (`--emit-ir`) | ✅ inttoptr pos 잘못된 base 확인 |
| Minimal repro 작성 + 확인 | ✅ test_buf2.bmb 동일 패턴 재현 |
| Workaround 시도 (`let addr`) | ❌ 동일 bug, source-level fix 불가 |
| ISSUE-20260512-store_u8-null-ptr-base 등록 (P0) | ✅ |
| `cargo test --release` | ✅ |

**Defects**: P0 silent correctness bug 진단. fix는 multi-cycle (Rule 6 충돌 + 양쪽 backend parity 필요).

## Reflection

### advisor leverage

cycle 2769 verify 도구가 이번 cycle에서 P0 silent bug 식별로 이어짐. 도구 부재 시 이 bug는 측정 noise로 가장하여 지속될 가능성. **measurement integrity infrastructure** 효과 누적 검증.

### 외부 관점 — 6 dimensions

1. **Scope fit**: 의도 — light char bug fix. 실제 — root cause 진단 + ISSUE 등록 (cycle 2768 양식 강화 정합: 1 cycle 추정 → 실제 발견 cycle만 1, fix 3-5 cycle 별도).
2. **Latent defects**: **P0 store_u8 silent bug**. fix multi-cycle (Rule 6 충돌 → HUMAN 결정 필요).
3. **Structural improvement opportunities**:
   - bootstrap에 동일 패턴 점검 (store_u8 사용 위치 audit)
   - 골든 테스트 추가: `store_u8(arg_a + 0, c)` 패턴 (회귀 방지)
   - 더 적극적 verify (예: 매 cycle 종료 시 verify_bench_outputs 자동 실행)
4. **Philosophy drift**: 없음. workaround 회피 (Option B 작동 안 함). proper fix는 Rule 6 검토 필요.
5. **Roadmap impact**:
   - P0 bug 발견은 매우 중요 — measurement integrity 외에도 BMB 안전성 가설 영향
   - 잔여 cycles: HANDOFF/ROADMAP 갱신 (cycle 2773) + 종료 (cycle 2774)
6. **User-facing quality**: bench output level — silent diff에 대한 사용자 인지가 verify 도구로만 가능 (cycle 2769 도구 효과)

### P0 ranking 정당화

이 bug는 compile success + run success + 잘못된 출력 = silent. correctness 문제는 P-track gap (1.04x noise) 또는 token packing 등과 차원이 다름. **Other bench가 같은 패턴 사용 시 다 silent하게 잘못 동작**. P0 정합.

### 진단 cycle의 가치 누적

이번 세션 cycle estimate-vs-실측 갭 패턴 (cycles 2765/2766/2767) → 메타 fix (cycle 2768) → 도구 (cycle 2769-2771) → P0 bug (cycle 2772). 진단 cycle 우선 정책이 결과적으로 **P0 발견**으로 이어짐.

## Carry-Forward

### Actionable (다음 cycle)

**Cycle 2773**: HANDOFF/ROADMAP 사전 갱신:
- 10-cycle 결과 요약 (cycles 2765-2772)
- 신규 ISSUE 6개 등록 정리 (P0 1, P1 1, P2 1, P3 2, meta 1)
- claudedocs/ROADMAP.md `§ 6 ISSUE 양식 표준화` 갱신
- cycle-logs ROADMAP 갱신

### Structural Improvement Proposals

- **P0 store_u8 fix 처리**: HUMAN 결정 후 multi-cycle phase
- **bootstrap audit**: store_u8 사용 위치 점검 (다른 bench에서 silent bug 가능성)
- **bench output verification CI**: 다음 세션 first-real-run
- **FP tolerance epsilon arg**: `verify_bench_outputs.py` n_body 정상화
- **golden test 통합**: bench output을 정식 golden 자료로 (cycle 2769 carry-forward)

### Pending Human Decisions

- M3-3/M3-4 publish (HUMAN, 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 누적)
- Rule 6 / Rule 7 충돌 — **신규 차원**: P0 silent correctness bug (sorting rebuild + store_u8 null base + 비-correctness inline pass)는 Rust 동결 정책 검토 필요
- 6 신규 ISSUE 처리 시점 + 우선순위 ordering

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase D' (verify) 완료 (cycles 2769-2771)
- Phase E (sub-issue 처리) 시작 (cycle 2772 json_serialize → P0 발견)
- 잔여: cycle 2773 (HANDOFF/ROADMAP 갱신) + cycle 2774 (commit + 세션 종료)

claudedocs `ROADMAP.md`:
- M3 = ~99% (불변)
- M4 진행 ~50% (불변, M4-1 HUMAN 차단)
- **신규 § P0 store_u8 bug** 영속화 권고

### Next Recommendation

**Cycle 2773**: HANDOFF.md 전면 rewrite + ROADMAP.md § 6 sub-section 추가 + cycle-logs/ROADMAP.md 정리. commit 미실행.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| 신규 P0 ISSUE | `claudedocs/issues/ISSUE-20260512-store_u8-null-ptr-base.md` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2772.md` | gitignored |
