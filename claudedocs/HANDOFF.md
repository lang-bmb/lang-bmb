# BMB Session Handoff — 2026-05-10 (Cycles 2619-2633 — M4 언어 갭 + M5-1 payload enum)

> **HEAD**: 커밋 예정 (Cycle 2628 후 최종 커밋)
> **실무 앵커**: `claudedocs/ROADMAP.md`

---

## 0. 이번 세션 작업 (Cycles 2619-2633)

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
| 2627 | HANDOFF + ROADMAP 갱신 | — |
| 2628 | M4 최종 커밋 | 완료 |
| 2629-2632 | C# 바인딩 + PyPI 수정 | M4-6 C# 5/5 ✅, PyPI windows-2022 수정 |
| **2633** | **M5-1 payload enum 구현** | **`enum Option { None, Some(i64) }` + match ✅** |
| **2634** | **Rule 문서화 + OOM 분석** | **wildcard 지원 확인 + CLAUDE.md Rule 2/3 업데이트 ✅** |
| **2635** | **M5-2 Result enum 검증** | **`Result<Ok,Err>` + 3-variant + 체이닝 골든 테스트 3개 ✅** |
| **2636** | **HANDOFF + M5-3 설계** | **DESIGN-M5-3 설계 문서 + HANDOFF 갱신 ✅** |
| **2637** | **M5-3 다중 필드 enum** | **`Branch(i64,i64)` + `Three(i64,i64,i64)` 구현 ✅** |

---

## 1. 현재 상태

### 언어 갭 구현 현황

| 기능 | 이전 | 현재 |
|------|------|------|
| `let (a, b) = expr` | ❌ 미지원 | ✅ M4-3 구현 (Cycle 2621) |
| `Type::method(args)` | ❌ 미지원 | ✅ M4-4 구현 (Cycle 2620) |
| `Option::Some(x)` 표현식 | ❌ 미지원 | ✅ M5-1 구현 완료 (Cycle 2633) |

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
| M4 Adopted | 🔄 ~40% (M4-3 ✅, M4-4 ✅, M4-6 ✅, M4-5→M5-1 ✅, M4-1 미착수) |
| M5 Language Completeness | 🔄 M5-1 ✅ M5-2 ✅ M5-3 ✅ (Fixed Point 차단 — arena OOM pre-existing, O(n²) 문자열 AST) |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210 passed |
| `bootstrap` 골든 테스트 | ✅ 총 2840개 (M5: enum_payload, enum_wildcard, enum_result, enum_multi_payload, enum_chaining, enum_multi_field, enum_3field) |
| struct/enum 회귀 (Stage 1) | ✅ 8/8 PASS (enum_match, enum_variant, enum_payload, struct_complex, struct_method, nested_struct, mut_struct, struct_fn) |

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
| M5-1 | payload enum 구현 | 언어 아키텍처 | ✅ **완료** (Cycle 2633) |
| M5-2 | Result enum + 다중 payload | 언어 | ✅ **완료** (Cycle 2635, M5-1 인프라 재사용) |
| M5-3 | Multi-field enum `Branch(i64,i64)` | 언어 | ✅ **완료** (Cycle 2637) |
| M5-4 | String payload 타입 추론 + dead code 정리 | 언어/위생 | ⏳ 다음 2-3 cycles |

---

## 3. 다음 세션 우선순위

### 1순위 — M5-4 + enum 마무리

1. **`enum_payload_extract` / `resolve_payload_extracts` dead code 제거**: M5-3 이후 미사용
2. **M5-4**: String payload (`Some(String)`) 타입 추론 개선
3. **bootstrap arena OOM 근본 해결 방향**: 문자열 AST → 구조체 전환 (장기, 다음 메이저 아키텍처)

### 2순위 — PyPI publish + NuGet publish

3. **PyPI windows-2022 수정 push** → 재실행 트리거 (`.github/workflows/pypi-publish.yml` 수정 이미 커밋됨)
4. **NuGet publish**: 5개 C# 패키지 (M4-6 완료 후)

### 3순위 — M3 완료 (HUMAN 결정 필요)

5. **M3-3** npm publish + **M3-4** PyPI publish → **v0.100** 선언
6. **M4-1** B 공식 측정 (API key 필요)

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

## 5. HUMAN 결정 사항 (2026-05-10 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ **bmb-algo** (알고리즘·CPU bound → 성능 가설 직접 증명) |
| npm publish | ✅ **즉시 진행** — `workflow_dispatch` dry_run=false |
| PyPI publish | ✅ **즉시 진행** — `workflow_dispatch` publish=true, repository=pypi |
| v0.100 버전 선언 | ✅ **M3 publish 완료 직후** 메인테이너 결정 |
| B 공식 측정 | ✅ **즉시 실행** — `BMB_BENCH_API_KEY` 설정 후 `bmb-ai-bench run` |
| M5-1 하위 호환성 | ✅ **전체 마이그레이션** — unit enum도 `{i64, i64}` 로 통일 (이중 코드젠 경로 금지) |
| M5-1 LLVM 표현 | ✅ **고정 2-word** — `%EnumValue = type { i64, i64 }` (heap alloc 없음) |
| M5-1 가변 페이로드 | ✅ **M5-2로 defer** — M5-1 범위 = i64 단일 페이로드 + Option/패턴 매칭 |

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

---

## 8. M5-1 구현 핵심 사항

### 페이로드 enum LLVM 표현

```
heap calloc(2, 8) → 2-word struct:
  word 0: tag (i64)    — variant index (0-based)
  word 1: payload (i64) — 값 (unit variant = 0)
```

### 두 lowering 시스템 동시 처리 필수 (Rule 추가 예정)

| 시스템 | 위치 | 목적 |
|--------|------|------|
| recursive | `lower_expr_sb` | 표현식 내 중첩 eval |
| iterative | `step_expr` | 함수 body `let` 체인 |

신규 AST 노드 추가 시 **두 곳 모두** 수정 필수. (struct_init, lambda, enum_val 선례)

### bootstrap Arena OOM (pre-existing, 사이클 2237 이후)

`compiler.bmb` (~20K LOC) → Stage 2 빌드 시 arena 한계(16G) 초과.  
M5-1과 무관한 사전 존재 문제. Fixed Point 복원에 32G+ arena 또는 증분 컴파일 필요.

**세션 종료**: 2026-05-10 (Cycles 2619-2633 — M4 언어 갭 + M5-1 payload enum)
