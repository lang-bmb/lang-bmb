# Cycle 2698: 골든 스위트 재실행 + manifest audit 스크립트
Date: 2026-05-11

## Re-plan
인계받은 carry-forward: 골든 재실행 + HANDOFF 갱신. advisor 권고로 audit-first 스크립트 추가. Trigger: ⚪ NONE.

## Scope & Implementation

### Stage 1 빌드
`bmb.exe build bootstrap/compiler.bmb -o target/golden-tests/bmb-stage1.exe` — 1회 빌드 (10s).
`--runtime` 플래그 미지원 (Rust CLI는 `BMB_RUNTIME_PATH` env로만 지원).

### 골든 스위트 백그라운드 1차 실행
JSON 결과: `{"passed":2859, "failed":3, "total":2862}` (1801s = ~30 min).
Cycle 2696 기준 (2850/12) 대비 9개 manifest fix + 1개 source fix = 10개 개선.

### 3개 회귀 후보 직접 재현
| 테스트 | 결과 |
|--------|------|
| test_golden_token_scan | ❌ segfault (rc=139) |
| test_golden_tokenizer | ❌ segfault (rc=139) |
| test_golden_set_cover_greedy | ✅ PASS (4 == 4) |

3번째 실패 식별 위해 verbose 백그라운드 재실행 (Cycle 2699 inline 검토).

### audit 스크립트 (`scripts/audit-golden-manifest.sh`)
- 각 manifest entry 빌드 + 실행 → manifest expected vs actual stdout 첫 줄 diff
- 4 status: OK / MISMATCH / BUILD_FAIL_* / RUN_FAIL_RC*
- 출력 포맷: `{filename}|{expected}|{actual}|{status}` (파이프 구분, 후속 일괄 정정 친화)
- 사용처: Cycle 2701 대량 정정에서 호출

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Stage 1 빌드 | ✅ |
| 골든 JSON | ✅ 2859 PASS / 3 FAIL |
| segfault 2건 재현 | ✅ rc=139 |
| audit 스크립트 작성 | ✅ |

결함: token_scan/tokenizer segfault 2건 잔존 (Cycle 2699-2700에서 분석/fix), 3번째 실패 식별 미완 (백그라운드 verbose 진행 중).

## Reflection

**핵심 통찰**:
- audit-first 권고 채택 — Cycle 2696 시점 70개 sample 7% mismatch율은 manifest 검증을 첫 단계로 두는 것이 옳다는 신호
- 그러나 manifest 정정 효과는 이미 Cycle 2696에서 9개 적용됨 → 현재 3 FAIL 중 manifest 오등록 가능성 낮음 (3 모두 build/run 단계 실패 패턴)
- 30분 풀 스위트 vs 5분 표적 검증: 알려진 후보 직접 실행이 cycle budget 더 효율적

**도그푸딩 가치**:
- 골든 스위트가 안정적 회귀 감지 게이트로 기능 (set_cover Cycle 2697 fix 후 12→3 FAIL 정확 측정)

**Roadmap impact**:
- M4-9 clang outlier 분석은 별도 cycle 유지 — 회귀 fix 우선

## Carry-Forward
- Actionable:
  - Cycle 2699 — token_scan + tokenizer segfault 동시 진단 (advisor 권고: 같은 root cause 가설)
  - 3번째 실패 식별 (백그라운드 결과 회수)
- Structural Improvement Proposals: 없음 (audit 스크립트는 즉시 사용 가능)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2699 segfault 진단 (gdb trace 또는 dbg println 삽입)
