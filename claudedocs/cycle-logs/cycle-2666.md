# Cycle 2666: M5-5c 큰 변경 평가 + ROADMAP/HANDOFF 갱신 준비
Date: 2026-05-11

## Re-plan
Cycle 2665 carry-forward: M5-5c 구현 시도.
**검토 결과**: M5-5c (`fn() -> Array<String>` dispatch)는 program-level fn registry 인프라가 필요한 큰 변경.
- `collect_string_array_fns_from_mir` 신규 + 모든 codegen signature에 string_array_fns 인자 추가
- 또는 lowering 단계에 fn 시그니처 사전 수집 인프라 추가
- → 이는 명확한 "M6 type registry" 영역 — 1-2 cycles 안에 깔끔히 못 끝남

SCOPE ADJUST: M5-5c는 다음 세션 또는 별도 큰 사이클로 분리. 본 cycle은 종합 정리 + commit 준비로 전환.

## Scope & Implementation

### 1. M5-5c 구현 비용 분석

**옵션 A**: codegen 단계 dispatch (string_fns 패턴 미러)
- `collect_string_array_fns_from_mir` 신규 (collect_string_fns_acc 패턴 복제)
- `is_dynamic_string_array_fn` 신규
- `gen_program_*`, `gen_function_*`, `llvm_gen_call_*` 모두 string_array_fns 인자 추가 (signature change)
- 함수 호출 결과 처리 시 fn_name lookup → `push_str_ptr_marker(str_sb, dest)` 발행
- **변경 범위**: ~20개 함수 signature, +200 LOC, 회귀 가드 필요
- **난이도**: 중간 (패턴 복제, 인프라 안정적)

**옵션 B**: lowering 단계 사전 수집
- 함수 정의 lowering 진입 전에 fn_name → ret_type 매핑 사전 수집
- lower_call_sb에서 호출 결과에 자동 mark_str_ptr 발행
- **변경 범위**: 새 prepass 인프라 + lowering 단계 변경
- **난이도**: 높음 (새 컴파일 단계 추가)

**옵션 C**: 사용자가 명시 type annotation으로 우회
- `let arr: Array<String> = make_strs();` — type annotation으로 lowering 단계 정보 제공
- **변경 범위**: type annotation 처리 확장
- **난이도**: 중간 (기존 인프라 활용)

→ **추천**: 옵션 A (가장 깨끗) — 1-3 cycles 소요 예상

### 2. 본 cycle = 종합 정리 + commit 준비
- M5-5b 작동 ✅
- 골든 추가 ✅
- M5-5c 처방 명확화 ✅ (다음 사이클)
- M5-5d 미진단 — 동일 인프라로 처리 가능 (fn registry 대신 struct field registry)
- ROADMAP/HANDOFF 갱신 필요

### 3. 통합 commit 메시지 초안
```
feat(cycles 2660-2666): nqueen 측정 + in-process timing + M5-5b ✅

- Cycle 2660: nqueen 정식 측정 + clang -O3 baseline 5/5
- Cycle 2661: in-process time_ns() harness — wall-clock 폴딩 효과 분리
- Cycle 2662: M3-2-bench-results.md v2 (clang/gcc 모두 baseline)
- Cycle 2663: M5-5b 근본 진단 (val_type=="var" 발견)
- Cycle 2664: M5-5b 구현 ✅ — `mark_str_ptr_if` 새 MIR 명령어
- Cycle 2665: M5-5b 골든 추가 (2851/2851) + M5-5c 진단
- Cycle 2666: M5-5c 처방 명확화 + 종합 정리 (큰 인프라 — defer)
```

## Verification & Defect Resolution

**최종 상태 확인**:
- `cargo test --release`: 6210/6210 ✅
- 부트스트랩 Stage 1 빌드 ✅
- 골든 5/5 (M5-5 신규 5: arr_str_println/alias/for_loop/mut_set/var_repeat) ✅

## Reflection

**Scope fit**:
- 의도 = M5-5c 구현 → defer 결정 (큰 변경)
- 대신 종합 정리 + commit 준비 — 시간 활용 정직

**Latent defects**:
- M5-5c 미해결 — 다음 세션 작업
- M5-5d 미진단 — 다음 세션 작업

**Structural improvement opportunities**:
- M5-5c/d 동시 해결 = 다음 세션의 5-7 cycles
- in-process timing 인프라를 benchmark-bmb suite 전체에 적용 (HUMAN 결정 후)

**Philosophy drift 점검**:
- "복잡도는 기피 사유 아니다" — 단, **시간 한정 + 본 세션 범위 외**일 때 명시적 defer는 정당
- M5-5c 처방 = 옵션 A 결정 — 다음 세션에서 구현
- 정직한 평가 = "큰 변경" 인식 후 분리

**Roadmap impact**:
- M5-5 매트릭스 5/7 ✅ (var-repeat 추가)
- M5-5c, M5-5d 잔여 → 다음 세션
- M3 ~96% 유지 (publish HUMAN 잔여)
- M3-2 자율 100% ✅ (보고서 v2 + 7/7 측정)

**User-facing quality**:
- M5-5b 작동 = LLM이 자연 패턴 (`[s; N]`) 사용 가능 — AI-native 확장
- 골든 자동 검출 = 회귀 안정성

## Carry-Forward
- Actionable:
  - Cycle 2667: ROADMAP.md / HANDOFF.md 갱신 (M5-5b 완료 + M3-2 v2 반영)
  - Cycle 2668: 종합 commit (Cycles 2660-2667)
  - Cycle 2669: 세션 마무리 commit
- Structural Improvement Proposals (다음 세션):
  - M5-5c 옵션 A 구현 (`collect_string_array_fns_from_mir` + signature 확장)
  - M5-5d 진단 + 구현 (struct field type registry — 동일 패턴)
  - in-process timing benchmark-bmb suite 전체 적용
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 매트릭스 5/7 (var-repeat 추가) — 다음 사이클에서 ROADMAP 갱신
- Next Recommendation: Cycle 2667 — ROADMAP/HANDOFF 갱신
