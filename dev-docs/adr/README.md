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
| [0029](0029-escape-hatches-and-rationale.md) | 탈출구(`assume`·`external_body`)와 구조화 근거 | 채택 | 백로그 2E-01 |
| [0030](0030-effects-alloc-panic-nondet.md) | 효과 확장(`alloc`·`panic`·`nondet`)과 테스트 필요 면적 | 채택 | 백로그 2F-01 |
| [0031](0031-vec-and-ownership.md) | 힙 `Vec<T>`과 소유·이동 검사 | 채택 | 백로그 2G-01, 2G-07 |
| [0032](0032-incremental-build-key.md) | 계약 방화벽 증분 빌드의 키 | 채택(자율) | 백로그 2D-10 |
| [0033](0033-per-query-spec-declarations.md) | 의무별 `spec fn` 선언(질의 머리말 축소), ADR-0032 키 갱신 | 채택(자율) | 백로그 2D-14 |
| [0034](0034-mission-goals-and-benchmarks.md) | 미션 목표와 벤치마크 체계(북극성, 목표 G1~G6, 만들 벤치마크 BVB·BRB·BNB·BSB) | 채택(승인, 목표값은 ADR-0115로 확정) | 조사 2026-10-24 |
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
| [0052](0052-const-items.md) | `const` 항목: 검사기가 평가하는 상수 식, 배열 길이에 사용, 증명에서 투명한 값 | 채택(승인) | 2V-14 |
| [0053](0053-closures-and-function-values.md) | 클로저와 함수 값: 이동 캡처, `Fn` 계열 경계, Verus식 계약, 바깥으로 나가지 않는 클로저 | 채택(승인) | 2V-16 |
| [0054](0054-scalar-reference-parameters.md) | 스칼라 참조 매개변수 `&T`·`&mut T` | 채택(자율) | 2T-08, ADR-0015·0026 범위 |
| [0055](0055-match-on-references.md) | 참조에 대한 `match`와 참조 바인딩(Rust 기본 바인딩 모드) | 채택(자율) | 2O-02, ADR-0015·0026·0047 범위 |
| [0056](0056-recursive-struct-encoding.md) | 구조체만으로 된 재귀의 증명 인코딩(`Int` 손잡이, 지상 사실) | 채택(자율) | 2O-03d, ADR-0027·0047 범위 |
| [0057](0057-canonical-counting-loops.md) | 셀 수 있는 `while`의 정규형 수정은 린트 경고(W0907)와 `bmb fix`로 | 채택(자율) | 2K-04, ADR-0049 §2 범위 |
| [0058](0058-hypothesis-first-tracks.md) | 가설 우선 트랙 재설정(T0~T5)과 수요 기반 언어 페이즈 | 채택 | 방향성 논의 2026-10-28 |
| [0059](0059-hash-references-names-as-metadata.md) | 해시 참조와 메타데이터로서의 이름(정체성 해시, 이름은 색인) | 채택 | T1-08, H-ID |
| [0060](0060-type-invariants.md) | 타입 불변식: 가시성 대신 경계에서 증명하는 자료 불변식 | 채택 | T1-09, H-ENCAP |
| [0061](0061-floating-point-f64.md) | 부동소수 `f64`: IEEE 의미, 기본 불투명 인코딩과 `by (bit_vector)` 안의 비트 정밀 증명 | 채택 | T2-01, T3 수요 |
| [0062](0062-proof-functions-and-induction.md) | 증명 함수와 귀납: Verus 꼴 `proof fn`, `proof { }` 안의 호출, 코드 생성에서 지움 | 채택 | T1-17, T1-14 실패 분석 |
| [0063](0063-canonical-query-symbols.md) | 질의 기호의 정준 이름: 정체성 해시 대신 질의 안의 위치 번호(ADR-0059 §4 기호 행) | 채택 | T1-10e |
| [0064](0064-non-recursive-spec-fn-macros.md) | 배열을 묶는 잎 `spec fn`의 정의는 양화 공리 대신 매크로(ADR-0017 §4 보완) | 채택 | T2-04 |
| [0065](0065-commutation-relation.md) | 편집의 교환 관계: 쓰기 집합, 읽기 집합, 읽는 쪽 기준의 정제 면제 | 채택 | T1-16, 협업 논의 Q1 |
| [0066](0066-measurement-model-environment.md) | 측정 모델 환경: 기본 `qwen3.8-27b`, 프런티어 교차 비교 `claude-sonnet-5-5`(정식 측정 경로, T0-03) | 채택(§3의 같은 커밋은 0070) | T0-03, ADR-0058 §2 |
| [0067](0067-macro-probe-for-obligations.md) | 증명 의무는 잎 정의를 모두 매크로로 쓴 탐침을 한도의 1%로 먼저(ADR-0064 보완) | 채택 | T2-05 |
| [0068](0068-compact-agent-view.md) | 압축 에이전트 투영(`bmb mcp --compact`): 표면은 본문 없는 정규형, 성공한 증명은 개수. 기본값 전환은 측정 뒤 승인 | 채택 | 2D-23, 2D-25 |
| [0069](0069-probe-for-lint-checks.md) | 공허 검사와 사전조건 충족 검사도 탐침을 먼저(ADR-0067 결정 4 보완) | 채택 | T2-06 |
| [0070](0070-measurement-condition.md) | 같은 측정 조건은 커밋이 아니라 모델이 보는 것과 판정으로 정한다(ADR-0066 §3 보완) | 채택 | INF-17, INF-18, T0-11 |
| [0071](0071-lean-fixed-context.md) | 고정 문맥: 도구 문서는 도구 설명 한 곳(`bmb mcp --tool-docs`), 프롬프트 언어 규칙은 진단으로 배울 수 없는 것만. 기본값 전환은 측정 뒤 승인 | 채택 | 2D-28, T0-17 |
| [0072](0072-satisfiability-probe-unfolds-non-recursive.md) | 충족 가능성 검사의 탐침은 재귀 밖의 정의를 모두 매크로로(ADR-0069 보완) | 채택 | T2-10 |
| [0073](0073-edit-surfaces-file-diff-and-json-ast.md) | H-EDIT의 비교 표면: 파일 diff는 `git apply --recount`(`bmb mcp --file-tools`), JSON AST는 구문 트리 + JSON Patch(`--ast-tools`, `bmb ast`) | 채택 | T1-06 |
| [0074](0074-transaction-writes-every-item.md) | 트랜잭션의 `put`·`remove`는 단위의 모든 항목을 주소로: 타입 `unit::T`, 고유 `impl` 함수 `unit::T::f`(함수마다 병합), trait `unit::Trait`, trait impl `<unit::T as unit::Trait>`(통째로). `use`는 E0214. 동작은 모두에게, 새 설명은 `--item-docs`와 새 판에서 | 채택 | T0-23 |
| [0075](0075-specialized-instances-under-quantifiers.md) | 기저 인스턴스 재시도는 한정 기호 아래 비재귀 적용에 특수화 인스턴스를 더하고, 반례의 완전성은 적용 단위로(ADR-0023 보완) | 채택 | T0-24 |
| [0076](0076-visible-output-limit-and-length-continuation.md) | 세션 출력 한도는 보이는 출력(텍스트·도구 호출, ceil(bytes/4))만 세고, 출력 한도에서 끊긴 응답은 같은 차례에서 한 번 이어 쓴다(ADR-0066 §3·0070 보완, 새 측정 조건) | 채택 | T0-16, T0-27 |
| [0077](0077-repeated-runs-and-target-rule.md) | 반복 측정(qwen의 m1·m1b 5회, `run_models.py --repeat`)과 목표값 판정(실행 수준 95% t 구간, Wilson 하한, 충족·미충족·미정), 단일 실행 비교의 최소 차이(ADR-0066 §5의 선언된 예외) | 채택 | T0-12 |
| [0078](0078-supplementary-hypotheses.md) | 보완 가설 여섯의 등록(H-EFFECT, H-INTENT, H-REVIEW, H-SMALL, H-REPAIR, H-COMPAT)과 판정 기준 동결, 나머지 열넷은 등록하지 않음(사유) | 채택 | T1-42 |
| [0079](0079-exists-in-specifications.md) | 명세의 `exists|x: T| e`: `forall`과 같은 묶음·트리거·문맥 규칙, 구문 트리 `ExprKind::Quant`, 인코딩은 SMT `exists`(언어 범위 확대) | 채택 | T0-25 |
| [0080](0080-struct-literal-field-order.md) | 구조체 생성 식의 필드는 선언 순서로: 검사기가 요구하고 고친다(E0434), 정규형은 정렬하지 않는다(ADR-0026 §1·§6의 정렬 문구 대체) | 채택 | 2B-13 |
| [0081](0081-static-drop-at-branch-joins.md) | `if`·`else`와 `match`에서 한 갈래만 옮긴 값은 옮기지 않은 갈래 끝에서 정적으로 해제(ADR-0031 §2의 E0414를 그 합류점에서 대체, 플래그 없음) | 채택 | 2O-07 |
| [0082](0082-equal-types-share-an-identity.md) | 필드가 같은 두 타입은 정체성 하나(구조 정체성 유지, 도구는 경로·다중 집합으로 다룸) | 채택(자율) | T1-28 |
| [0083](0083-in-place-moves-under-type-invariants.md) | 반복 사이에 깨지는 타입 불변식: 루프 머리 유효성은 그대로, 제자리 이동은 내장 `copy_within`(권장 D) | 채택 | T1-26, T1-26a |
| [0084](0084-effect-attributes-sound-subset.md) | 효과의 LLVM 속성은 건전한 부분집합만(힙 포인터 없는 타입의 `memory`, `alloc`·피호출자 검사가 있으면 속성 없음), 구현은 속도 측정 근거가 생긴 뒤(ADR-0030 §6 대체) | 채택 | 2F-04 |
| [0085](0085-truncating-division-from-one-div-mod.md) | 0 쪽 자름 나눗셈·나머지를 같은 피연산자의 `div`·`mod` 한 쌍으로(ADR-0017 정의식 보완, 제수가 0이 아닐 때 같은 함수) | 채택 | 1B-47 |
| [0086](0086-repeats-for-bvb-and-brb.md) | qwen 반복을 BVB·BRB(원판·쓰기 표면)로 넓히고, 단일 실행 비교는 집합별 최소 차이 max(D(X), D(Y))로(ADR-0077 §2·§4 보완) | 채택 | T0-33 |
| [0088](0088-gateway-capacity-failures.md) | 게이트웨이 용량 장애(지출 한도 402, 동시 요청 한도 429)는 세션 결과가 아니다: 한도는 실행을 멈추고, 동시 세션 합 ≤ 8, 그런 세션이 섞인 실행은 통째로 무효·재실행(ADR-0066 보완) | 채택 | INF-23 |
| [0089](0089-early-ground-instance-retry.md) | 탐침이 결정하지 못한 의무는 기저 인스턴스 재시도를 탐침 몫으로 먼저(ADR-0067 순서 보완) | 채택 | 1B-46 |
| [0090](0090-bit-operands-two-complement-image.md) | 명세에서 비트 연산의 피연산자는 그 타입 폭의 2의 보수 상(ADR-0048 §4 보완) | 채택 | 2N-10 |
| [0091](0091-system-interface-and-runtime.md) | 시스템 인터페이스와 런타임: 닫힌 내장 목록(Rust의 이름, BMB 꼴의 시그니처), libc 위 IR 런타임, `io`·`nondet`, 큰 스택의 `main`, FFI 없음 | 채택 | 4C-03, 4C-01 승인 |
| [0092](0092-standard-library-core.md) | 표준 라이브러리 핵심: BMB로 쓰고 증명한 `std` 단위(텍스트·바이트, 정렬된 맵·집합, 정렬, 수 서식·해석, JSON), 계약 스타일과 증명 비용 예산 | 채택 | 4B-03, 4B-01 승인 |
| [0093](0093-prompt-caching-in-measurement-requests.md) | 측정 요청의 프롬프트 캐싱: 마지막 도구 표시와 자동 캐싱(5분), 토큰 지표는 전체 프롬프트, 캐시는 측정 조건이 아니다 | 채택 | INF-27 |
| [0094](0094-input-driven-loops.md) | 입력이 끝날 때까지 도는 루프: `nondet` 정의의 `loop`는 `decreases`를 생략할 수 있고, 부분 정확성으로 증명하며 `termination: unproved`·`nonterminating` 표식을 남긴다 | 채택 | 4C-04 |
| [0095](0095-syntax-decision-from-measurement.md) | 측정에 따른 문법 결정(변형 넷·입력 별칭 둘) | 채택 | 백로그 2R-06 |
| [0096](0096-default-proof-context-without-closure.md) | 기본 증명 문맥은 본문으로 닫지 않는다(r2): 자기 절과 피호출자 계약이 부르는 `spec fn`, 불투명성은 한 단계 | 채택(승인) | 백로그 2R-11, ADR-0095 결정 1 대체 |
| [0097](0097-inherent-impl-blocks-by-bounds.md) | 경계가 다른 고유 `impl` 블록: 경계 목록마다 블록 하나, 메서드 이름은 블록들 사이에 유일 | 채택(승인) | 백로그 2T-11, ADR-0026 일부 대체 |
| [0098](0098-specifications-read-moved-values.md) | 명세는 옮긴 값을 읽는다(옮긴 때의 값, 새 표면 없음) | 채택(승인) | 백로그 2T-12 |
| [0099](0099-spec-integer-widening.md) | 명세의 정수 넓힘: 갈래·결과는 범위가 담기면 같은 타입, 넓히는 `as`는 입력 별칭 | 채택(승인) | 백로그 2N-08, ADR-0022 확장 |
| [0100](0100-stage0-feature-freeze.md) | stage0 기능 동결: 대상은 받아들이는 언어, 감지는 코퍼스 결과 지문(`scripts/surface.py`), 예외는 사유와 함께 | 승인(선언은 구현 뒤) | 백로그 4D-01, ADR-0037 §3 |
| [0101](0101-official-write-surface.md) | 정식 측정의 쓰기 표면은 정의 단위 한 번 쓰기(`write`), 원판은 동결된 비교 기준 | 채택(승인) | 백로그 T0-13 |
| [0102](0102-lifecycle-unit-and-test-replacement-scope.md) | 효용의 판정 단위: G5 과제 단위(미충족 기록)와 수명 단위 G5-L, E3 판정 기준, 원칙 4의 범위(순수 핵심) | 채택(승인) | 종합 평가 2026-11-09 P4·P5, ADR-0034·0041 보완 |
| [0103](0103-evidence-stage-before-freeze.md) | 동결 전 증거 단계(2H-01), 순차 반복의 검정력, 주장의 기준 모델, 외부 과제, 산문 수리 제안 투자 멈춤 | 채택(승인) | 종합 평가 2026-11-09 P1·P2·P7·P8·D5, ADR-0077 확장 |
| [0104](0104-large-scale-tasks-on-the-port-twin.md) | 대규모 과업 축(H-COMPREHEND·H-REFACTOR·H-TASK)과 측정 장: 포팅 쌍둥이, 커밋 재연 | 채택(승인) | 사람 지시 2026-11-09, 백로그 3P |
| [0105](0105-stage3-scope.md) | 3단계 범위: 판정은 포팅 쌍둥이, 3B·3C·3D는 측정된 필요로 연다, 2H-01 결과별 우선순위 사전 등록 | 채택(승인) | 백로그 2H-02 |
| [0106](0106-twin-documentation-parity.md) | 포팅 쌍둥이의 문서 조건: Rust 팔은 주석 없이(`//!`만), 주석 있는 대조 조건 등록, 범위는 주석 없는 원문 | 채택(자율) | 여덟째 실행 계획 "3P 공정성", ADR-0104 결정 4 |
| [0107](0107-map-function-values.md) | 맵의 함수 값: L2 `passes` 간선, 생성 정의의 위치는 대신하는 원문 자리, L0·L1 `display` | 채택(자율) | 백로그 2V-20h, ADR-0020 §1 |
| [0108](0108-undecided-obligations-carry-a-verdict.md) | 미결정 의무가 판정 정보를 싣는다: 범위를 줄인 반례 찾기(`sat`은 확정 반례, `unsat`은 `bounded`), 응답 안의 `uses` 탐색, H-UNDECIDE 등록 | 채택(자율) | 아홉째 실행 계획, 백로그 2D-33·2D-34 |
| [0109](0109-probe-candidate-invariants.md) | 커밋하지 않는 일괄 탐침: 후보 불변식을 Houdini로 거르는 읽기 도구(`bmb probe`, MCP `probe`는 `--probe-tool`) | 채택(자율) | 아홉째 실행 계획, 백로그 2D-35 |
| [0110](0110-stateless-repair-condition.md) | 상태 없는 수리 차례: 매 요청이 새 대화(과제, 지금 상태, 시도마다 한 줄, 직전 결과)인 측정 조건, 두 팔에 같은 규칙, H-HISTORY 등록 | 채택(자율) | 아홉째 실행 계획, 백로그 T0-36·T0-37 |
| [0111](0111-decision-gates-and-hypothesis-tiers.md) | 로드맵의 판정 관문 A~E(관문 B에서 E3 판정, 결과별 조치 사전 등록), 판정 이월은 한 번, 가설 등급(존재·차별·조정)과 등급순 측정 대기열 | 채택(승인) | 사람 지시 2026-11-11, `dev-docs/reviews/roadmap-2026-11-11.md`, 백로그 2H-08 |
| [0112](0112-ab-verdict-by-the-interval-of-the-difference.md) | A/B 비교는 차이의 구간으로 판정(Welch, Newcombe 바닥, 정확 순열 p 병기), 반복은 팔당 4회 이상, 토큰은 과제별 비율 | 채택(승인) | 종합 검토 R3, 백로그 T0-39, ADR-0077·0103 보완 |
| [0113](0113-ported-components-are-fixed-in-stage0.md) | 포팅된 구성 요소는 stage0에서 고정: 변경은 두 쪽 한 커밋 + 차등 시험, S4 `accept`는 구성 요소 단위 사유 | 채택(승인) | 종합 검토 R10, 백로그 4D-09, ADR-0100 보완 |
| [0114](0114-closure-proof-context.md) | 클로저의 증명 문맥: 클로저가 적은 절의 spec fn은 `call`의 기본 문맥, 바깥 함수의 `#[uses(..)]`를 물려받음 | 채택(자율) | 백로그 2V-20k, ADR-0096·0053 |
| [0115](0115-mission-targets-confirmed.md) | 미션 목표값 확정: 값은 그대로, 기준 모델·구간 판정·변별 불가(비교군 90% 이상 또는 10% 이하) 규칙 | 채택(승인) | 백로그 2A-16, ADR-0034 |
| [0116](0116-cost-checkpoint-b0-and-stage0-shrink.md) | 포팅 비용 점검 관문 B0(P2 뒤, 실측 배율로 전체 재추정), 동결 전제에 stage0 축소 조사 | 채택(승인) | 포팅 지도 §7, 백로그 4D-11·4D-12, ADR-0111 보완 |
| [0117](0117-definition-summaries.md) | 정의의 요약 `#[summary("..")]`(함수·타입·trait·상수), `///`는 입력 꼴, 해시는 읽지 않음, 맵 L1의 `summary` 키 | 채택(승인) | 백로그 3P-08, ADR-0106 결정 5 |
| [0118](0118-twin-main-documentation-condition.md) | 쌍둥이 측정의 주 문서 조건: 정의 요약 대 Rust `///`(같은 글, Rust는 `Requires`·`Ensures` 문장만 더함), 보조 `none`, 대조 `all` | 채택(승인) | 백로그 3P-09, ADR-0106 결정 5 |
| [0119](0119-oracle-strength-by-mutants.md) | H-ORACLE의 변이체 판: BRB의 Rust 변이체 96개를 에이전트의 테스트(Rust)와 계약 + 게이트(BMB)가 죽이는 비율, 저장된 최종 소스로 계산, 판정 규칙 사전 등록 | 채택(승인) | 백로그 T0-40, 종합 검토 M5 |
| [0120](0120-tool-surfaces-follow-their-hypotheses.md) | 후보 도구 표면(JSON 구문 트리 편집, 파일 diff 편집, 일괄 탐침)은 H-EDIT·H-PROBE 판정에 따라 정리, 시한은 도구 포팅 전. 언어 표면은 그대로 동결 | 채택(승인) | 백로그 4D-11, ADR-0113·0116 보완 |

