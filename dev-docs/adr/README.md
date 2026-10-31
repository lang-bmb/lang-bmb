# 결정 기록 (ADR)

BMB v2의 설계 결정을 한 건에 파일 하나로 남긴다. 현재 확정된 설계는 `docs/DESIGN.md`에 있고, ADR은 그 설계가 *왜* 그렇게 되었는지를 결정 단위로 기록한다.

## 규칙

- 파일 이름: `NNNN-짧은-영문-슬러그.md`. 번호는 4자리, 단조 증가, 재사용하지 않는다.
- 한 파일에 결정 하나. 한 화면 안에 들어오게 짧게 쓴다.
- 채택된 ADR은 고치지 않는다. 결정이 바뀌면 새 ADR을 쓰고, 옛 ADR의 상태를 `대체됨(ADR-NNNN)`으로만 바꾼다.
- 오타·링크 수정은 예외로 허용한다.
- 결정이 `docs/DESIGN.md`를 바꾸면, 같은 커밋에서 DESIGN.md도 고친다.
- 주장은 코드 경로나 측정으로 뒷받침될 때만 쓴다(`dev-docs/v2-genesis.md` §3.5).

## 상태 값

| 상태 | 뜻 |
| --- | --- |
| 제안 | 검토 중. 아직 구속력 없음 |
| 채택 | 현재 유효 |
| 대체됨(ADR-NNNN) | 다른 ADR로 바뀜 |
| 폐기 | 채택하지 않음 |

## 형식

```markdown
# ADR-NNNN: 제목

- 상태: 채택
- 날짜: YYYY-MM-DD
- 관련: DESIGN.md §n, ADR-NNNN

## 맥락
무엇이 쟁점이었나.

## 결정
무엇을 정했나.

## 근거
왜 그렇게 정했나. 검토한 대안이 있으면 함께.

## 결과
이 결정으로 생기는 제약과 비용.
```

## 목록

