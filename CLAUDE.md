# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is BMB?

BMB (Bare-Metal-Banter) is an AI-native contract-verified systems programming language. Its core philosophy is **"Performance > Everything"**. Safety and correctness are not separate goals—they are natural consequences of pursuing maximum performance through compile-time proofs.

## Why BMB Exists

**핵심 질문**: "Runtime Overhead Zero를 달성하려면 언어는 어떤 모양이어야 하는가?"

### 기존 언어의 딜레마

```
Runtime Overhead ←――――――――――→ Developer Effort
      감소                        증가
```

Runtime Overhead Zero를 위해서는:
- 모든 타입 명시, 모든 Contract 작성, 수학적 증명 제공

**인간에게 불가능한 수준의 노력** → 모든 기존 언어는 "적정 수준의 런타임 오버헤드"를 수용

### AI가 바꾼 것

```
Before: Runtime Overhead ←→ Developer Effort (인간 한계)
After:  Runtime Overhead ←→ AI Effort (AI가 감당)
```

AI는 장황한 코드, 반복적 Contract 작성을 불평 없이 수행.

### 왜 기계어 직접 생성이 아닌가?

AI도 무한하지 않다:
- **Context Window**: 유한함
- **Hallucination**: 추상화 없이 디테일만 다루면 오류율 급증
- **Verification**: 생성 결과 검증 필요

**결론**: AI도 적절한 추상화 수준 필요

### BMB의 위치

> **BMB는 "AI가 효율적으로 작성할 수 있는 가장 낮은 추상화 수준"이다.**

- **더 낮추면**: Context 폭발, 검증 불가, Hallucination
- **더 높이면**: 런타임 오버헤드 발생

**이 포지션은 AI 이전에는 존재할 수 없었다.**

## Core Philosophy: Performance > Everything

### Why Single Goal?

```
Performance 추구
    → 런타임 체크 제거 필요
    → 컴파일타임 증명 필요
    → 증명된 코드는 자동으로 안전
```

**"Stability"는 독립적 목표가 아니다.** BMB의 모든 "안정성" 기능은 성능을 위해 존재하며, 안정성은 그 과정에서 자연스럽게 따라온다.

| 기능 | 존재 이유 | 안정성과의 관계 |
|------|----------|----------------|
| `pre idx < len` | bounds check **제거**를 위해 | 증명됐으니 안전한 것은 당연 |
| `T?` + contract | null check **제거**를 위해 | 증명됐으니 안전한 것은 당연 |
| `pure fn` | CSE, vectorization **활성화**를 위해 | side-effect 없으니 안전한 것은 당연 |
| explicit conversion | 숨겨진 **비용 방지**를 위해 | 명시적이니 안전한 것은 당연 |

### BMB vs Rust: 방향이 반대다

| 언어 | 1차 목표 | 방법 | 결과 |
|------|----------|------|------|
| Rust | Safety | Ownership + Borrow Checker | 성능도 좋음 |
| BMB | Performance | Compile-Time Proofs | 안전성도 보장됨 |

**BMB는 "안전한 언어"가 아니다. "빠른 언어"이고, 빠르기 위해 선택한 방법이 결과적으로 안전하다.**

---

## Workflow Rules (CRITICAL — Insights-Driven)

> 106 세션, 232시간 분석에서 도출된 반복 마찰 패턴 해소 규칙.

### Rule 1: 구현 우선, 계획은 최소화

```
❌ 세션의 80%를 분석/계획에 소비 → 구현 시간 부족 → 부분 완료
✅ 계획 5분 이내 → 즉시 구현 시작 → 테스트로 검증
```

- `plan and implement` 요청 시: 계획 단계는 빠르게 완료하고 **즉시 구현으로 전환**
- 이미 계획 파일이 존재하면 읽고 바로 구현 시작
- 사이클 실행 중에는 분석보다 **코드 작성 시간을 최대화**

### Rule 2: BMB 코드 작성 전 문법 검증 필수

BMB는 커스텀 언어이므로 일반 언어 지식으로 추측하면 안 된다.

```
❌ Rust/Python 문법을 BMB에 그대로 적용
✅ tests/ 디렉토리의 기존 테스트를 grep해서 실제 지원 문법 확인
```

