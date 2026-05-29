# Cycle 3267: M13 Phase 3 Full — Contract Text 추출
Date: 2026-05-29

## Re-plan
Cycle 3264의 M13 Phase 3 stub을 full implementation으로 업그레이드.

## Scope & Implementation
- `bootstrap/compiler.bmb`: M13 Phase 3 full contract 추출 구현
  - `extract_pre_texts(src, rp)` → tab-separated pre condition texts
  - `extract_post_texts(src, rp)` → tab-separated post condition texts
  - `extract_intent_tok(src, rp)` → intent: "..." text
  - `rh_find_tab_or_end(s, idx)` → tab/end position for iteration
  - `emit_contract_json(texts, sb, idx)` → JSON array emission
  - `emit_fn_hint(fname, pctexts, qctexts, meta, sb, isfirst)` → 6-param JSON emitter
  - `repair_hint_scan(src, scanpos, sb, isfirst)` → main scanner
  - `repair_hint_run(input, src)` → actual JSON generation with scan
  - `json_esc_sb` + `json_esc` → JSON escaping

## Bugs Fixed
- **Root cause discovered**: `let x = e1; e2; e3;` 체인에서 `e2; e3`는 `{ }` 블록 없이 invalid.
  - `= let _sep = 0; sb_push(sb, "}"); 0;` → 파서는 `let _sep = 0; sb_push(sb, "}")` 을 함수 body로 인식, `;` 종료 후 `0`이 top-level에 등장 → 오류
  - 수정: `let _fin = sb_push(sb, "}"); 0;` 또는 `= { ... }` 블록 사용
- **Python binary write**: bootstrap 수정 시 Python text write 금지 → `'wb'` mode 필수
- **Python string escaping**: `\"` in Python string → 백슬래시 제거됨

## Verification
- cargo test 3800+2390+47+22+23 PASS ✅
- `bootstrap/compiler.exe repair-hint test_contract.bmb` → 완전한 JSON 출력 ✅
- `{"name":"add","intent":"Adds two numbers","pre":["x >= 0","y >= 0"],"post":["it >= 0"]}`
- compiler.bmb lint: 178 (177 pre-existing + 1 new [complex] repair_hint_scan)

## Reflection
- M13 Phase 1+2+3 ✅ COMPLETE
- Key lesson: BMB 표현식 체인에서 non-let expressions는 `{ }` 블록 필요

## Carry-Forward
- Actionable: M13 [complex] repair_hint_scan 허용 (21 calls, 자연스러운 복잡성)
- Next Recommendation: Final commit + remaining 3 cycles
