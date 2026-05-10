# BMB Session Handoff — 2026-05-10 (Cycles 2619-2628 — M4 언어 갭 구현)

> **HEAD**: 커밋 예정 (Cycle 2628 후 최종 커밋)
> **실무 앵커**: `claudedocs/ROADMAP.md`

---

## 0. 이번 세션 작업 (Cycles 2619-2628)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2619 | 위생 + M4 이슈 등록 | 3개 이슈 등록, 기준선 확인 |
| 2620 | M4-4 Static Method Call | `Type::method(args)` 파서 구현 ✅ |
| 2621 | M4-3 Let-Tuple Destructuring | `let (a, b) = expr` 파서 구현 ✅ |
| 2622 | M4-5 스코프 분석 | payload enum = M5-1로 재분류 |
| 2623 | CLAUDE.md Rule 2 업데이트 | 지원 문법 목록 갱신, 사이드 이펙트 명시 |
| 2624 | 엣지 케이스 골든 테스트 | 2개 테스트 추가 (let-tuple-advanced, static-method-advanced) |
| 2625 | M5-1 아키텍처 설계 문서 | `DESIGN-M5-1-payload-enum.md` 작성 |
| 2626 | M4 통합 테스트 | `test_golden_m4_integration.bmb` → 42 |
| 2627 | HANDOFF + ROADMAP 갱신 | (현재) |
| 2628 | 최종 커밋 | 예정 |

---

## 1. 현재 상태

### 언어 갭 구현 현황

| 기능 | 이전 | 현재 |
|------|------|------|
| `let (a, b) = expr` | ❌ 미지원 | ✅ M4-3 구현 (Cycle 2621) |
| `Type::method(args)` | ❌ 미지원 | ✅ M4-4 구현 (Cycle 2620) |
| `Option::Some(x)` 표현식 | ❌ 미지원 | ❌ M5-1로 재분류 (7-12 cycles 필요) |

### Track 스냅샷

| Track | % | 상태 |
|-------|---|------|
| M (Machine-First) | ~100% ✅ | 완료 |
| N (MCP Server) | ~99% ✅ | 완료 |
| O (Context Pack) | ~95% ✅ | 완료 |
| Q (Ambiguity Audit) | ~92% ✅ | 완료 |
| R (LLM Bench) | ~95% ✅ | 완료 |
| S (BMB-rewrite) | ~99% ✅ | 완료 |
| T (External Bindings) | ~95% ✅ | 완료 |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated | ✅ COMPLETE |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~90% (showcase 선정+벤치+publish 잔여, HUMAN 결정) |
| M4 Adopted | 🔄 ~30% (M4-3 ✅, M4-4 ✅, M4-5→M5-1, M4-1/M4-6 미착수) |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo nextest run --release` | ✅ 6210 passed |
| `bootstrap` 골든 테스트 | ✅ 신규 4개 추가 (let_tuple, static_method_call, let_tuple_advanced, static_method_advanced, m4_integration) |

---

## 2. 태스크 목록

### M3 완료 태스크 (변경 없음 — HUMAN 결정 대기)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M3-1 | **[HUMAN]** showcase 선정: bmb-algo vs bmb-json | 결정 | 즉시 |
| M3-2 | showcase 공식 벤치마크 측정 (v0.98 기준) | 자율 | 1-2 cycles |
| M3-3 | **[HUMAN]** npm publish | 실행 | 즉시 |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | 즉시 |

### M4 태스크 상태

| # | 태스크 | 상태 |
|---|--------|------|
| M4-1 | **[HUMAN+KEY]** B 공식 측정 | ⏳ API key 필요 |
| M4-2 | 언어 갭 이슈 등록 | ✅ 완료 (Cycle 2619) |
| M4-3 | `let (a, b) = expr` tuple destructuring | ✅ 완료 (Cycle 2621) |
| M4-4 | `Type::method()` static method call | ✅ 완료 (Cycle 2620) |
| M4-5 | `Option::Some(x)` 표현식 | → **M5-1**로 재분류 |
| M4-6 | C# 바인딩 scaffold | ⏳ 미착수 |

### M5 준비 태스크 (신규)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M5-1 | payload enum 설계 및 구현 | 언어 아키텍처 | 7-12 cycles |
| M5-2 | (TBD) | - | - |

---

## 3. 다음 세션 우선순위

### 1순위 — M3 완료 (HUMAN 결정 필요)

1. **M3-1** showcase 선정 → **M3-2** 벤치마크 측정 (자율)
2. **M3-3** npm publish + **M3-4** PyPI publish

### 2순위 — M4 계속

3. **M4-6** C# 바인딩 scaffold (자율, 3-5 cycles)
4. **M4-1** B 공식 측정 (API key 필요)

### 3순위 — M5 준비

5. **M5-1** payload enum 구현 시작 (설계 문서: `claudedocs/issues/DESIGN-M5-1-payload-enum.md`)

---

## 4. M4-4 사이드 이펙트 주의사항

`Type::Variant(x)` 구문이 `(call <Type_Variant> x)` 로 파싱됨.

```bmb
// 현재 동작
fn Option_Some(v: i64) -> i64 = v;  // 자유 함수 필요
let x = Option::Some(42);           // → Option_Some(42) 호출

// M5-1 완성 후
let x = Option::Some(42);           // → enum_construct 노드
```

→ CLAUDE.md Rule 2에 이미 명시됨. M5-1 완성 전까지 payload enum constructor 사용 금지.

---

## 5. HUMAN 결정 대기 (변경 없음)

| 항목 | 현황 |
|------|------|
| M3 showcase 선정 | ⏳ bmb-algo(1순위) / bmb-json(2순위) |
| npm publish | ⏳ `workflow_dispatch` dry_run=false |
| PyPI publish | ⏳ `workflow_dispatch` publish=true, repository=pypi |
| v0.100 버전 선언 | ⏳ M3 완료 후 |
| B 공식 측정 | ⏳ `BMB_BENCH_API_KEY` 설정 필요 |
| M5-1 payload enum 설계 결정 | ⏳ unit enum 하위 호환성 + LLVM 표현 방식 |

---

## 6. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항

- **BMB_PATH 절대경로 필수**: `BMB_PATH=D:/data/lang-bmb/target/release/bmb.exe`
- **lsp.exe 재빌드**: `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- **verify_host.exe 재빌드**: `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- **BMB 소스 em-dash 금지**: U+2014 → ASCII 하이픈
- **캐시 파일**: `*.vh_cache`, `*.vh_proofdb` → `.gitignore` 등록됨

---

## 7. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `cargo nextest run --release` → 6210/6210 확인
- [ ] bootstrap 골든 테스트: `compiler.exe run test_golden_m4_integration.bmb` → 42
- [ ] `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- [ ] `python3 bootstrap/lsp_test.py` → 100/100 확인

---

**세션 종료**: 2026-05-10 (Cycles 2619-2628 — M4 언어 갭 구현)