**BMB bootstrap 컴파일러 지원 문법** (Cycle 2620-2646 추가):
- Tuple destructuring (`let (a, b) = expr`) ✅ — bootstrap: Cycle 2621 (양쪽 컨텍스트), Rust interp: Cycle 2939 (블록 컨텍스트만, 표현식 컨텍스트는 LALR 충돌로 미지원)
- Static method calls (`Type::method(args)`) ✅ — Cycle 2620에서 추가 (`Type_method(args)` 로 망글링)
- Payload enum constructor (`Option::Some(42)`) ✅ — Cycle 2633에서 추가 (heap calloc 2-word 표현)
- Payload enum match (`Option::Some(v) => v`) ✅ — Cycle 2633에서 추가 (tag 비교 + payload extract)
- Multi-field enum (`Node::Branch(20, 30)`, `Triple::Three(1,2,3)`) ✅ — Cycle 2637에서 추가
- Underscore wildcard (`_` in match) ✅ — 기존 지원 (오해 수정: Cycle 2634 확인)
- `println(String)` / `println(f64)` 자동 dispatch ✅ — Cycle 2640/2643 (str_sb 추적 기반)
- struct String 필드 (`p.name`) 자동 dispatch ✅ — Cycle 2645 (`~s` registry suffix)

**BMB가 지원하지 않는 문법** (반복 마찰 원인):
- Trait impl blocks
- Generic type parameters in bootstrap compiler (부분 지원)
- 함수 body 다중 statement는 `{...}` 블록 필수 (단일 식이면 `=` 표현식)
- Field assignment는 `set b.label = x` 형식 (`b.label = x` 미지원)

**주의 — M4-4 사이드 이펙트 (해소됨)**:
M5-1 완료로 `Option::Some(x)` 는 이제 정상 enum_val AST 노드로 처리됨.
구 workaround (`fn Type_Variant(v: T) -> EnumType = ...`) 는 제거 예정.

**주의 — M5-1 heap 표현**:
`enum Option { None, Some(i64) }` → 모든 variant가 `calloc(2, 8)` 2-word struct.
  word 0 = tag (i64), word 1 = payload (i64, unit variant = 0).
타입에 payload variant가 하나라도 있으면 unit variant도 heap 할당됨.

**파싱 경로 주의 — 블록 컨텍스트**:
`{...}` 내부 `let`은 `parse_block_let` (별도 코드 경로), 표현식 `let`은 `parse_let_expr`.
새 let 기능 추가 시 **두 곳 모두** 수정 필요 (Cycle 2621 발견).

**BMB 코드 작성 시 반드시**:
1. `tests/` 디렉토리에서 유사 테스트 2-3개를 grep으로 확인
2. 기존 통과 테스트에 사용된 문법만 사용
3. 새 문법이 필요하면 `grammar.lalrpop` 확인 후 파서 지원 여부 판단

### Rule 3: 부트스트랩 변경은 3-Stage 검증 필수

```
Stage 1 통과 ≠ Stage 2 통과 ≠ Stage 3 통과
```

부트스트랩 컴파일러(`bootstrap/`) 변경 시:
1. **Stage 1**: Rust 컴파일러로 `compiler.bmb` 빌드 → 골든 테스트
2. **Stage 2**: Stage 1 바이너리로 `compiler.bmb` 컴파일 → Stage 2 IR 생성
3. **Stage 3**: Stage 2 바이너리로 `compiler.bmb` 컴파일 → **Fixed Point (S2 IR == S3 IR) 검증**

> **Fixed Point 주의** (Cycle 2930 정정): GCC MinGW-w64 링커는 동일 소스라도 빌드 시마다 binary가 달라짐 (비결정적 `.exe`). **Binary hash가 아닌 IR hash**로 비교해야 함.
> 올바른 검증: `compiler.exe build compiler.bmb --emit-ir -o s3.ll` + `compiler.exe build compiler.bmb --emit-ir -o s4.ll` → `diff s3.ll s4.ll` (0 diff = Fixed Point 달성).

**알려진 부트스트랩 실패 패턴**:
| 패턴 | 증상 | 원인 |
|------|------|------|
| O0 통과, O2 세그폴트 | opt -O2에서만 크래시 | LLVM 최적화 상호작용 (select+alias analysis 등) |
| Stage 1 통과, Stage 2 스택오버플로 | 재귀 깊이 초과 | 부트스트랩 파서의 재귀 제한 |
| TK_INT 토큰 충돌 | 잘못된 파싱 | 정수 리터럴 토큰과 식별자 충돌 |
| stale runtime object | 링크 에러 | bmb_runtime.c/bmb_event_loop.c 재빌드 필요 |
| Stage 2 arena OOM (32G+) | compiler.bmb self-compile 시 메모리 한계 초과 | 문자열 기반 AST의 O(n²) 성장 (Cycle 2634 확인) |

**이중 Lowering 시스템 (신규 노드 추가 시 필수 — Cycle 2634)**:
`bootstrap/compiler.bmb`에는 두 개의 독립 lowering 경로가 있다:
- **recursive**: `lower_expr_sb` (5464 근처) — 중첩 표현식 평가
- **iterative**: `step_expr` (3811 근처) — 함수 body `let` 체인

신규 AST 노드 추가 시 **두 경로 모두** 반드시 처리해야 한다.
누락 시 `%_t-1` LLVM 심볼 생성 등 빌드 실패.
선례: `struct_init`, `lambda`, `enum_val` 모두 양쪽 처리.

