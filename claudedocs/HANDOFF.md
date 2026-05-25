# BMB Session Handoff — 2026-05-25 (Cycles 3094-3102)

> **HEAD**: `c9ef6fcc`
> **이번 세션 작업**: Cycles 3094-3102 (M7-4 COMPLETE — AI 계약 생성 파이프라인)
> **3-Stage Fixed Point**: ✅ `ea550bf3` (변경 없음 — 계약만 추가)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: **M8 계획** 또는 **Track B 계약 계속** (1342개 미계약 잔여)

---

## 이번 세션 작업 요약 (Cycles 3094-3102)

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3094 | `bmb verify --list-uncontracted` | CLI 추가, 1467개 미계약 함수 JSON |
| 3095 | `suggest_contracts` MCP tool | heuristic 제안 (pos→pre≥0, find_→post≥-1) |
| 3096 | `list-uncontracted.bmb` 자동화 | P1(683)/P2(23)/P3(761) 분류 스크립트 |
| 3097 | Track B: P2 + 주요 P1 | 15개 계약 (count_/find_/skip_/scan_) |
| 3098 | Track B: skip_/find_ 배치 | 24개 계약 (파서/렉서 skip_/find_ 배치) |
| 3099 | Track B: find_/keyword_ | 41개 계약 (find_ 28 + skip_ 3 + keyword_ 9) |
| 3100 | Track B: count_/get_ | 21개 계약 (count_ 6 + get_ 15) |
| 3101 | Track B: collect_/index_/trl_ | 24개 계약 (collect_ 12 + index_ 7 + trl_ 5) |
| 3102 | M7-4 COMPLETE 선언 | ROADMAP/HANDOFF 업데이트 |

### 핵심 성과

**M7-4 ✅ COMPLETE** — 자동 Contract 생성 AI 파이프라인:

1. **`bmb verify --list-uncontracted`** (Cycle 3094):
   - `bmb/src/main.rs`에 `--list-uncontracted` + `--suggest` 플래그 추가
   - `list_uncontracted_fns()`: AST 스캔 → JSON 출력 (name/line/params)
   - 검증: 1513 총 함수 중 1467개 미계약 확인

2. **`suggest_contracts` MCP tool** (Cycle 3095):
   - `ecosystem/bmb-mcp/mcp_server.bmb`에 9번째 tool 추가
   - heuristic: pos/idx/start 파라미터 → `pre >= 0`; find_/skip_ 이름 → `post >= -1`
   - `str_find_from` 헬퍼 (3-인수 str_find 대체), `extract_param_name`, `param_is_pos_like` 등

3. **`list-uncontracted.bmb` 자동화** (Cycle 3096):
   - BMB 자체로 작성된 우선순위 분류 스크립트
   - P1(683): pos/idx/start/offset 파라미터 → 즉시 `pre >= 0`
   - P2(23): find_/skip_/scan_ 패턴 이름 → `post >= -1` 또는 `post >= 0`
   - P3(761): 기타 (parser/llvm/etc.)

4. **Track B 계약 125개 추가** (Cycles 3097-3101):
   - 1467 → 1342 미계약 (8.5% 감소)
   - Python regex 배치 패치로 효율적 적용
   - 주요 계약 패턴: `pre pos >= 0`, `post it >= 0`, `post it >= -1`

**M7 전체 ✅ COMPLETE** (M7-1 ~ M7-4):
- M7-1: compiler.bmb 계약 17종 + llvm.assume 25개 (Cycle 3075-3078)
- M7-2: SmtSort::Str + String SMT theory (Cycle 3079)
- M7-3: forall/exists E2E + Track B 20종+ (Cycles 3084-3093)
- M7-4: AI 파이프라인 + Track B 125종 (Cycles 3094-3102)

### 신규 파일

- `bootstrap/list-uncontracted.bmb`: Track B 자동화 스크립트
- `claudedocs/cycle-logs/cycle-3094.md` ~ `cycle-3102.md`

### 검증 결과

- `bmb check bootstrap/compiler.bmb`: ✅ (3232 warnings, 0 errors)
- `bmb verify bootstrap/compiler.bmb --list-uncontracted`: 1342 미계약 ✅
- `bmb run bootstrap/list-uncontracted.bmb`: `{"total":1342,"priority1_pos_param":...}` ✅
- 3-Stage Fixed Point: `ea550bf3` ✅ (계약만 추가 — Fixed Point 불변)

---

## 다음 세션 시작점

### 즉시 착수 가능 (자율 결정)

**Track B 계속** (1342개 잔여):
- P1 잔여: 파서 함수 148개 (parse_* prefix)
- LLVM 관련: 50개 (llvm_* prefix)
- `bmb run bootstrap/list-uncontracted.bmb`로 현황 재확인

**M8 계획 수립**:
- ROADMAP.md M8 섹션 신규 정의
- 제안: M8 = Native 컴파일 완전화 (bootstrap → native build pipeline)

### HUMAN 결정 필요

없음 — 자율 결정 가능.

---

## 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| 총 함수 | 1513 |
| 계약 있음 | 171 (11.3%) |
| 미계약 | 1342 |
| 3-Stage FP | `ea550bf3` |
| cargo test | ✅ |
| bmb check | ✅ (3232 warnings) |

---

## 알려진 미결 사항

- **함수 호출 body post 불검증**: `fn f() = g()` 형태 — callee가 uninterpreted. workaround 없음, 언어 설계 한계.
- **Track B 1342개 잔여**: 지속적 작업 필요 (목표: 전 함수 계약)
- **bool 반환 함수 post 계약**: trivially true → 우선순위 낮음