| ADR | 제목 | 상태 | 출처 |
| --- | --- | --- | --- |
| [0001](0001-ai-friendliness-as-definition.md) | AI 친화성은 목표가 아니라 정의 | 채택 | genesis Q1 |
| [0002](0002-correctness-constraint-performance-objective.md) | 정확성은 제약, 성능은 목적함수 | 채택 | genesis Q2 |
| [0003](0003-minimize-test-surface.md) | 테스트 필요 면적의 산출·최소화 | 채택 | genesis Q3 |
| [0004](0004-design-doc-derived-not-written.md) | 설계서는 작성하지 않고 도출한다 | 채택 | genesis Q4 |
| [0005](0005-language-scope-code-and-maps.md) | 언어의 책임은 코드와 맵까지 | 채택 | genesis Q4' |
| [0006](0006-rationale-bound-to-node-hash.md) | 근거는 노드 해시에 묶어 코드 안에 | 채택 | genesis Q5 |
| [0007](0007-design-doc-tool-separate-repo.md) | 설계서 도구는 별도 리포, 마크다운 1순위 | 채택 | genesis Q6 |
| [0008](0008-standard-maps-l0-l2.md) | 표준 맵 L0~L2 고정, 추가는 하위 호환 | 채택 | genesis Q7 |
| [0009](0009-transactional-writes.md) | 쓰기는 트랜잭션, 증명은 빌드 게이트 | 채택 | genesis Q8 |
| [0010](0010-extension-maps-in-pure-bmb.md) | 확장 맵 생성기는 BMB 순수 함수 | 채택 | genesis Q9 |
| [0011](0011-storage-format-text.md) | 저장 형식은 텍스트, 배치는 도구가 | 채택 | genesis R1 |
| [0012](0012-independent-compiler-to-llvm.md) | rustc 없는 독자 컴파일러, LLVM 직접 | 채택 | genesis Q10 |
| [0013](0013-inherit-surface-strengthen-semantics.md) | 표면은 승계, 발전은 편의 포기에서 | 채택 | genesis Q11 |
| [0014](0014-deterministic-tools-ai-judges.md) | 결정론적 도구가 메우고 AI는 판단만 | 채택 | genesis Q12 |
| [0015](0015-second-class-references.md) | 소유·이동 + 2차 참조 | 채택 | genesis Q13 |
| [0016](0016-milestone-1-scope.md) | 첫 마일스톤 범위: 언어 부분집합과 도구 질의 | 채택 | genesis §6-1 |
| [0017](0017-integer-semantics-and-smt-encoding.md) | 정수 의미론과 SMT 인코딩 | 채택(자율) | 백로그 1B-01 |
| [0018](0018-obligation-isolation-and-division-facts.md) | 증명 의무 격리와 나눗셈 보조 사실 | 채택(자율) | 1B 구현 |
| [0019](0019-normal-form-and-definition-hash.md) | 정규형과 정의 해시 | 채택(자율) | 백로그 1D-01 |
| [0020](0020-map-schema.md) | 맵 JSON 스키마 L0~L2 | 채택(자율) | 백로그 1D-04 |
| [0021](0021-measurement-procedure.md) | M1 측정 절차 | 채택(자율) | 백로그 1E-01 |
| [0022](0022-mixed-integer-types-in-specs.md) | 명세에서 정수 타입 혼합 허용 | 채택(자율) | 백로그 1E-05 |
| [0023](0023-ground-instance-retry.md) | 결정 불가 의무의 기저 인스턴스 재시도 | 채택(자율) | 백로그 1B-14 |
| [0024](0024-checker-relaxations.md) | 검사기 완화 세 가지(읽기 전용 스칼라 인자, 명세의 부호 없는 부정·큰 리터럴) | 채택(승인) | 백로그 1A-13 |
| [0025](0025-task-set-m1b.md) | 두 번째 동결 과제 집합 `m1b` | 채택(자율) | 백로그 2A-01 |
| [0026](0026-data-types-surface.md) | 데이터 타입 표면: 구조체·열거형·`match`·`impl`(내장 변형 맨이름, 수신자 `self: &T`) | 채택(승인된 계획) | 백로그 2B-01·2B-11 |
| [0027](0027-data-types-encoding.md) | 데이터 타입의 증명 인코딩(구조체는 필드 분해, 열거형은 SMT 데이터타입) | 채택(승인) | 백로그 2B-02, `dev-docs/experiments/2B-02` |
| [0028](0028-integer-conversions.md) | 정수 변환과 검사·포화 산술 | 채택 | 백로그 2C-01, 2C-05 |
| [0029](0029-escape-hatches-and-rationale.md) | 탈출구(`assume`·`external_body`)와 구조화 근거 | 제안 | 백로그 2E-01 |
| [0030](0030-effects-alloc-panic-nondet.md) | 효과 확장(`alloc`·`panic`·`nondet`)과 테스트 필요 면적 | 제안 | 백로그 2F-01 |
| [0031](0031-vec-and-ownership.md) | 힙 `Vec<T>`과 소유·이동 검사 | 채택 | 백로그 2G-01, 2G-07 |
| [0032](0032-incremental-build-key.md) | 계약 방화벽 증분 빌드의 키 | 채택(자율) | 백로그 2D-10 |
| [0033](0033-per-query-spec-declarations.md) | 의무별 `spec fn` 선언(질의 머리말 축소), ADR-0032 키 갱신 | 채택(자율) | 백로그 2D-14 |
| [0034](0034-mission-goals-and-benchmarks.md) | 미션 목표와 벤치마크 체계(북극성, 목표 G1~G6, 만들 벤치마크 BVB·BRB·BNB·BSB) | 채택(승인, 목표값 잠정) | 조사 2026-10-24 |
| [0035](0035-spec-strength-kill-check.md) | 명세 강도 지표(BSB)의 판별 정의 | 채택(자율) | 백로그 2A-10 |
| [0036](0036-syntax-evaluation-scale.md) | 문법 평가 저울(기준 C1~C6, 제약 K1~K4·목적 O1~O5, 차용·확장·변형·기각 판정 규칙, 증거 기준) | 채택(자율) | 백로그 2R-01 |
| [0037](0037-bootstrap-and-language-maturity.md) | 셀프호스팅 부트스트랩 필수(stage2=stage3 고정점, stage0 기능 동결)와 언어 성숙 단계 | 채택(사람 지시) | ADR-0012 일부 대체 |
| [0038](0038-bvb-porting-rules.md) | BVB(공개 Verus 과제 이식) 규칙 | 채택(자율) | 백로그 2A-08·2A-09 |
| [0039](0039-continuous-health-management.md) | 지속 관리 체계(다섯 축 M·V·S·P·O, `health.py` 래칫, 자립 사다리 L0~L4, 실용성 규칙, `ship.sh`) | 채택(사람 지시) | 운영 문서 `dev-docs/HEALTH.md` |
| [0040](0040-premises-revised-by-pain-research.md) | 페인포인트 조사에 따른 전제 E1~E3·원칙 4 수정, 범위 밖 명시, 새 가설 H-*(반증 조건), 목표 G7 누적 검증 비용(BCB) | 채택(승인) | `dev-docs/research/` |
| [0041](0041-tradeoff-ledger.md) | 트레이드오프 원장(극대화 축·수용 비용·규칙), 테스트 비용의 두 차원(실행 최적화 대 구조적 제거), 목표 G8 테스트 코드·G9 바이너리 크기 | 채택(사람 지시) | ADR-0040 문구 보정 |
| [0042](0042-bcb-design.md) | BCB(누적 검증 비용 벤치) 설계: 계획 편집 이력, 다섯 팔의 선택 규칙, BMB 남은 테스트, 툴체인 사건 이중 측정, 기준 팔 표본화, 반증 조건 사전 등록 | 채택(자율) | 백로그 2A-14 |
| [0043](0043-caller-key-without-uses.md) | 증분 키의 피호출자 행은 `#[uses]`를 뺀 호출자 해시(호출자 질의는 피호출자의 `#[uses]`를 읽지 않는다) | 채택(승인) | ADR-0032 피호출자 행·ADR-0019 §2 일부 대체, 백로그 2D-22 |
| [0044](0044-bytes-and-byte-strings.md) | 바이트 `u8`, 바이트 문자열 리터럴, 바이트 출력과 바이트 라이브러리 | 채택 | 백로그 2S-01, 2S-03 |
| [0045](0045-judgment-criteria.md) | 판단 기준: 원인 층에서 고치고 정석을 택한다, 비용은 기준이 아니다(`dev-docs/JUDGMENT.md`) | 채택(사람 지시) | ADR-0041 "표현력 제한" 해석 |
| [0046](0046-generics-and-traits.md) | 제네릭과 trait: 정의마다 한 번 증명(정렬 매개변수), 단형화 코드, 겹치지 않는 명시적 impl | 채택(승인) | 2V-06·2V-12, ADR-0026 §1 일부 대체 |
| [0047](0047-owned-types.md) | 소유 타입 일반화: `Box<T>`, 재귀 타입, `Vec`을 담는 타입, drop glue, 구조적 `decreases` | 채택(승인) | 2V-07·2V-12, ADR-0026·0031 일부 대체 |
| [0048](0048-integer-types-and-bit-operations.md) | 정수 타입 완비, 비트 연산, 리터럴 진법, 혼합 bitvector 인코딩과 `by (bit_vector)` | 채택(승인) | 2V-08·2V-12, ADR-0017 보완 |
| [0049](0049-early-exit-and-for.md) | 조기 탈출(`return`·`break`·`continue`·`?`)과 범위 `for`·`loop` | 채택(승인) | 2V-09·2V-12 |
| [0050](0050-strings.md) | `String`·`&str`·`char`: UTF-8 불변식, 바이트 보기 | 채택(승인) | 2V-10·2V-12, ADR-0044 §5 일부 대체 |
| [0051](0051-method-call-syntax.md) | 메서드 호출 구문을 정본으로, 수신자 한 단계 조정 | 채택(승인) | 2V-11·2V-12, ADR-0026 일부 대체 |
| [0052](0052-const-items.md) | `const` 항목: 검사기가 평가하는 상수 식, 배열 길이에 사용, 증명에서 투명한 값 | 제안 | 2V-14 |
| [0053](0053-closures-and-function-values.md) | 클로저와 함수 값: 이동 캡처, `Fn` 계열 경계, Verus식 계약, 바깥으로 나가지 않는 클로저 | 제안 | 2V-16 |
| [0054](0054-scalar-reference-parameters.md) | 스칼라 참조 매개변수 `&T`·`&mut T` | 채택(자율) | 2T-08, ADR-0015·0026 범위 |
| [0055](0055-match-on-references.md) | 참조에 대한 `match`와 참조 바인딩(Rust 기본 바인딩 모드) | 채택(자율) | 2O-02, ADR-0015·0026·0047 범위 |
| [0056](0056-recursive-struct-encoding.md) | 구조체만으로 된 재귀의 증명 인코딩(`Int` 손잡이, 지상 사실) | 채택(자율) | 2O-03d, ADR-0027·0047 범위 |
| [0057](0057-canonical-counting-loops.md) | 셀 수 있는 `while`의 정규형 수정은 린트 경고(W0907)와 `bmb fix`로 | 채택(자율) | 2K-04, ADR-0049 §2 범위 |
| [0058](0058-hypothesis-first-tracks.md) | 가설 우선 트랙 재설정(T0~T5)과 수요 기반 언어 페이즈 | 채택 | 방향성 논의 2026-10-28 |
| [0059](0059-hash-references-names-as-metadata.md) | 해시 참조와 메타데이터로서의 이름(정체성 해시, 이름은 색인) | 채택 | T1-08, H-ID |
| [0060](0060-type-invariants.md) | 타입 불변식: 가시성 대신 경계에서 증명하는 자료 불변식 | 채택 | T1-09, H-ENCAP |
| [0061](0061-floating-point-f64.md) | 부동소수 `f64`: IEEE 의미, 기본 불투명 인코딩과 `by (bit_vector)` 안의 비트 정밀 증명 | 채택 | T2-01, T3 수요 |
| [0062](0062-proof-functions-and-induction.md) | 증명 함수와 귀납: Verus 꼴 `proof fn`, `proof { }` 안의 호출, 코드 생성에서 지움 | 채택 | T1-17, T1-14 실패 분석 |
| [0063](0063-canonical-query-symbols.md) | 질의 기호의 정준 이름: 정체성 해시 대신 질의 안의 위치 번호(ADR-0059 §4 기호 행) | 제안 | T1-10e |
| [0064](0064-non-recursive-spec-fn-macros.md) | 배열을 묶는 잎 `spec fn`의 정의는 양화 공리 대신 매크로(ADR-0017 §4 보완) | 채택 | T2-04 |
| [0065](0065-commutation-relation.md) | 편집의 교환 관계: 쓰기 집합, 읽기 집합, 읽는 쪽 기준의 정제 면제 | 제안 | T1-16, 협업 논의 Q1 |
| [0066](0066-measurement-model-environment.md) | 측정 모델 환경: 기본 `qwen3.8-27b`, 프런티어 교차 비교 `claude-sonnet-5-5`(정식 측정 경로, T0-03) | 채택(§3의 같은 커밋은 0070) | T0-03, ADR-0058 §2 |
| [0067](0067-macro-probe-for-obligations.md) | 증명 의무는 잎 정의를 모두 매크로로 쓴 탐침을 한도의 1%로 먼저(ADR-0064 보완) | 채택 | T2-05 |
| [0068](0068-compact-agent-view.md) | 압축 에이전트 투영(`bmb mcp --compact`): 표면은 본문 없는 정규형, 성공한 증명은 개수. 기본값 전환은 측정 뒤 승인 | 채택 | 2D-23, 2D-25 |
| [0069](0069-probe-for-lint-checks.md) | 공허 검사와 사전조건 충족 검사도 탐침을 먼저(ADR-0067 결정 4 보완) | 채택 | T2-06 |
| [0070](0070-measurement-condition.md) | 같은 측정 조건은 커밋이 아니라 모델이 보는 것과 판정으로 정한다(ADR-0066 §3 보완) | 채택 | INF-17, INF-18, T0-11 |
| [0071](0071-lean-fixed-context.md) | 고정 문맥: 도구 문서는 도구 설명 한 곳(`bmb mcp --tool-docs`), 프롬프트 언어 규칙은 진단으로 배울 수 없는 것만. 기본값 전환은 측정 뒤 승인 | 채택 | 2D-28, T0-17 |
| [0072](0072-satisfiability-probe-unfolds-non-recursive.md) | 충족 가능성 검사의 탐침은 재귀 밖의 정의를 모두 매크로로(ADR-0069 보완) | 채택 | T2-10 |

채택 계획(2026-10-24 승인): 0026은 2B 착수 때, 0028은 2C 착수 때 채택한다. 0029·0030·0031은 2단계 측정(2H) 뒤로 미룬다. "제안" ADR의 진단 코드 번호는 잠정이다. 제안끼리 겹치는 번호가 있으며(예: E0212·E0412), 채택 후 구현할 때 `compiler/src/diag.rs`에서 확정한다.