```
❌ lower_expr_sb에만 추가 → step_expr에서 fall-through → %_t-1 생성
✅ lower_expr_sb + step_expr 모두 추가 (step_expr는 lower_*_sb 함수로 위임)
```

### Rule 4: 벤치마크 방법론 엄격 준수

```
❌ BMB: debug 빌드 vs C: -O0 비교 → 의미 없는 결과
✅ BMB: --release + opt -O2 vs C: -O2 -march=native 비교
```

| 항목 | 필수 조건 |
|------|----------|
| BMB 빌드 | `--release` 플래그 + `opt -O2` 이상 |
| C 베이스라인 | `-O2` 또는 `-O3 -march=native` |
| Rust 베이스라인 | `--release` (최소 O2) |
| 비교 대상 | 동일 알고리즘, 동일 입력 |
| 최적화 미적용 비교 금지 | 절대 수행하지 않음 |

### Rule 5: AST/MIR 변형 추가 시 전수 검색

새 AST 노드나 MIR 명령어 추가 시:
1. **먼저** `grep`으로 해당 enum의 **모든 match arm** 위치 검색
2. 누락 없이 **한 번에** 전체 업데이트
3. 특히: `parser/`, `ast/`, `types/`, `mir/`, `codegen/`, `interp/`, `bootstrap/`

```
❌ 하나씩 수정하다 누락 → 반복 컴파일 에러
✅ grep "Expr::" bmb/src/ -rn → 모든 위치 파악 → 일괄 수정
```

### Rule 6: BMB 중심 개발 (Rust 졸업 정책)

v0.94부터 Rust 컴파일러는 동결 상태. **모든 새 기능은 BMB(compiler.bmb)에서 직접 구현.**

```
❌ bmb/src/*.rs 수정 → cargo test → bootstrap에 포팅
✅ bootstrap/compiler.bmb 직접 수정 → Stage 1 빌드 → 3-Stage 검증
```

| 항목 | 정책 |
|------|------|
| Rust 새 기능 추가 | ❌ 금지 |
| Rust 테스트 추가 | ❌ 금지 (새 테스트는 BMB 골든 테스트) |
| Rust 버그 수정 | ⚠️ 부트스트래핑 차단 시에만 |
| Rust P0 정확성 버그 수정 | ✅ 예외 허용 — 단, 최소 패치 원칙 엄수 (아래 참고) |
| `cargo test --release` | 🔧 회귀 방지 목적으로만 유지 |

**P0 예외 조항** (D3, Cycle 2781):
Rust 코드젠에서 발생하는 **P0 정확성 버그** (잘못된 IR 생성으로 런타임 오류 발생)는 최소 패치로 수정할 수 있다.

적용 조건:
1. **P0만 해당** — 성능 문제, 기능 요청, 리팩토링은 포함하지 않음
2. **최소 패치 원칙** — 해당 버그만 수정, 주변 코드 정리 금지
3. **BMB 포팅 불필요한 경우** — bootstrap/compiler.bmb가 해당 코드를 대체하기 전까지만 유효
4. **확인 필수** — `cargo test --release` + 관련 벤치마크 회귀 없음

예시 (Cycle 2776 D1): `llvm_text.rs`의 `param_set` 휴리스틱이 GEP 인트린식 인수를 잘못 할당 → 벤치마크 출력 오류. 최소 패치(6줄)로 수정, 정상 동작 복원.

### Rule 7: 코드젠 백엔드 구분

BMB에는 두 가지 LLVM 코드젠 백엔드가 있다:
- **inkwell backend** (`codegen/llvm.rs`): LLVM C API 바인딩 (`--features llvm` 사용 시)
- **text backend** (`codegen/llvm_text.rs`): 텍스트 LLVM IR 생성 (`.ll` 파일, 기본)

변경 시 **양쪽 백엔드에 동일하게 적용**해야 한다. 두 백엔드가 서로 다른 IR을 방출하면 미묘한 런타임 차이가 발생한다 (예: Cycle 362에서 발견된 inkwell `add_inline_main` argc 초기화 누락).
부트스트랩 컴파일러(`bootstrap/compiler.bmb`)는 text backend만 사용.

### Rule 8: 출력 디폴트 = AI 친화 구조화

모든 컴파일러 출력의 **디폴트는 머신/AI 친화 구조화 형식** (JSON 등). 인간 친화 표시는 **명시 옵션 (`--human`)**.

```
❌ 새 명령에 인간 친화 출력만 구현 (컬러 ANSI, ASCII art 등)
✅ 디폴트 = compact JSON / 머신 파싱 가능 / 스키마 명시;
   `--human` 플래그로 컬러/포매팅 분기
```

