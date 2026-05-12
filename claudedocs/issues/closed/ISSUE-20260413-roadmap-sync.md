# ISSUE-20260413 — ROADMAP 벤치마크 결과 동기화

**우선순위**: P3
**영역**: docs
**상태**: **CLOSED (Cycle 2733 — v0.98 재측정으로 모든 claim resolved)**

## Resolution (Cycle 2733)

ISSUE의 3가지 불일치 claim 모두 v0.98 (2026-05) 데이터로 resolved:

| 원본 claim | 현재 상태 |
|-----------|----------|
| "0 FAIL" 주장이 8 FAIL 현실 모순 | ✅ 골든 2862/2862 PASS (Cycle 2701) — 0 FAIL 사실로 |
| "BMB > C AND Rust" 일부만 참 | ✅ M1 16/16 ≤1.05x PASS + P-track 6/6 ≤1.085x (Cycle 2725, 2732 양식 정규화) |
| "G-1 S2 ≠ S3" Fixed Point 미달 | ✅ S2 == S3 Fixed Point 회복 (Cycle 2711-2714, 재검증 Cycle 2718) |

BENCHMARK_REPORT.md (Jan 2026, v0.51.22) 자체는 stale로 잔존하나, 실무 측정 데이터는 `target/benchmarks/v098-historic.json` + `tier3-10runs.json`이 권위. `docs/ROADMAP.md`는 Cycle 2237 이후 Fixed Point 상태 정확히 반영.

**남은 작업** (별도 cycle):
- `ecosystem/benchmark-bmb/BENCHMARK_REPORT.md` 재생성 또는 stale 경고 추가 (별도 작업 — separate ISSUE 등록 불필요, ROADMAP § 4 M3-2 항목에 흡수)

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-01-25 (v0.51.22 BENCHMARK_REPORT) |
| `stale_after` | 2026-04-25 (이미 STALE — 3.5개월 경과) |
| `measurement_source` | `ecosystem/benchmark-bmb/BENCHMARK_REPORT.md` (v0.51.22, 1년 stale) |
| `observed_rate` | 8 FAIL (v0.51.22) — Cycle 2725에서 6/6 P-track 모두 ≤1.085x 재측정으로 **무효화 가능성 큼** |
| `scope` | 309 build benchmarks |
| `env_hash` | Jan 2026 환경 (구) |

**상태 권고**: Cycle 2731 또는 close 검토 — v0.98 측정 데이터로 ROADMAP 정합성 재확인 후 close.

## 문제

`dev-docs/ROADMAP.md`가 "309 빌드 ✅, 16+ FASTER, **0 FAIL**"라고 기재하나, `ecosystem/benchmark-bmb/BENCHMARK_REPORT.md` (v0.51.22, Jan 25, 2026)는 **8 FAIL** 상태를 보고.

### 불일치 항목

| 문서 | 주장 | 실제 |
|------|------|------|
| ROADMAP.md | 0 FAIL | 8 FAIL (v0.51.22) |
| ROADMAP.md | "BMB > C AND Rust" | 일부만 참. 53% FAIL |
| ROADMAP.md | G-1 "3-Stage Fixed Point (S2 == S3)" | S2 ≠ S3 IR (md5 diff) |

## 조치

1. ROADMAP.md 현재 상태 섹션 업데이트 — 실제 측정 반영
2. Graduation 진행도 G-1 100% → 조정 (Fixed Point 미달 반영)
3. "Next Focus" 섹션에 P0 이슈 3개 반영
4. BENCHMARK_REPORT.md 재생성 (최신 측정)

## 완료 기준

- ROADMAP, BENCHMARK_REPORT, HANDOFF.md 일관성
- 측정 근거가 있는 주장만 유지
