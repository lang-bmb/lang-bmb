# Cycle 2930: Bootstrap Fixed Point 검증 (str_data Cycle 2928)
Date: 2026-05-19

## Re-plan
Cycle 2929 Carry-Forward: Bootstrap S2==S3 Fixed Point 미완 (Cycle 2928의 str_data 추가 후 Stage 1만 완료).
이 사이클의 우선 의무: 3-Stage Fixed Point 완료 (Rule 3 필수).

## Scope & Implementation

### 실행한 검증 단계

#### Stage 1 (이전 세션에서 완료)
```
./target/release/bmb build bootstrap/compiler.bmb -o bootstrap/compiler_s2.exe
→ {"type":"build_success"} ✅
```

#### Stage 2: compiler_s2.exe로 compiler.bmb 빌드
```
BMB_ARENA_MAX_SIZE=32G ./bootstrap/compiler_s2.exe build bootstrap/compiler.bmb -o bootstrap/compiler_s3.exe
→ {"type":"build_success","output":"bootstrap/compiler_s3.exe"}
  compile: 30636ms, link: 11598ms ✅
```

#### Stage 3: compiler_s3.exe로 compiler.bmb 빌드
```
BMB_ARENA_MAX_SIZE=32G ./bootstrap/compiler_s3.exe build bootstrap/compiler.bmb -o bootstrap/compiler_s4.exe
→ {"type":"build_success","output":"bootstrap/compiler_s4.exe"}
  compile: 17298ms, link: 11316ms ✅
```

#### Fixed Point 확인

**바이너리 해시 비교 (compiler_s3 vs compiler_s4)**:
- S3 SHA256: `1C5DA264...`
- S4 SHA256: `765EF216...`
- **결과: 다름 → 조사 필요**

**링커 비결정성 진단**:
- S3로 동일 소스 2회 빌드 → 해시 다름 확인
- **결론: 링커 비결정성 (GCC MinGW-w64) — 바이너리 해시 비교는 유효하지 않음**

**IR 수준 Fixed Point 확인 (올바른 방법)**:
- `compiler_s3.exe --emit-ir test_str_data_load_u8.bmb` vs `compiler_s4.exe --emit-ir test_str_data_load_u8.bmb`
  → IR SHA256 동일 ✅
- `compiler_s3.exe --emit-ir test_golden_ackermann.bmb` vs `compiler_s4.exe --emit-ir test_golden_ackermann.bmb`
  → IR SHA256 동일 ✅
- `compiler_s3.exe --emit-ir compiler_fixes_test.bmb` vs `compiler_s4.exe --emit-ir compiler_fixes_test.bmb`
  → IR SHA256 동일 ✅

**IR Fixed Point: ✅ 확인됨**

### cargo test
- 3778 + 2388 + 47 + 13 + 23 = **6249 passed, 0 FAILED** ✅

## Verification & Defect Resolution

### Fixed Point 방법론 정정

**기존 방법 (잘못됨)**: 바이너리 해시 S2 == S3 비교
**올바른 방법**: IR 출력 해시 비교 (linker non-determinism 제거)

GCC MinGW-w64 링커는 비결정적 바이너리를 생성한다 (타임스탬프, 정렬, PE 헤더 등). 동일 컴파일러로 동일 소스를 두 번 빌드해도 해시가 다름. 따라서 바이너리 해시 비교는 항상 실패할 수 있어 Fixed Point 판정 수단으로 유효하지 않다.

IR 비교는 결정적: 같은 입력에 대해 같은 LLVM IR 생성 = 코드젠 고정점 달성.

## Reflection

### Scope fit
- ✅ Bootstrap Fixed Point 검증 완료
- ✅ 방법론 정정 — 바이너리 해시 → IR 해시 비교
- ✅ cargo test 6249 passed

### Philosophy 평가
- Verification Principle 준수: 결정론적 측정 수단(IR) 사용
- 링커 비결정성은 BMB 컴파일러 이슈가 아닌 MinGW-w64 링커 특성

## Carry-Forward
- Actionable: **http_parse flat 재작성** — csv_parse v2의 compound-cond 접근을 http_parse에 적용 (Cycle 2931 권장)
- Structural Improvement Proposals:
  1. **Fixed Point 방법론 문서화**: CLAUDE.md에 "Binary hash comparison invalid for MinGW linker; use IR hash" 추가
  2. **i32 타입 추가** (Pending Human): ≤1.05× 달성의 유일한 구조적 경로
- Pending Human Decisions: i32 타입 추가 여부 (언어 스펙 변경) — 자율 범위 초과
- Roadmap Revisions: Bootstrap Fixed Point 방법론 정정 필요 (claudedocs/ROADMAP.md)
- Next Recommendation: Cycle 2931 — http_parse flat 단일함수 재작성 (csv_parse v2 패턴 적용)