근거 (Vision v1.0 realignment, Cycle 2507):
- 1차 사용자 = 인간+AI 협업 (LLM이 BMB 작성, AI agent가 출력 파싱)
- 기존 정렬:
  - `bmb/src/main.rs` `HUMAN_OUTPUT` AtomicBool, default false
  - `bmb::error::report_error_machine`, `report_warnings_machine`
  - LSP queries `--format json|compact|llm`

**적용 시 점검**:
1. 새 명령 추가 시 `is_human_output()` 분기 도입
2. JSON 출력은 안정 스키마 (필드 명·타입 변경은 호환성 분석 후)
3. 인간 모드는 머신 모드의 superset 아님 — 두 표현은 등가성 유지

---

## Development Principles (CRITICAL)

**프로그래밍 언어 프로젝트의 특수성을 항상 감안할 것.**

### Principle 1: 성능 저하는 버그다

성능 문제는 기능 버그와 **동일한 심각도**로 취급한다.

```
❌ "일단 동작하게 만들고 나중에 최적화"
❌ "premature optimization is the root of all evil"
✅ "처음부터 최적의 코드를 생성하도록 설계"
```

**Rationale:** BMB의 존재 이유가 성능이다. 성능이 나쁜 BMB는 존재 가치가 없다.

### Principle 2: Workaround는 존재하지 않는다

프로그래밍 언어에서 workaround는 기술부채가 아니라 **결함**이다.

| 상황 | 잘못된 대응 | 올바른 대응 |
|------|------------|------------|
| 재귀가 루프보다 느림 | `@inline` 힌트 추가 | 언어에 루프 구문 추가 |
| 특정 패턴 최적화 실패 | 벤치마크 코드 수정 | MIR 최적화 패스 추가 |
| LLVM이 최적화 못함 | "LLVM 한계" 결론 | 더 나은 IR 생성 |

| 일반 프로젝트 | BMB (언어 프로젝트) |
|--------------|-------------------|
| 빠른 해결책 우선 | 근본적 원인 해결 우선 |
| Workaround 허용 | Workaround 금지 |
| 리팩토링 나중에 | 지금 당장 올바르게 |
| 스펙 변경 회피 | 필요시 스펙 변경 감수 |

**Rationale:** BMB 위에서 수천 개의 프로그램이 작성된다. 컴파일러의 workaround는 모든 프로그램에 전파된다.

### Principle 3: 복잡도는 기피 사유가 아니다

```
❌ "루프 추가는 파서, AST, 타입체커, MIR, 코드젠, 부트스트랩
    전부 수정해야 하니까 alwaysinline으로 대체하자"

✅ "루프 추가가 근본 해결이다.
    파서 → AST → 타입체커 → MIR → 코드젠 → 부트스트랩 순서로 진행"
```

작업량이 크다는 것은 하지 않을 이유가 아니다. 근본 해결이 필요하면 실행한다.

**Rationale:** 쉬운 해결책은 대부분 workaround다. 어려운 해결책이 proper fix인 경우가 많다.

### Principle 4: "언어 한계"는 답이 아니다

BMB는 우리가 만드는 언어다. 한계가 있으면 **언어를 바꾼다**.

```
❌ "이것은 BMB 언어 설계의 한계입니다"
✅ "이것은 언어 스펙 변경이 필요한 영역입니다"
```

**Rationale:** "언어 한계"라고 결론내리는 순간 개선이 멈춘다. 우리는 언어를 만드는 사람들이다.

---

## Decision Framework

문제 발견 시 **반드시 위에서부터** 검토:

| 순위 | 수준 | 검토 질문 | 예시 |
|------|------|----------|------|
| 1 | **언어 스펙** | 이 기능이 언어에 있어야 하는가? | while 루프, 패턴 매칭 |
| 2 | **컴파일러 구조** | MIR/AST가 이를 표현할 수 있는가? | `is_tail` 필드 추가 |
| 3 | **최적화 패스** | 최적화 패스가 이를 처리하는가? | TCO, LICM 패스 |
| 4 | **코드 생성** | 생성되는 IR이 최적인가? | `musttail` 방출 |
| 5 | **런타임** | 런타임이 지원해야 하는가? | GC, 예외 처리 |

**낮은 수준에서 해결하려는 유혹을 경계하라.** 1번에서 해결할 문제를 4번에서 해결하면 그것은 workaround다.

### 마일스톤 vs 버전 분리 (2026-05-01 vision realignment)

기술 마일스톤과 메이저 버전 선언은 **다른 차원**이다. 작업 분류 시 두 차원을 섞지 않는다.

| 차원 | 결정 권한 | 게이트 |
|------|---------|------|
| 기술 마일스톤 (M1~M4) | 자율 (자가 검증) | 내부 binary 조건만 |
| 릴리스 버전 (v0.x → v1.0) | **비자율** (메인테이너 + 커뮤니티) | **외부 신호 필수** |

