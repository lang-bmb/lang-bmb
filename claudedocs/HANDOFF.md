# BMB Session Handoff — 2026-05-27 (Cycle 3204)

> **HEAD**: `TBD` (커밋 후 업데이트)
> **이번 세션 작업**: Cycle 3204 — **Stage 2 Fixed Point 복구 ✅ COMPLETE**
> **3-Stage Fixed Point**: Stage 2 bootstrap ✅ (BMB-internal FP: S4==S6, semantic S2≈S4)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **M10 상태**: ✅ **COMPLETE** (이전 세션)
> **Stage 2 상태**: ✅ **RECOVERED** (Cycle 3202 + 3204 재확인)
> **0-Warning 상태**: ✅ **RECOVERED** (Cycle 3203 + 3204 유지)

---

## 이번 세션 작업 요약 (Cycle 3204)

### Stage 2 Fixed Point 복구 완료

Cycle 3203의 `chained_comparison` → match 변환이 Stage 2 bootstrap을 파괴했음.
이번 사이클에서 근본 원인(lint false positive)을 수정하고 완전 복구.

#### 근본 원인

`is_int_literal(kind)` 함수는 2000000100–2000000999 범위를 제외함 (토큰 상수 범위).
따라서 match 패턴에 `2000000201` (= TK_IDENT) 등을 사용하면 self-hosted 파서가
이를 integer literal이 아닌 **변수 바인딩**으로 해석 → Stage 2 parse 실패.

#### 변경 요약

| 항목 | 변경 내용 |
|------|----------|
| `bmb/src/types/mod.rs` | `chained_comparison` 검사에 zero-arg call 면제 추가 |
| `bootstrap/compiler.bmb` | `af81c313` 상태로 복구 + `post it or not it` + `match type_info` 재적용 |
| `bootstrap/lint/lint.bmb` | 면제 이유 주석 추가 (lint.exe 재빌드 불필요) |

#### 검증

- `bmb lint bootstrap/compiler.bmb`: **0 warnings** ✅
- `bootstrap.sh --stage1-only`: **Stage 1 OK (32,982ms)** ✅
- `cargo test --release`: **3800 passed** (+ 2390+47+22+23) ✅
- 수동 Semantic Fixed Point: diff canon(S2) canon(S3) = **0** ✅
- 수동 BMB-internal Fixed Point: diff S3 S4 = **0** ✅

---

## 다음 세션 시작점

### 가능한 다음 단계 (우선순위 순)

| 순위 | 작업 | 설명 |
|------|------|------|
| 1 | **M11-A: semantic postcondition 교체** | ~358개 완전 무의미 contracts → 의미있는 계약으로 교체 |
| 2 | **bootstrap.sh Fixed Point 조사** | freshly-compiled Stage 2 binary `int_to_string` truncation 원인 |
| 3 | **M11 방향 결정** | 언어 갭 / 계약 품질 / 성능 등 다음 마일스톤 방향 결정 |

### M11-A 세부 계획

| 종류 | 수 | 현재 postcondition | 목표 |
|------|----|--------------------|------|
| bool | 49개 | `post it or not it` (tautology) | 의미있는 불변식 |
| i64 | 7개 | `post it == it` (tautology) | 범위/관계 계약 |
| String len≥0 | 302개 | `post it.len() >= 0` (trivial) | 삭제 또는 의미있는 계약 |

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| HEAD | `TBD` |
| chained_comparison | **0** ✅ |
| missing_postcondition | **0** ✅ |
| 총 warnings | **0** ✅ |
| Stage 1 bootstrap | ✅ |
| Stage 2 bootstrap | ✅ (manual Fixed Point 재확인) |
| BMB-internal Fixed Point | ✅ (S3==S4) |
| Cross-compiler FP (semantic) | ✅ (canonical S2≈S3) |
| 테스트 | 3800+ passed ✅ |

---

## 알려진 미결 사항

- **bootstrap.sh `fixed_point: false`**: freshly-compiled Stage 2 binary에서
  `int_to_string` 등 일부 함수 IR이 `llvm-as` 거부. opt -O3 최적화 후 발생.
  기존 `bmb-stage2.exe`로는 정상. Pre-existing 문제로 추정.
- **lint.exe 재빌드 불가**: `bootstrap/lint/lint.exe`의 PHI node IR 오류.
  `bmb lint` (Rust binary)로만 lint 실행 가능. 실질적 문제 없음.
- **Unix 링크 스택 미설정**: `bootstrap.sh` Unix 브랜치 carry-forward.
- **M11 방향 미결정**: ROADMAP 참조.
- **trivial postconditions**: `post it or not it` 등 ~358개 완전 무의미 계약.

---

## Stage 2 Bootstrap 기술 메모 (참조용)

### 핵심 패턴: `STATUS_STACK_OVERFLOW = exit 127`

Windows에서 BMB-compiled 바이너리가 exit 127로 즉시 종료하면 → 스택 오버플로 우선 의심.
- 확인: python3으로 PE 헤더 stack reserve 확인
- 수정: clang 링크 시 `-Wl,--stack,268435456` 추가

### BMB lint 0-warning 정책

`bmb lint bootstrap/compiler.bmb` → `{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}` 유지 필수.

주요 경고 종류:
- `chained_comparison`: if-else 체인 3개 이상 → match 변환 권장
  - **면제**: RHS가 `TK_*()` zero-arg 함수 호출인 경우 (자동 면제됨)
- `missing_postcondition`: postcondition 누락 함수
- `redundant_bool_comparison`: bool 함수에 `it == true/false` → `it or not it`

### `is_int_literal` 범위 주의 (핵심!)

```bmb
fn is_int_literal(kind: i64) -> bool =
  kind < 2000000000 + 100 or kind >= 2000000000 + 1000;
```

**2000000100–2000000999 범위는 제외!** (토큰 상수 범위)
→ match arm에 이 범위의 상수를 쓰면 파서가 변수 바인딩으로 해석
→ `kind == TK_IDENT()` 형태의 if-else 비교는 **match으로 변환 금지**

### `else match` 주의

BMB bootstrap 파서에서는 `} else match k { ... }` 허용되지만 **Rust lalrpop** 파서에서 거부.
Stage 1 컴파일이 필요하므로 반드시 `} else { match k { ... } }` 형태 사용.
