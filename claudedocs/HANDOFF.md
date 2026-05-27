# BMB Session Handoff — 2026-05-27 (Cycle 3205)

> **HEAD**: `(커밋 후 갱신)`
> **이번 세션 작업**: Cycle 3205 — **bootstrap.sh Full Fixed Point 복구 ✅ COMPLETE**
> **3-Stage Fixed Point**: `fixed_point: true` ✅ E2E 검증 완료
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **M10 상태**: ✅ **COMPLETE** (이전 세션)
> **Stage 2 상태**: ✅ **RECOVERED** (Cycle 3202 + 3204 재확인)
> **0-Warning 상태**: ✅ **RECOVERED** (Cycle 3203 + 3204 유지)
> **M11-B 상태**: ✅ **COMPLETE** (Cycle 3205 — fixed_point: true 확인)

---

## 이번 세션 작업 요약 (Cycle 3205)

### bootstrap.sh Full Fixed Point 복구

Cycle 3204의 `fixed_point: false` 잔여 문제를 근본 원인 진단 후 수정.

#### 근본 원인

`scripts/bootstrap.sh`가 Stage 2 바이너리 컴파일 시 `opt -passes='default<O3>,scalarizer'`
적용. LLVM opt의 O1+ 최적화가 BMB-generated IR의 printf 기반 방출 코드를 절단 →
Stage 3 IR이 6,193 lines 생성 (정상: ~134,211 lines).

#### 수정

`scripts/bootstrap.sh`에서 Stage 2 컴파일 경로 opt 제거 → `llc -filetype=obj -O2`만 사용.

#### 검증

```json
{"stage1":{"success":true},"stage2":{"success":true},"stage3":{"success":true},"fixed_point":true}
```

---

## 다음 세션 시작점

### 가능한 다음 단계 (우선순위 순)

| 순위 | 작업 | 설명 |
|------|------|------|
| 1 | **M11-A: semantic postcondition 교체** | ~1,114개 trivial contracts → 의미있는 계약으로 교체 |
| 2 | **언어 갭 추가** | stack array / closure / generic 등 미지원 기능 |

### M11-A 세부 계획

| 종류 | 수 | 현재 postcondition | 목표 |
|------|----|--------------------|------|
| bool (skip 확정) | 6 | `post it or not it` (tautology) | 변경 금지 |
| bool (교체 대상) | ~43개 | `post it or not it` (tautology) | 의미있는 불변식 |
| i64 (skip 확정) | 7 | `post it == it` (tautology) | 변경 금지 |
| String len≥0 (skip 확정) | 77 | `post it.len() >= 0` (trivial) | 변경 금지 |
| String len≥0 (교체 대상) | ~225개 | `post it.len() >= 0` (trivial) | 삭제 또는 의미있는 계약 |

> **주의**: ROADMAP `skip 확정` (6 bool + 77 String + 7 i64) 절대 변경 금지

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| chained_comparison | **0** ✅ |
| missing_postcondition | **0** ✅ |
| 총 warnings | **0** ✅ |
| Stage 1 bootstrap | ✅ |
| Stage 2 bootstrap | ✅ |
| bootstrap.sh fixed_point | ✅ **true** (E2E 확인) |
| BMB-internal Fixed Point | ✅ |
| 테스트 | 3800+ passed ✅ |

---

## 알려진 미결 사항

- **BMB IR → opt 최적화 불가**: printf 기반 IR 방출 코드가 opt O1+ 적용 시 절단됨.
  Stage 2 컴파일은 `llc -O2`만 사용 (opt 없음). IR 방출 방식 개선 시 opt 재적용 가능.
- **lint.exe 재빌드 불가**: `bootstrap/lint/lint.exe`의 PHI node IR 오류.
  `bmb lint` (Rust binary)로만 lint 실행 가능. 실질적 문제 없음.
- **Unix 링크 스택 미설정**: `bootstrap.sh` Unix 브랜치 carry-forward.
- **trivial postconditions**: ~1,114개 완전 무의미 계약 (M11-A 대상).

---

## 핵심 기술 메모

### `is_int_literal` 범위 주의 (핵심!)

```bmb
fn is_int_literal(kind: i64) -> bool =
  kind < 2000000000 + 100 or kind >= 2000000000 + 1000;
```

**2000000100–2000000999 범위는 제외!** (토큰 상수 범위)
→ match arm에 이 범위의 상수를 쓰면 파서가 변수 바인딩으로 해석
→ `kind == TK_IDENT()` 형태의 if-else 비교는 **match으로 변환 금지**
→ `bmb lint`의 `chained_comparison`은 zero-arg call RHS를 자동 면제함

### bootstrap.sh opt 제거 (핵심!)

Stage 2 바이너리 컴파일: `opt` 사용 금지, `llc -filetype=obj -O2`만 사용.
LLVM opt O1+ → BMB IR 절단 → fixed_point false.
