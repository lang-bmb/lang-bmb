# Cycle 2928: str_data builtin 추가 + byte_at LICM 검증
Date: 2026-05-19

## Re-plan
HANDOFF Carry-Forward: 없음. ROADMAP 권장: byte_at → load_u8(ptr) 최적화 (long-term, csv/http parse ≤1.05× 목표).
byte_at overhead 제거 시도: str_data(s) -> i64 + load_u8(ptr+i) 패턴으로 inner loop에서 String struct lookup 제거.
Plan valid, 진행.

## Scope & Implementation

### 조사 결과 (STEP 1)

#### 현황 파악
- `str_data(s: String) -> i64`: Rust text backend (llvm_text.rs:5689) ✅, Rust 타입 시스템 (types/mod.rs:540) ✅
- `load_u8(addr: i64) -> i64`: bootstrap builtin (compiler.bmb:11229) ✅, Rust text backend ✅
- **`str_data` 부재**: bootstrap compiler의 user-callable builtin 목록에 없음 → 추가 필요

#### byte_at의 현재 LLVM IR
```llvm
-- 루프마다 반복 (LICM 이전):
%dest_str = inttoptr i64 %str_handle to ptr
%dest_dpp = getelementptr inbounds {ptr,i64,i64}, ptr %dest_str, i32 0, i32 0
%dest_dp = load ptr, ptr %dest_dpp, !invariant.load !0  ← 루프 불변
%dest_bp = getelementptr inbounds i8, ptr %dest_dp, i64 %idx
%dest_b = load i8, ptr %dest_bp
%dest = zext i8 %dest_b to i64
```

### 변경 사항

#### 1. bootstrap/compiler.bmb: `@str_data` 인라인 에미터 추가
- `llvm_gen_call` 함수 (line ~7140)에 `@str_data` case 추가
- `llvm_gen_call_reg` 함수 (line ~7447)에 동일 추가
- `mlcse_safe_builtins` 목록에 `str_data` 추가 (line 11229)
- 인라인 IR: `inttoptr → GEP field[0] → load ptr (!invariant.load) → ptrtoint`

#### 2. tests/bootstrap/test_str_data_load_u8.bmb: 신규 테스트
- `str_data(s) + load_u8(ptr+i)` 패턴 검증
- byte_at와 동일 결과 확인 (comma count 4/4 ✓)

#### 3. csv_parse/bmb/main_inproc.bmb: str_data+load_u8로 재작성
- 모든 inner 함수를 `ptr: i64, len: i64` 기반으로 전환
- `parse_csv` 진입 시 `str_data(data)` 1회 호출 후 ptr 전달

#### 4. http_parse/bmb/main_inproc.bmb: Cycle 2924 버전으로 복원
- str_data 버전이 더 느림 (2906→3017 µs, 짧은 문자열에서 extra arg overhead 지배)
- byte_at 버전이 최적임을 확인

### Stage 1 빌드 (bootstrap self-compile)
```
./target/release/bmb build bootstrap/compiler.bmb -o bootstrap/compiler_s2.exe
→ {"type":"build_success"}  ✅
```

## Verification & Defect Resolution

### cargo test
- 6249+ passed, 0 FAILED ✅ (3778 + 2388 + 47 + 13 + 23)

### A/B 측정 (동일 조건, 7회 median)

| 버전 | CSV BMB (µs) | 비고 |
|------|-------------|------|
| byte_at (Cycle 2923) | 3515 | 기준선 |
| str_data+load_u8 | 3524 | +0.3% (사실상 동일) |

**str_data/byte_at ratio: 1.003× → 중립 (개선 없음)**

### 핵심 발견: LLVM LICM이 이미 처리하고 있음
LLVM은 `!invariant.load` + `readonly noalias` 파라미터 정보를 이용해 byte_at의 루프 불변 struct load를 LICM으로 호이스트:
- 루프 이전 1회: `inttoptr + GEP field[0] + load ptr`
- 루프당: `GEP i8 + load i8 + zext` (3 insn)

str_data+load_u8 패턴:
- str_data 1회: 동일 4 insn
- 루프당: `add i64 + inttoptr + load i8 + zext` (4 insn, 1 추가)

**결론: byte_at가 str_data+load_u8보다 실제로 더 효율적** (GEP가 inttoptr+add보다 LLVM 친화적)

### http_parse 회귀 원인
- 함수들이 `(s, ptr, len)` 3인자 → (s, pos) 2인자 대비 call overhead 증가
- 짧은 문자열 (100-200 bytes) 파싱: 함수 call overhead > scan overhead
- str_data 버전: 3017 µs vs byte_at 버전: 2906 µs → 3.8% 느림 → 복원

### csv_parse 현황
- str_data 버전: 3524 µs ≈ byte_at 버전: 3515 µs → 중립
- str_data 버전을 유지 (새 API 시연, 기능 정확, 성능 동일)
- **실제 overhead 원인: byte_at가 아니라 함수 call indirection** (C는 모두 inline 단일 함수)

## Reflection

### Scope fit
- str_data 추가: ✅ (bootstrap 새 capability)
- 성능 개선 목표 (≤1.05×): ❌ 미달성 — LICM으로 이미 최적화됨이 밝혀짐

### 핵심 학습
1. `byte_at` + LLVM LICM = `str_data+load_u8` (동등 성능)
2. csv/http 14-18% overhead는 byte_at가 아니라 **함수 call 구조**에서 발생
3. C vs BMB의 진짜 차이: C는 단일 flatten 함수, BMB는 분리된 함수들의 call chain
4. 실질적 개선 경로: CSV parse 알고리즘을 C처럼 단일 함수로 구조조정

### 철학 평가
- Principle 2 (Workaround 금지) 준수: str_data가 효과 없음을 측정으로 확인 후 포기
- Verification Principle: 측정으로 가설 기각 — 올바른 접근

## Carry-Forward
- Actionable: **csv_parse 알고리즘 구조조정** — C처럼 단일 flatten 함수로 (Cycle 2929 권장)
- Structural Improvement Proposals:
  1. **csv_parse 알고리즘 재작성**: line-scan 후 field-parse inline → function call overhead 제거 → ≤1.05× 가능
  2. **http_parse**: `content-length` 헤더 scan이 항상 14-char 비교 — str_data+ptr 방식이 유효한 경우가 있음 (longer headers)
- Pending Human Decisions: 없음 (기존 동일)
- Roadmap Revisions: byte_at 최적화 분류를 "LICM으로 이미 최적화됨, 실제 overhead = call structure" 로 갱신 필요
- Next Recommendation: Cycle 2929 — csv_parse를 C-스타일 단일 flatten 함수로 재작성 (≤1.05× 목표)