마일스톤·버전 매핑 + 외부 신호 임계값은 `docs/ROADMAP.md` § "Vision v1.0 Framework" 참조.

**의사결정 시**:
- 자율 사이클 작업 = M1~M4 진척으로 분류 (B/P/A/D/C 우선순위)
- v1.0 선언/메이저 점프는 사이클 안건이 아님 (외부 신호 충족 시 별도 결정)
- 옛 Track A/B/C/D 등 표기는 ROADMAP § "Track migration" 표를 따라 새 분류로 매핑

---

## Verification Principle

모든 성능 주장은 **측정으로 증명**한다.

```
주장: "Contract가 최적화를 가능하게 한다"
증명: Contract 유무에 따른 어셈블리 비교

주장: "BMB는 C와 동등한 성능"
증명: 동일 알고리즘의 벤치마크 비교
```

측정 없는 성능 주장은 허용하지 않는다.

---

## 개발 사이클 (Cycle Development Mode)

BMB 개발은 **사이클 기반 반복 개발**을 따른다. 각 사이클은 명확한 범위와 산출물을 갖는다.

### 사이클 프로세스

```
┌─────────────────────────────────────────────────────────────┐
│  1. 개발 범위 설정                                          │
│     - 프로젝트 철학과 정렬 (Performance > Everything)       │
│     - 명확한 목표와 완료 조건 정의                          │
│     - 로드맵/이슈와 연결                                    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  2. 개발 구현                                               │
│     - Decision Framework 준수 (언어 스펙 → 코드젠 순서)     │
│     - Workaround 금지, 근본 해결 우선                       │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  3. 테스트 (단위/통합)                                      │
│     - cargo test --release                                  │
│     - 부트스트랩 검증 (Stage 1 최소)                        │
│     - 벤치마크 회귀 확인                                    │
└─────────────────────────────────────────────────────────────┘
                              ↓
                    ┌─────────────────┐
                    │ 2~3 반복        │
                    │ (테스트 통과까지)│
                    └────────┬────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  4. 평가                                                    │
│     - 미비점 (incomplete): 구현되지 않은 부분               │
│     - 결함 (defect): 버그, 성능 문제                        │
│     - 개선점 (improvement): 더 나은 방법 발견               │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  5. 후속 단계 도출                                          │
│     - 다음 사이클 범위 정의                                 │
│     - 이슈 등록 (claudedocs/issues/)                        │
│     - 로드맵 업데이트                                       │
└─────────────────────────────────────────────────────────────┘
```

### 사이클 완료 체크리스트

| 항목 | 필수 | 설명 |
|------|------|------|
| 테스트 통과 | ✅ | `cargo test --release` (절대 debug 빌드로 테스트하지 않음) |
| 부트스트랩 성공 | ✅ | 최소 Stage 1 + 골든 테스트; 부트스트랩 변경 시 Fixed Point 필수 |
| 성능 회귀 없음 | ✅ | Tier 1 벤치마크 ≤ 2% |
| 미비/결함/개선 문서화 | ✅ | 발견 사항 기록 |
| 후속 작업 정의 | ✅ | 다음 단계 명확화 |
| BMB 문법 검증 | ✅ | 새 BMB 코드 작성 시 기존 테스트 참조 확인 (Rule 2) |

### 실패 시 자가 진단 절차

테스트 실패 시 "Known Failure Patterns" 섹션을 **먼저** 확인:
1. `--release` 누락인가?
2. 잘못된 codegen 백엔드(inkwell vs text)인가?
3. BMB 미지원 문법 사용인가?
4. runtime 재빌드 필요한가?
5. 위 패턴에 해당하지 않으면 근본 원인 분석

### 사이클 산출물 템플릿

```markdown
## Cycle N 완료: [사이클 제목]

### 개발 범위
- [이번 사이클에서 수행한 작업]

### 현재 상태
- 테스트: ✅/❌
- 부트스트랩: ✅/❌
- 벤치마크: ✅/❌ (성능 변화 요약)

### 미비/결함/개선 도출
| 유형 | 내용 | 심각도 |
|------|------|--------|
| 미비 | ... | ... |
| 결함 | ... | ... |
| 개선 | ... | ... |

### 후속 단계
1. [다음 작업 1]
2. [다음 작업 2]
```

---

## Benchmark Cycle Guidelines (부트스트래핑 + 벤치마크 주도개발)

BMB는 계약/형식검증을 언어스펙에 포함해 **제로 오버헤드**를 달성, 이론상 가장 빠른 프로그래밍 언어가 되어야 한다.

### 목표

**C/Rust를 추월**하는 것이 목표다. 벤치마크에서 성능 타협은 없으며, 필요시 **언어 스펙 변경/컴파일러 최적화**를 포함한다.

### 성능 판정 기준

