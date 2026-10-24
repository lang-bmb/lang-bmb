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
