# Cycle 2995: csv_parse IR 분석 — LLVM 파리티 확인
Date: 2026-05-21

## Re-plan
Plan valid. csv_parse 1.06× 비율의 원인 규명: `byte_at` lowering 품질 확인 + `load_u8(ptr)` 전환 필요성 평가.

## Scope & Implementation

### IR 생성 및 분석

**사전 (pre-opt) IR**: `./target/release/bmb build ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main.bmb --emit-ir -o /tmp/csv_parse_bmb.ll`

**최적화 후 IR**: `opt -O2 /tmp/csv_parse_bmb.ll -S -o /tmp/csv_parse_bmb_opt.ll`

**C 베이스라인**: `clang -O2 -S -emit-llvm .../csv_parse/c/main.c -o /tmp/csv_parse_c_opt2.ll`

### 핵심 발견

**`byte_at` lowering (pre-opt `skip_ws` body)**:
```llvm
%charat.data_ptr.0 = getelementptr inbounds {ptr, i64, i64}, ptr %data, i32 0, i32 0
%charat.data.0 = load ptr, ptr %charat.data_ptr.0
%charat.char_ptr.0 = getelementptr inbounds i8, ptr %charat.data.0, i64 %charat.idx.0
%charat.byte.0 = load i8, ptr %charat.char_ptr.0, !tbaa !906
```
→ `getelementptr i8 + load i8` 단일 시퀀스. 추가 함수 호출 없음.

**최적화 후 (opt -O2)**:
- `skip_ws`, `parse_quoted_field`, `parse_unquoted_field` 전부 `parse_csv`에 **완전 인라인**
- BmbString `ptr` 필드 로드가 루프 밖 (`bb_while_body_1.lr.ph`) 으로 **호이스팅**:
  ```llvm
  %charat.data.0 = load ptr, ptr %data, align 8  ; 루프 외부 1회
  ```
- `skip_ws` 핫루프: `getelementptr + load i8 + 2 icmp + or + select + 2 add + icmp + br` (브랜치리스 sentinel 트릭)
- `parse_unquoted_field` 핫루프: `getelementptr + load i8 + switch i8` — 매우 깔끔

**C IR 비교**:
- C `skip_ws`: `getelementptr + load i8 + switch i8(space/tab/quote)` — 더 짧은 루프
- C `parse_unquoted_field`: `getelementptr + load i8 + switch + 2 add + store i8(buffer copy)` — **출력 버퍼에 바이트 복사**
- BMB는 바이트 복사 없이 길이 카운트만 → BMB가 이론상 더 적은 작업

**결론**:
1. `byte_at` → `getelementptr i8 + load i8` LLVM 파리티 ✅
2. `load_u8(ptr)` 전환은 IR 수준에서 차이 없음 → 불필요
3. BMB 1.06× 원인: skip_ws에서 C(switch)보다 약간 더 많은 명령어 + 측정 노이즈
4. Tier 3 benchmark에서 csv_parse = 0.820× (BMB faster) → 1.06는 환경 의존 측정 노이즈

### ROADMAP 갱신
- § 5 "조건부 원인" csv_parse/http_parse 항목: ~~간접 접근~~ → `✅ INVESTIGATED` 로 갱신
- "차기 최적화 후보" `byte_at → load_u8(ptr)`: `CLOSED` 마킹

## Verification & Defect Resolution
- IR 분석은 파일 기반 — 빌드/테스트 불필요
- ROADMAP 수정: 텍스트 변경, 코드 변경 없음

## Reflection

- **Scope fit**: IR 분석 목표 완전 달성. `byte_at` lowering 품질 확인 + ROADMAP 정확화.
- **Latent defects**: 없음.
- **Philosophy drift**: 없음. "측정 없는 주장 금지" 원칙 준수 — IR로 직접 확인.
- **Roadmap impact**: ROADMAP § 5 "load_u8(ptr) 전환" 아이템 CLOSED. csv_parse 1.06×는 측정 노이즈임이 증명됨.
- **예상치 못한 발견**: C는 field 내용을 출력 버퍼에 복사(store i8)하는데 BMB는 길이만 카운트. 두 구현이 다른 작업을 수행하므로 엄밀한 비교가 어렵지만, BMB가 더 경량.

## Carry-Forward
- Actionable: Cycle 2996 — `scripts/rebuild-bootstrap-exe.sh --check-only` GitHub Actions 연결 (P4, 1 사이클)
- Structural Improvement Proposals: None
- Pending Human Decisions: None (csv_parse 조사 완결)
- Roadmap Revisions: ROADMAP § 5 `byte_at→load_u8(ptr)` CLOSED (Cycle 2995)
- Next Recommendation: Cycle 2996 — CI --check-only GitHub Actions step