| 결과 | 판정 | 조건 |
|------|------|------|
| BMB > C/Rust | ✅ PASS | 목표 달성 |
| BMB ≈ Clang/Rust | ✅ OK | **동일 LLVM 백엔드** 사용, IR 동등성 확인 필수 |
| BMB ≈ GCC | ⚠️ 조건부 OK | LLVM 한계 증명 필요 (IR 최적화 확인) |
| BMB < Clang/Rust | ❌ FAIL | 원인 분석 + 해결 방안 제시 필수 |

### 동등/근접(OK) 인정 조건

1. **LLVM 한계인 경우**
   - BMB IR이 최적화되어 있음이 확인됨
   - 같은 백엔드(Clang, Rust)와는 동등/추월 달성
   - GCC 대비 차이는 LLVM 한계로 인정 가능

2. **C/Rust가 이미 오버헤드 제로인 경우**
   - 동등 IR 생성이 확인되어야 함
   - 런타임에 제거할 오버헤드가 없음이 증명됨

### @inline 전략 (Cycles 2941-2942 확립)

LLVM은 큰 함수를 자동 인라이닝 대상에서 제외한다. 반복 호출되는 핫 함수가 임계값을 초과하면:

```
증상: BMB가 C보다 느림, 어셈블리에서 callq 반복 확인
원인: 함수가 LLVM 인라이닝 임계값 초과 (통상 ~100 instructions)
수정: @inline fn my_func(...) → alwaysinline attribute → LLVM 강제 인라이닝
결과: cross-function 최적화 (loop invariant hoisting, constant propagation) 활성화
```

**적용 조건**:
- 반복 호출되는 핫 함수 (N × call overhead ≫ code size 증가)
- 인라이닝 후 LLVM이 추가 최적화 가능한 경우 (상수 인수, 반복문 fusion 등)

**사례 (Cycle 2941-2942)**:
- http_parse `parse_http_flat` @inline → 5× inlining → 1.099×→0.947×
- brainfuck `find_matching*` + `interpret_check` @inline → 1.274×→0.949×

**역효과 조건 (Cycle 2944 확인)**: 대형 독립 루프 함수(~200-line IR 이상)에 @inline 시
코드 블로트 + 명령어캐시 압박으로 오히려 회귀 가능. csv_parse: @inline → +17% 회귀.
적용 전 IR 크기 확인 필수: `--emit-ir` + `opt -O2` 후 함수 라인 수 측정.

**주의**: `@inline`은 컴파일러 결함의 workaround가 아닌 **사용자 최적화 지시자**다.
컴파일러 결함 (TCO 미적용, 불필요한 spill 등)은 별도 수정.

### BMB가 느린 경우 필수 액션

```
1. 원인 분석
   - IR 비교 (BMB vs C/Rust)
   - 어셈블리 비교
   - 병목 지점 식별

2. 해결 방안 결정
   ┌─────────────────────────────────────────┐
   │ 언어 스펙 변경이 필요한가?              │
   │   → 새 구문/기능 추가 검토              │
   │                                         │
   │ 컴파일러 최적화가 필요한가?             │
   │   → MIR 패스/코드젠 개선 검토           │
   │                                         │
   │ LLVM 활용이 부족한가?                   │
   │   → 더 나은 IR 생성 방법 검토           │
   │                                         │
   │ 핫 함수 call overhead가 원인인가?       │
   │   → @inline으로 강제 인라이닝 시도      │
   └─────────────────────────────────────────┘

3. 구현 또는 이슈 등록
   - 즉시 해결 가능하면 구현
   - 대규모 변경이면 이슈로 등록 후 추적
```

### 사이클 완료 시 필수 출력

```
## Cycle N 완료

### 현재 상태
- 벤치마크 결과 요약
- 변경 사항

### 성능 판정
- 각 벤치마크의 OK/FAIL 판정
- FAIL인 경우 원인 분석

### 미비/결함/개선 도출
- 발견된 문제점
- 필요한 언어 스펙/컴파일러 변경

### 다음 진행 작업
- 구체적 액션 아이템
```

---

## Build Commands

```bash
# Build the compiler (release mode)
cargo build --release

# Build with LLVM support (required for native compilation)
# On Windows with MSYS2/MinGW LLVM, use MinGW target to avoid header conflicts:
cargo build --release --features llvm --target x86_64-pc-windows-gnu

# On Linux/macOS:
cargo build --release --features llvm

# Run all tests
cargo test --release

# Run Clippy linter
cargo clippy --all-targets -- -D warnings
```

## Build Environment (Windows)

### Required Tools

| Tool | Version | Path | Purpose |
|------|---------|------|---------|
| LLVM | 21.1.8 | C:/msys64/ucrt64/bin | IR optimization, codegen |
| GCC | MinGW-w64 | C:/msys64/ucrt64/bin | Linking |
| Rust | stable | ~/.rustup | Compiler build |

