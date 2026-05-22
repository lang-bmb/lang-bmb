# ISSUE-20260522 — String builtin-return var + literal concat → GEP (native codegen)

## 핵심 메타

**우선순위**: P1
**영역**: codegen
**상태**: Open

## 측정 stamp (필수)

| 필드 | 값 | 비고 |
|------|-----|------|
| `measurement_date` | 2026-05-22 | |
| `stale_after` | 2026-08-22 | |
| `measurement_source` | `bmb build scripts/probe-llm-pipeline.bmb` | |
| `observed_rate` | 재현 100% (builtin-return String var + literal concat 모든 경우) | |
| `scope` | native codegen 전체 (인터프리터 정상) | |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / gcc MinGW | |

**측정 추이**:

| date | source | observed | 변화 |
|------|--------|----------|------|
| 2026-05-22 | probe-llm-pipeline native build | probe_exe.ll:1677 GEP 오류 | 최초 발견 |

## 문제

재현 코드:
```
let endpoint = getenv("SOME_VAR");  // or exec_with_stdin(...)
let _r = println("prefix: " + endpoint);  // FAILS in native build
```

`bmb build` 시 잘못된 LLVM IR 생성:
```
%_t15_offset_load = load i64, ptr %endpoint_v5.addr, align 8
%_t15 = getelementptr inbounds i64, ptr "prefix: ", i64 %_t15_offset_load
```

- `endpoint_v5.addr`는 `alloca ptr` (BmbString* pointer)
- `load i64`로 raw 포인터 값을 정수로 읽음
- GEP 인덱스로 사용 → `ptr "prefix: "[raw_addr]` → 유효하지 않은 LLVM IR

```
probe_exe.ll:1677: error: expected value token
  %_t15 = getelementptr inbounds i64, ptr "endpoint: ", i64 %_t15_offset_load
```

**인터프리터 모드**: 정상 동작 (타입 정보 직접 사용)

## 핵심 증거

조건:
- builtin 반환값 (`getenv`, `exec_with_stdin`)을 `let` 직접 바인딩
- 이후 `"literal" + var` 또는 `var + "literal"` 패턴

정상 동작하는 유사 패턴:
```
let temp_file = root + "/probe.bmb";      // String concat 결과 → 정상
let _r = println("[4/4]: " + temp_file);  // ← PASS
```

실패 패턴:
```
let endpoint = getenv("GPUSTACK_ENDPOINT");  // builtin 직접 바인딩 → 버그
let _r = println("endpoint: " + endpoint);   // ← FAIL (GEP 생성)
```

우회 방법 (Cycle 3044 확인):
```
// 함수 파라미터로 전달하면 ptr로 올바르게 처리됨
fn print_endpoint(ep: String) -> i64 = println("endpoint: " + ep);
let _r = print_endpoint(endpoint);  // PASS
```

## 추정 root cause

text backend (`bmb/src/codegen/llvm_text.rs`) `step_expr` 반복 lowering에서:
- builtin call 결과를 expr temp에 저장 (예: `%_t4.addr = alloca ptr`)
- 이를 named variable에 대입: `store ptr %_t4.call, ptr %endpoint_v5.addr`
- 이후 `"literal" + endpoint` 처리 시, codegen이 `endpoint_v5.addr` 대신 `_t4.addr`를 참조하는 버그
- 더 나쁘게, `_t4.addr` 에서 `i64`로 load (ptr를 정수로 캐스팅) → GEP 인덱스로 사용

가설: expr temp alloca (`_t4.addr`)와 named var alloca (`endpoint_v5.addr`) 간 
변수 추적 로직에 off-by-one 또는 wrong-pointer 버그 존재.

## 영향 평가

| 영역 | 영향 |
|------|------|
| CI | 기존 테스트에 builtin-return var concat 패턴 없어 CI 미탐지 |
| 부트스트랩 | 영향 없음 (bootstrap은 text backend 직접 사용하지 않음) |
| 개발 마찰 | M6-P2 bmb-ai-bench 포팅 시 native 빌드 불가 — 인터프리터 only |
| M축 | M6-P2 native 이식 블로킹 (우회 가능하나 번거로움) |

## 해결 방안 (옵션 비교)

### Option A: llvm_text.rs step_expr 수정 (proper fix)
- `estimated_cycles`: 1-2 **(hypothesis — IR 분석 후 결정)**
- 절차: step_expr에서 builtin call 결과 변수 참조 시 named var alloca 사용하도록 수정
- 리스크: step_expr는 복잡한 반복 lowering — 회귀 위험
- 검증: `cargo test --release` + probe native 빌드 성공

### Option B: 우회 패턴 표준화 (임시)
- `estimated_cycles`: 0 (이미 적용됨)
- 절차: 모든 BMB 스크립트에서 builtin-return var concat 시 helper fn 사용
- 트레이드오프: 스크립트마다 helper 함수 추가 → 코드 비대화

## HUMAN 결정 필요

- Option A vs B 우선순위 결정 (M6-P2 포팅이 우선이면 B로 계속 진행)

## 종결 기준 (close criteria)

- [ ] `bmb build scripts/probe-llm-pipeline.bmb` 성공 (Option A)
- [ ] 또는 M6-P2 완성 후 interpreter-only 운용으로 결정 (Option B 수용)
- [ ] `cargo test --release` 회귀 없음

## 메타

- 관련 ISSUE: 없음
- 인용 cycle: cycle-3044.md
- 외부 참조: `bmb/src/codegen/llvm_text.rs` step_expr lowering 영역
