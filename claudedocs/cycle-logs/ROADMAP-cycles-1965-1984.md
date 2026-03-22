# Roadmap: Cycles 1965-1984 — Dogfooding 기반 라이브러리 확장 + 컴파일러 개선
Date: 2026-03-22

## 목표
BMB 바인딩 라이브러리를 실전 수준으로 확장하고, dogfooding 과정에서 발견되는 컴파일러 한계를 해결한다.

## Phase 1: bmb-crypto 확장 (Cycles 1965-1968)
- gotgan-packages의 bmb-md5, bmb-base32, bmb-crc32 통합
- HMAC-SHA256 신규 구현
- Python 바인딩 + 벤치마크 vs hashlib

## Phase 2: bmb-algo 확장 (Cycles 1969-1972)
- gotgan-packages에서 추가 알고리즘 통합
- 정렬/검색/그래프 알고리즘 추가
- 벤치마크 vs scipy/networkx

## Phase 3: bmb-text 라이브러리 (Cycles 1973-1976)
- bmb-string-algo, bmb-memchr, bmb-trie, bmb-tokenizer 통합
- Python 바인딩 + 벤치마크

## Phase 4: Bootstrap @export + 컴파일러 (Cycles 1977-1980)
- Bootstrap compiler에 @export, dllexport, precondition 포팅
- 3-Stage 검증
- dogfooding에서 발견된 컴파일러 버그 수정

## Phase 5: 품질 + 패키징 (Cycles 1981-1984)
- Python 패키지 구조 개선 (setup.py, wheel)
- 벤치마크 자동화 스크립트
- 크로스 플랫폼 빌드 검증
- 문서 + README
