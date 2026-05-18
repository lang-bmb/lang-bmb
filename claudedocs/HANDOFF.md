# BMB Session Handoff — 2026-05-19 (Cycles 2939-2942 — 언어 갭 + 성능 최적화)

> **HEAD**: `797d7e3f` (Cycle 2942 완료)
> **이전 HEAD**: `2af17fb4` (Cycle 2927 완료)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930) — GCC MinGW 링커 비결정성으로 binary hash 비교 불가, IR hash 비교로 방법론 정정. bootstrap/compiler_s3.exe IR == compiler_s4.exe IR
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2943

---

## 이번 세션 작업 요약 (Cycles 2939-2942)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2939 | let (a,b) = expr Rust interpreter | `Expr::LetTuple` + `desugar_stmts` 탈당화 (9파일 unreachable!) + grammar.lalrpop BlockStmt 규칙 |
| 2940 | str_byte_at native + println dispatch | bmb_runtime.c + llvm_text.rs + mir/lower.rs MIR 타입 기반 dispatch |
| 2941 | csv_parse break-loop + http_parse @inline | csv 1.204×→1.057× + http 1.099×→0.947× (BMB faster) |
| 2942 | brainfuck @inline bracket+interpreter | brainfuck 1.274×→0.949× (BMB faster). 전체 7/7 real-world 6개 BMB faster |

### 성능 현황 (tier3 real-world inproc — Cycle 2942 기준)

| 벤치마크 | BMB (µs) | C GCC (µs) | 비율 | 이전 비율 |
|---------|----------|-----------|------|---------|
| brainfuck | ~7830 | ~8247 | **0.949×** ← BMB faster | 1.274× |
| csv_parse | ~3119 | ~2950 | **1.057×** | 1.204× |
| http_parse | ~2395 | ~2528 | **0.947×** ← BMB faster | 1.099× |
| lexer | ~1458 | ~8562 | **0.170×** ← BMB 5.9× faster | - |
| json_parse | ~2545 | ~3275 | **0.777×** ← BMB faster | - |
| json_serialize | ~494 | ~713 | **0.693×** ← BMB faster | - |
| sorting | ~502579 | ~3240793 | **0.155×** ← BMB 6.5× faster | - |

**7/7 real-world: 6개 BMB faster, 1개(csv_parse) 1.057× 이내**

### 핵심 변경 사항

**1. `let (a, b) = expr` Rust interpreter 지원 (Cycle 2939)**:
- `bmb/src/ast/expr.rs`: `Expr::LetTuple` + `desugar_stmts` 탈당화 로직
- `bmb/src/grammar.lalrpop`: BlockStmt 컨텍스트 tuple destructuring 규칙
- 9개 파일에 `unreachable!()` arm 추가
- `tests/golden/tuple_destructuring.bmb` 신규 (6출력 검증)
- LALR 충돌로 Expr 컨텍스트 미지원 (의도된 설계)

**2. native codegen 개선 (Cycle 2940)**:
- `bmb/runtime/bmb_runtime.c`: `bmb_str_byte_at` 함수 신규
- `bmb/src/codegen/llvm_text.rs`: str_byte_at declare + 매핑 + void list
- `bmb/src/mir/lower.rs`: println/print 인자 타입 기반 native dispatch
  `println(String)` → `println_str`, `println(f64)` → `println_f64`

**3. csv_parse v3 + http_parse @inline (Cycle 2941)**:
- csv_parse: in_quote 플래그 → break 기반 quoted loop (phi node 제거)
- http_parse: `@inline fn parse_http_flat` → 5× inlining → LLVM cross-function 최적화
- **패턴 확립**: LLVM 인라이닝 임계값 초과 함수 → `@inline`으로 명시적 강제

**4. brainfuck @inline (Cycle 2942)**:
- `@inline find_matching_close`, `@inline find_matching_open`, `@inline interpret_check`
- bracket 탐색 + 전체 interpreter 인라이닝
- 1.274× → 0.949×

### 테스트 변화
2388 tests, 0 FAILED (cargo test --release).

---

## 다음 사이클 (Cycle 2943)

- **Cycle 2943** (현재 세션): HANDOFF/ROADMAP 갱신 + CLAUDE.md @inline 패턴 문서화 + 추가 언어 갭 탐색
- **Cycle 2944** (현재 세션): 필요 시 추가 최적화 또는 cleanup

### 잔여 개선 가능 항목

| 항목 | 현재 | 개선 방법 |
|------|------|----------|
| csv_parse | 1.057× | calloc→memset 패턴 (native memset 필요) |
| brainfuck | 0.949× | 추가 최적화 가능하나 이미 BMB faster |
| inttoptr UB | P3 flakiness | 대규모 codegen 변경 (Option A, 5-10 cycles) |