채택 계획(2026-10-24 승인): 0026은 2B 착수 때, 0028은 2C 착수 때 채택한다. 0029·0030·0031은 2단계 측정(2H) 뒤로 미룬다. "제안" ADR의 진단 코드 번호는 잠정이다. 제안끼리 겹치는 번호가 있으며(예: E0212·E0412), 채택 후 구현할 때 `compiler/src/diag.rs`에서 확정한다.
| [0121](0121-port-mirrors-stage0-data-shapes.md) | 포트의 자료 구조는 stage0과 같은 모양(소유 트리는 소유 트리로), 인덱스 구조의 불변식은 "가리키는 쪽이 뒤", 모양 변경은 원장의 측정이 요구할 때 |
| [0122](0122-port-ledger-measures.md) | 포팅 원장의 지표: 명세 줄은 기계가 가르고 배율은 실행·전체 둘, 증명된 정의는 신뢰 사실 없는 것, 작성 비용은 세션 토큰·게이트 실행·미결정으로 끝난 실행 |
| [0123](0123-solver-memory-limit.md) | 해법기 프로세스에 고정 메모리 한도(`Z3_MEMORY_MB`), 닿은 질의만 미결정(E0811), 다음 의무는 새 프로세스 |
| [0124](0124-push-length-is-a-runtime-fact.md) | `push`·`push_str`의 길이는 증명 의무가 아니라 런타임이 지키는 사실(넘으면 `abort`), E0816은 `with_capacity`에만 |
| [0125](0125-parent-agent-with-subagents.md) | 모에이전트가 서브에이전트를 관리: 관문 단위 목표, 실행으로 검증, 병렬 둘, `compiler/` 작성자는 하나, 승인 안건은 쌓고 계속 |
| [0126](0126-advisory-solver-memory-limit.md) | `lint`의 권고 질의는 따로 된 메모리 한도(`Z3_ADVISORY_MEMORY_MB`=128), 닿으면 미결정. `rlimit`은 그대로라는 결정 2는 ADR-0131이 대체 |
| [0127](0127-bounded-responses.md) | 에이전트 경계(`bmb mcp --bounded`)의 `query`·`map` 응답은 바이트 예산 안의 창: 잘림 행, `offset`·`limit`, `overview` |
| [0128](0128-review-change-set.md) | H-REVIEW 변경 집합(`bench/review`, 143개) 동결: 명세 diff 판과 코드 diff 판, 모델 리뷰어에 한한 판정, 이 집합에서 토큰 기준은 닿지 않음 |
| [0129](0129-memory-verdict-in-a-fresh-process.md) | 메모리로 인한 미결정은 새 프로세스의 첫 질의에서만 판정: 작업 프로세스는 한도의 절반, 끝난 질의는 혼자 다시 묻는다. 한도가 끝낸 질의도 재시도를 받고, 증명 캐시 키에 메모리 한도(v5) |
| [0130](0130-intent-item-set.md) | H-INTENT 항목 집합(`bench/intent`, 90개) 동결: 코드+명세 판과 명세+예시 판, 모델 리뷰어에 한한 판정, 토큰 기준은 닿지 않음, 예시의 겉모습이 라벨을 82/90 맞힘 |
| [0131](0131-advisory-rlimit.md) | `lint`의 권고 질의(첫 증명 뒤)의 전체 질의는 따로 된 `rlimit`(`Z3_ADVISORY_RLIMIT`=1,000,000), 탐침 몫과 도달 한도는 값 그대로, 닿으면 미결정. 모형을 찾는 `bmb examples`는 게이트의 `rlimit` 그대로. ADR-0126 결정 2를 대체 |
| [0132](0132-token-criteria-of-review-and-intent.md) | H-REVIEW·H-INTENT의 토큰 기준은 그대로 두고 동결한 두 집합은 탐지율만 판정. H-REVIEW의 토큰 축은 포트의 이력으로, H-INTENT의 토큰 조건은 함수 단위에서 충족 불가로 등록 |
| [0133](0133-recursive-functions-in-satisfiability-checks.md) | 탐침이 결정하지 못한 충족 검사(W0901·W0905)는 닿는 재귀 정의를 `define-funs-rec`으로 쓴 질의를 탐침의 몫으로 먼저 푼다(뜻은 정량 정의와 같음). 미결정 216 → 86, V11 −38%. `bmb examples`에는 `T1-44g`에서 |
| [0134](0134-examples-of-recursive-and-unrevealed-contracts.md) | `bmb examples`가 재귀 함수 꼴, 써 넣은 입력(배열은 값 하나), `#[uses]`와 무관한 계약의 정의로 질의. `bench/intent` 예시 재동결: `allowed` 없는 항목 12 → 0, 토큰 비 1.571 → 1.662. `lib/std` 34 → 52 |
| [0135](0135-induction-lemma-drafts.md) | 귀납 보조 정리 초안은 `bmb fix --induction`이 후보를 만들고 게이트가 보조 정리와 함수를 함께 증명한 첫 후보만 적용. 꼴은 "재귀 술어 ⇒ 같은 매개변수의 술어" 하나. `bmb verify`의 진단과 플래그 없는 `fix`는 그대로 |
| [0136](0136-ablation-sets.md) | H-UNDECIDE·H-PROBE·H-HISTORY의 절제 판 여섯(`bvb-write`·`brb-write` × `-undecided`·`-probe`·`-stateless`) 동결: 원판과 `set.json`의 한 키만 다름, 프롬프트는 그대로 |
| [0137](0137-gate-a-evidence-split.md) | 관문 A의 증거 전제를 받아들이는 언어를 바꿀 수 있는 측정으로 좁힘: `2H-01`의 남은 세 칸(`T0-08` qwen 반복, `T1-41` qwen, `2A-14e`)은 관문 B의 전제로 |