### LLVM Tools Usage

The compiler uses external LLVM tools for optimization:

```
BMB Source → MIR → LLVM IR (inkwell) → opt -O2 → llc/inkwell → Object → gcc → Executable
```

| Tool | Status | Usage |
|------|--------|-------|
| `opt` | Required | IR-level optimization (`-O2`, `-O3`) |
| `llc` | Fallback | Codegen-level optimization (when opt unavailable) |
| `clang` | Optional | Alternative compiler driver |

### Environment Variables

```bash
# Required: Path to BMB runtime library
export BMB_RUNTIME_PATH="d:/data/lang-bmb/bmb/runtime"

# Optional: LLVM installation prefix (auto-detected from PATH)
# export LLVM_SYS_211_PREFIX="C:/msys64/ucrt64"
```

### Optimization Strategy

The compiler tries optimization methods in order:

1. **opt -O2/O3** (preferred) - Full LLVM optimization pipeline
2. **llc -O3** (fallback) - Codegen-level optimization only

**Note:** Some benchmarks (e.g., mandelbrot) perform better with llc-only due to opt transformation interactions.

## Using the Compiler

```bash
# Type check a BMB file
./target/release/bmb check <file>.bmb

# Run with tree-walking interpreter
./target/release/bmb run <file>.bmb

# Compile to native executable (requires --features llvm)
./target/release/bmb build <file>.bmb -o output

# Compile to LLVM IR
./target/release/bmb build <file>.bmb --emit-ir -o output.ll

# Format BMB source files
./target/release/bmb fmt <file>.bmb

# Lint BMB source files
./target/release/bmb lint <file>.bmb

# Contract verification with Z3
./target/release/bmb verify <file>.bmb

# Run microbenchmarks (@bench attribute)
./target/release/bmb bench <file>.bmb                # interpreter (default)
./target/release/bmb bench <file>.bmb --native       # native compile + run (v0.98)
```

Note on `--native`: synthesizes a harness per file, compiles via the standard
pipeline, runs it, and parses stdout into per-bench statistics. Uses
`bmb_black_box` to defeat DCE, but LLVM can still constant-fold pure bench
bodies — use `time_ns()`-seeded or memory-touching workloads to measure
meaningfully. See `docs/BENCHMARK.md` for details.

## Running Tests

```bash
# All tests
cargo test --release

# Specific module tests
cargo test parser::tests
cargo test types::tests

# Verbose output
cargo test -- --nocapture --test-threads=1
```

## 부트스트래핑 + 벤치마크 주도 개발 사이클

BMB 개발의 핵심 프로세스입니다. 부트스트래핑과 벤치마크를 통합하여:
1. **컴파일러 정확성** 보장 (부트스트랩 검증)
2. **성능 회귀 방지** + **지속적 성능 최적화** (벤치마크)

### 사이클 실행

```bash
# Quick check (빠른 검증, ~2분)
./scripts/quick-check.sh              # Tests + Stage 1 bootstrap + Tier 0 benchmarks

# Full verification (PR 전, ~15분)
./scripts/full-cycle.sh               # Full 3-stage bootstrap + all benchmarks

# Individual scripts
./scripts/bootstrap.sh                # 3-stage bootstrap verification
./scripts/benchmark.sh --tier 1       # Run Tier 1 benchmarks
python3 scripts/compare.py a.json b.json  # Compare benchmark results
```

### 사이클 산출물

사이클 완료 시 반드시 제시해야 하는 항목:

| 항목 | 내용 |
|------|------|
| **현재 상태** | 테스트/부트스트랩/벤치마크 통과 여부 |
| **미비/결함/개선점** | 발견된 이슈와 근본 원인 분석 |
| **다음 진행 작업** | 우선순위별 작업 목록 |

### 성공 조건

| 항목 | 조건 | 블로킹 |
|------|------|--------|
| 테스트 | 모든 테스트 통과 | ✅ Yes |
| 부트스트랩 | Stage 1 성공 (Quick) / Fixed Point (Full) | ✅ Yes |
| Tier 1 성능 | ≤ 2% 회귀 | ✅ Yes |
| Tier 0/2/3 성능 | ≤ 5% 회귀 | ⚠️ Warning |

See `docs/BOOTSTRAP_BENCHMARK.md` and `docs/ROADMAP.md` for detailed documentation.

## Architecture Overview

### Compilation Pipeline

```
Source (.bmb)
    ↓ Lexer (logos-based)
Token stream
    ↓ Parser (LR(1) lalrpop)
Untyped AST
    ↓ Type Checker (Hindley-Milner)
Typed AST
    ↓ SMT Generator (Z3, optional)
Verified AST
    ↓ MIR Lowering
Middle IR
    ↓ CodeGen (LLVM/WASM)
Native Binary or WebAssembly
```

