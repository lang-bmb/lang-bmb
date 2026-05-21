# Cycle 2722: Match jump table 재진단
Date: 2026-05-11

## Re-plan
인계 (Cycle 2721 RE-PLAN): brainfuck/lexer LLVM IR 검증 — match jump table infra 작동 여부. Trigger ⚪ NONE.

## Scope & Implementation

### 검증 방법
1. `bmb build --emit-ir` → 원본 IR
2. `opt -O2` → 최적화 IR
3. `llc -O3 -filetype=asm` → 어셈블리
4. switch instruction count + indirect jumps + LJTI label references 비교

### 결과 매트릭스

| 벤치마크 | BMB IR switch | opt IR switch | asm indirect jump | LJTI refs | jump table 작동? |
|---------|--------------|--------------|------------------|-----------|----------------|
| brainfuck | 1 | 3 | 1 | 53 | ✅ **작동** |
| lexer | 1 | 0 | 0 | 0 | ❌ **미작동** |

### 핵심 발견

**brainfuck**: `if c == 62 { ... } else if c == 60 { ... }` 형태의 sequential if-else 체인. LLVM SimplifyCFG가 이를 인식해 **dense switch + jump table로 자동 변환** (1 indirect jump + 53 LJTI label references).

→ ISSUE의 원진단 ("match → switch 매핑 필요")은 **brainfuck에 대해 false positive**. 인프라 작동 중.

**lexer**: `if c == 43 or c == 45 or c == 42 or ...` 형태의 multi-OR 단일 if. BMB가 `or`을 **eager evaluation**으로 lowering:
```llvm
%_t0 = icmp eq i64 %c, 43
store i1 %_t0, ptr %_t0.addr, align 1
%_t1 = icmp eq i64 %c, 45
store i1 %_t1, ptr %_t1.addr, align 1
%_t2 = or i1 %lhs, %rhs   ; ... (모든 비교 평가 후 OR)
```

LLVM SimplifyCFG는 **sequential branch** 패턴을 switch로 묶을 수 있으나, `or` chain의 eager evaluation + alloca/store/load 패턴은 변환 못 함. → **lexer slowness 원인 #1 후보**.

### Decision Framework 진단

| 수준 | 진단 | 변경 필요? |
|------|------|-----------|
| 1. 언어 스펙 | `or`은 short-circuit이어야 함 (spec 명세) | ✅ 검토 필요 — 현 lowering이 short-circuit 위반? |
| 2. 컴파일러 구조 | BMB IR generator의 `or` lowering 패턴 | ✅ **proper fix 후보** |
| 3. 최적화 패스 | LLVM SimplifyCFG가 처리 (BMB IR 정정 후 자동) | ❌ no change |
| 4. 코드 생성 | LLVM 자동 (BMB IR 정정 시) | ❌ |
| 5. 런타임 | — | — |

**Proper fix 권고**: BMB IR이 `or`을 short-circuit branch chain으로 lowering. Cycle 2723-2725 후보.

### 부수 발견

- **alloca + store/load 패턴**: BMB가 boolean 임시값을 메모리 변수로 lowering — register allocator 부담 + opt 변환 차단. mem2reg/SROA가 이를 처리하나? IR 본 결과 mem2reg가 `i1.addr` 정리 못함 (별도 cycle 분석 후보).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| brainfuck IR / opt IR / asm 추출 | ✅ |
| lexer IR / opt IR / asm 추출 | ✅ |
| jump table 작동 확인 (brainfuck) | ✅ 53 LJTI refs |
| jump table 미작동 확인 (lexer) | ✅ 0 LJTI refs |
| 실제 원인 식별 (lexer `or` lowering) | ✅ |
| BMB IR `is_operator_char` lowering 패턴 확인 | ✅ alloca + store/load + eager OR |

결함 (cycle): 없음. 재진단 산출물 명확.

## Reflection

### 외부 관찰자 관점

1. **재진단의 가치**: ISSUE 원진단을 무비판 수용했으면 이미 작동하는 인프라 재구현. brainfuck의 LJTI 53개는 jump table 완전 작동의 증거. **이슈 진단은 evidence-based 재검증 의무** 교훈.

2. **lexer의 진짜 원인**: ASCII byte 분류에 `or` chain 사용 — 인터프리터/lexer 패턴에서 흔함. BMB IR의 eager evaluation + alloca/store/load가 LLVM 최적화 차단. C 동등 `c == X || c == Y` 는 GCC/Clang에서 sequential branch + switch + jump table까지 자동 변환.

3. **단일 cycle로 분해 가능 분석**: 8분 작업 (IR 생성 + asm 생성 + grep). Cycle 2721 평가에서 "1 cycle 가능"이라 분류한 것이 정확.

4. **새 ISSUE 후보**: "BMB `or` chain lowering optimization" — 즉시 등록 권고.

### Roadmap impact

- ISSUE-20260413-match-jump-table: **rediagnosis 완료, false-positive 처리**
- 새 ISSUE: BMB `or` chain lowering — Cycle 2723-2724에 fix 후보 (compare-inline보다 ROI 높을 가능성)
- Cycle 2723 plan 재고: compare-inline 대신 `or` chain fix?

## Carry-Forward

- Actionable (Cycle 2723):
  - **🔍 ISSUE 재스코프**: ISSUE-20260413-match-jump-table → "lexer `or` lowering" 으로 변경 또는 close + 새 ISSUE
  - **`or` chain lowering proper fix 검토**: bootstrap/compiler.bmb IR generator
- Structural Improvement Proposals:
  - **BMB IR boolean alloca 정리 분석**: mem2reg가 `i1.addr` 처리 못하는 이유
  - **`and` chain도 동일 패턴**: 같이 분석 가능
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (Cycle 2723 plan 재고 — 다음 cycle STEP 0)
- Next Recommendation: Cycle 2723 = `or` chain lowering 분석 + 선택지 산출 (proper fix 또는 carry-forward)
