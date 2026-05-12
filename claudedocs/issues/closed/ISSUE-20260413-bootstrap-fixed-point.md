# ISSUE-20260413 — Bootstrap Fixed Point 달성

**우선순위**: P0
**영역**: bootstrap
**상태**: ✅ Resolved (Cycle 363, 2026-04-13)
**블로킹**: CLAUDE.md Rule 3 위반 (3-Stage Fixed Point 필수) — 해소됨

## 해결 내역 (2026-04-13, Cycles 362-363)

HANDOFF의 원인 추정("ifs_check_flex 함수명 생성 비결정성")은 stale 아티팩트 기반 오진이었음. 실제 원인:
1. `runtime/libbmb_runtime.a`가 weak `main` 적용 전 빌드(stale)되어 multiple-definition 링커 에러 발생
2. `bmb/src/codegen/llvm.rs:582` `add_inline_main`이 `bmb_init_runtime(argc, argv)` 호출 누락 → `g_argc=0`

Fix:
- `scripts/bootstrap.sh`: runtime .a가 source .c보다 오래되면 자동 재빌드
- `llvm.rs` add_inline_main: text backend와 동일하게 argc/argv 전달 + arena 정리 추가

검증: `./scripts/bootstrap.sh` → Fixed Point true (S2 == S3, 108574 lines identical).

## 문제

Stage 2/3 컴파일러가 생성하는 LLVM IR이 동일하지 않음 — Fixed Point 미달성.

### 증거

```bash
$ diff bootstrap/bmb_stage2.exe.ll bootstrap/bmb_stage3.exe.ll
< %_t28 = call i64 @ifs_check_flex_both_sides(i64 %fn_mir, i64 %line, i64 %pos, i64 %sb)
---
> %_t28 = call i64 @ifs_check_flex_copy_else(i64 %fn_mir, i64 %line, i64 %pos, i64 %sb)

< define i64 @ifs_check_flex_both_sides(...)
---
> define i64 @ifs_check_flex_copy_else(...)
```

### md5 불일치

```
bmb_stage2.exe: ad89a9f29978ce1128ee6887f13efdf1
bmb_stage3.exe: 32aceb8ecb279bc4084418996680f2e1
```

ROADMAP.md는 "3-Stage Fixed Point (S2 == S3)"라고 선언하고 있으나 **현재 실제 상태와 불일치**.

## 근본 원인 추정

1. **함수명 생성 비결정성**: `ifs_check_flex_*` 계열 함수명이 입력 순서/맵 반복 순서에 의존
2. **if-flex 최적화 경로**: 한 단계에서 `both_sides`, 다른 단계에서 `copy_else`로 다른 경로 선택
3. **HashMap 반복 순서**: BMB HashMap이 비결정적 반복 순서를 가질 가능성

## 해결 방안

### 1단계: 함수명 생성 결정론화
- 함수명을 입력 파일의 **위치 기반**으로 생성 (예: `ifs_check_flex_{file}_{line}_{col}`)
- 또는 **정규화된 카운터** 기반 (AST 순회 순서로 할당)

### 2단계: if-flex 최적화 경로 추적
- `bootstrap/compiler.bmb` 또는 `bootstrap/optimize.bmb`에서 `both_sides`/`copy_else` 분기 조건 확인
- 같은 입력에 대해 Stage 2/3가 다른 경로를 선택하는 이유 파악

### 3단계: 결정론적 맵/집합 사용
- HashMap → BTreeMap (정렬된 반복)
- 집합 반복 순서 의존 코드 탐지

## 구현 단계

1. [ ] `grep -rn "ifs_check_flex" bootstrap/` — 함수 생성 지점 파악
2. [ ] 함수명 생성 로직 분석 및 결정론적 재작성
3. [ ] Stage 2 IR vs Stage 3 IR 전체 diff 재실행
4. [ ] `./scripts/bootstrap.sh` 통과 확인 (exit code 0)
5. [ ] ROADMAP.md 업데이트 (G-1 실제 상태 반영)

## 완료 기준

- `md5sum bmb_stage2.exe bmb_stage3.exe` 동일
- `diff bmb_stage2.exe.ll bmb_stage3.exe.ll` 빈 출력
- `./scripts/bootstrap.sh --json` exit code 0 + `"fixed_point": true`
- 106,011 lines IR에서 어떤 diff도 없음

## 주의

이 이슈는 **다른 P0 이슈(match→jump table, hashmap 최적화) 작업 후 재발할 수 있음**. 새 코드젠 경로 추가 시 결정론성 테스트 필수.