### Key Source Modules (`bmb/src/`)

| Module | Purpose |
|--------|---------|
| `lexer/` | Token generation (logos) |
| `parser/` | Parsing; `grammar.lalrpop` (58KB) is the complete grammar |
| `ast/` | Abstract syntax tree with spans |
| `types/` | Type inference, checking, generics (`infer.rs`, `unify.rs`, `generics.rs`) |
| `smt/` | SMT-LIB2 generation for Z3 verification |
| `verify/` | Contract verification orchestration |
| `interp/` | Tree-walking interpreter |
| `mir/` | Middle intermediate representation |
| `codegen/` | LLVM IR (`llvm.rs`) and WASM (`wasm.rs`) generation |
| `lsp/` | Language Server Protocol implementation |
| `repl/` | Interactive REPL |
| `error/` | Rich error reporting (ariadne) |
| `main.rs` | CLI orchestration (~3000 LOC) |

### Bootstrap Compiler (`bootstrap/`)

Self-hosted BMB compiler (~32K LOC) written in BMB itself:
- `lexer.bmb` - Tokenization
- `parser.bmb`, `parser_ast.bmb` - Parsing and AST generation
- `types.bmb` - Type inference (220KB)
- `mir.bmb`, `lowering.bmb` - MIR and AST→MIR transformation
- `llvm_ir.bmb` - LLVM IR codegen
- `compiler.bmb` - Full pipeline

### Standard Library (`stdlib/`)

Modular BMB standard library: `core/`, `string/`, `array/`, `io/`, collections.

### Ecosystem (`ecosystem/`)

- `gotgan/` - Package manager
- `vscode-bmb/` - VS Code extension
- `tree-sitter-bmb/` - Syntax highlighting
- `playground/` - Online editor
- `benchmark-bmb/` - Performance test suite

## Key Language Features

- **Expression-based**: Everything (if, match, let) is an expression
- **Contract-first**: Pre/post conditions and invariants integral to type system
- **Ownership model**: Rust-inspired (`own`, `&`, `&mut`)
- **Full generics**: Type parameters with bounds and where clauses
- **Multiple backends**: LLVM IR, WASM

## CI Requirements

All PRs must:
- Pass `cargo test --release`
- Pass `cargo clippy -- -D warnings`
- Maintain bootstrap self-compile time < 60s
- Pass performance regression check (2% threshold)
- Include performance impact analysis for runtime-affecting changes

## Known Failure Patterns & Self-Recovery

> 반복 발생한 실패 패턴 카탈로그. 테스트 실패 시 이 목록을 먼저 확인할 것.

### 빌드/테스트 실패

| 증상 | 원인 | 해결 |
|------|------|------|
| `cargo test` 미통과 | `--release` 누락 | `cargo test --release` |
| `cargo build` LLVM 헤더 에러 | 타겟 누락 | `--target x86_64-pc-windows-gnu` 추가 |
| 링크 에러 (undefined reference) | runtime 소스 누락 | `bmb_runtime.c` + `bmb_event_loop.c` 모두 포함 |
| 링크 에러 (ws2_32) | 소켓 라이브러리 누락 | `-lws2_32` 링크 플래그 추가 |
| `bc` not found (벤치마크) | Windows Git Bash 제약 | awk/python으로 대체 |

### 부트스트랩 실패

| 증상 | 원인 | 해결 |
|------|------|------|
| Stage 2 세그폴트 (O2만) | LLVM opt 최적화 버그 | O0/O1로 테스트, IR 패턴 분석 |
| Stage 2 스택 오버플로 | 깊은 재귀 | 트램폴린/반복 방식으로 전환 |
| Stage 2 잘못된 출력 | 부트스트랩 파서 버그 | 골든 테스트로 최소 재현 |
| S2 IR ≠ S3 IR (고정점 실패) | 코드젠 비결정성 | `diff s3.ll s4.ll` 첫 차이점에서 원인 추적 (binary hash ❌ — GCC MinGW 비결정적) |

### BMB 코드 생성 실패

| 증상 | 원인 | 해결 |
|------|------|------|
| 파서 에러 (unexpected token) | BMB 미지원 문법 사용 | `grammar.lalrpop` 확인 |
| 타입 에러 (method not found) | bootstrap 미지원 메서드 | 기존 테스트 참조 |
| "성능 한계"로 결론 | 근본 원인 미분석 | IR 비교 → 언어 스펙 변경 검토 |

## Key Documentation

- `docs/SPECIFICATION.md` - Language specification
- `docs/DEVELOPMENT.md` - Development principles and guidelines
- `docs/ARCHITECTURE.md` - Compiler internals
- `docs/LANGUAGE_REFERENCE.md` - Complete language reference
- `docs/ROADMAP.md` - Development roadmap
- `docs/BUILD_FROM_SOURCE.md` - Build instructions
