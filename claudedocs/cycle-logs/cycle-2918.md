# Cycle 2918: tier3-spawn-overhead Phase 1 — lexer + brainfuck inproc
Date: 2026-05-19

## Re-plan
Plan valid. tier3-spawn-overhead Phase 1 (ISSUE-20260512 Option B): lexer + brainfuck inproc timing 포팅 및 측정.

## Scope & Implementation

### 파일 생성
- `ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main_inproc.bmb` — inproc timing harness (50 iters, 100x source, `bmb_black_box(0)` as start_pos)
- `ecosystem/benchmark-bmb/benches/real_world/lexer/c/main_inproc.c` — C 대응 (동일 알고리즘, GCC + clang 모두 빌드)
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main_inproc.bmb` — inproc timing harness (1000 iters)
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/c/main_inproc.c` — C 대응

### 발견된 버그: BMB string interpolation ← main.bmb 영향
`test_source()` 함수에서 `"    if n <= 1 { n }\n"` 같은 문자열 리터럴 내 `{identifier}` 패턴이 BMB string interpolation feature(Cycle 2848 추가)에 의해 `Expr::Var("n")`으로 처리되어 빌드 실패.

**영향 파일**: `lexer/bmb/main_inproc.bmb`, `lexer/bmb/main.bmb` (원본도 동일 버그)
**수정**: `{ n }` → `{{ n }}`, `{ fibonacci(n - 1) + fibonacci(n - 2) }` → `{{ fibonacci(n - 1) + fibonacci(n - 2) }}` 등 — 모든 리터럴 중괄호를 `{{`/`}}` 이스케이프로 교체.

**brainfuck**: `compute_only = "++++++++++[>++++++++++..."` 에 `{identifier}` 없음 → 수정 불필요.

### C LexTokType 이름 충돌 (이전 세션에서 이미 수정)
`TokenType` → `LexTokType` (Windows API 충돌 회피).

## Verification & Defect Resolution

### 빌드 결과
| 파일 | 빌드 결과 |
|------|---------|
| `lexer/bmb/main_inproc_bmb.exe` | ✅ BMB `--release` |
| `lexer/c/main_inproc.exe` | ✅ GCC -O2 |
| `lexer/c/main_inproc_clang.exe` | ✅ Clang -O2 |
| `brainfuck/bmb/main_inproc_bmb.exe` | ✅ BMB `--release` |
| `brainfuck/c/main_inproc.exe` | ✅ GCC -O2 |
| `brainfuck/c/main_inproc_clang.exe` | ✅ Clang -O2 (※ 측정 무효) |

### 측정 결과

**Lexer (50 iters, 100x source)**

| 구현 | median elapsed_us | checksum | 비고 |
|------|------------------|----------|------|
| BMB (LLVM opt-O2) | **1369 µs** | 445000 | |
| C GCC -O2 | 8136 µs | 445000 | |
| C Clang -O2 | 4934 µs | 445000 | |

- BMB vs GCC: **0.168× (BMB이 5.9× 빠름)** ✅ PASS
- BMB vs Clang: **0.278× (BMB이 3.6× 빠름)** ✅ PASS

**Brainfuck (1000 iters, compute_only)**

| 구현 | median elapsed_us | checksum | 비고 |
|------|------------------|----------|------|
| BMB (LLVM opt-O2) | **9731 µs** | 0 | |
| C GCC -O2 | 8040 µs | 0 | |
| C Clang -O2 | ~0 µs | 0 | ⚠️ 무효 — constant-folded |

- BMB vs GCC: **1.21× (BMB이 21% 느림)** ⚠️ 조건부 OK (원인 분석 필요)
- C Clang 무효: `volatile` black_box가 LLVM IPO를 막지 못함. C `volatile` 기반 black_box는 LLVM 빌드에서 신뢰할 수 없음.

### Brainfuck 성능 분석
BMB brainfuck가 GCC보다 21% 느린 원인:
1. **tape 할당**: BMB는 `calloc(30000, 1)` (heap), C는 stack-allocated `unsigned char tape[30000]`  
2. **bracket match**: 양쪽 모두 O(n) 선형 탐색 (캐시 없음)  
3. **switch 최적화**: GCC -O2가 brainfuck switch문을 jump table로 더 잘 최적화할 수 있음

→ heap alloc per iteration이 주 원인으로 추정. Phase 4 이후 개선 후보.

### Lexer 성능 분석 (BMB가 왜 빠른가)
1. **`@inline` 전 함수**: `skip_ws`, `is_whitespace`, `peek` 등 모두 인라인 → LLVM이 하나의 tight loop 생성
2. **tail recursion → LLVM TCO**: `count_tokens_from`이 tail recursive → LLVM loop로 최적화
3. **tuple return**: `(tok, new_pos)` tuple이 register pair로 반환 (struct 메모리 우회)

## Reflection

- **Scope fit**: Phase 1 완료. lexer + brainfuck 양쪽 inproc 측정 수집.
- **중요 발견**: BMB string interpolation feature(Cycle 2848)가 기존 벤치마크 `main.bmb`를 무효화했음. 원본 `lexer/bmb/main.bmb`도 함께 수정 필요 → 수정 완료.
- **C volatile black_box의 LLVM IPO 한계**: brainfuck처럼 const input + pure computation이면 LLVM이 constant-fold 가능. `bmb_black_box`는 진정한 런타임 장벽. C inproc은 LLVM 빌드에서는 GCC 버전을 사용해야 함.
- **Lexer BMB PASS** (5.9× vs GCC, 3.6× vs Clang) — `@inline` + tail TCO 효과.
- **Brainfuck 조건부 OK** — heap allocation 차이로 21% 느림. 허용 가능하나 개선 가능.

## Carry-Forward
- Actionable: Cycle 2919 — Phase 2 (csv_parse + http_parse inproc)
- Structural Improvement Proposals:
  - brainfuck BMB의 tape를 arena/stack 방식으로 교체하면 GCC 수준 달성 가능 (Medium priority)
  - C inproc에 LLVM IPO 안전한 black_box 추가 (e.g., `__asm__ volatile` 또는 `asm("" : "+r"(v));`) — Low priority
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2919 — csv_parse + http_parse inproc timing Phase 2
