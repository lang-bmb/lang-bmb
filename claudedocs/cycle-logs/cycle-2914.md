## Cycle 2914 완료: GPUStack B축 측정 (qwen3.6-35b-a3b)

### 개발 범위
- `.env.local` GPUStack 설정을 bmb-ai-bench에 자동 연동
- Qwen3 thinking mode 비활성화 + max_tokens 확장
- B축 전체 100문제 × 3 runs 측정 (qwen3.6-35b-a3b)

### 변경 파일
- `ecosystem/bmb-ai-bench/bmb_ai_bench/run_cmd.py` — GPUSTACK_* 폴백 + GPUStack 자동 설정
- `ecosystem/bmb-ai-bench/bmb_ai_bench/runner/llm_client.py` — extra_body 파라미터 추가
- `claudedocs/measurements/b_baseline_2026-05-18_c2914_qwen3.json` — 측정 결과 저장

### 측정 결과

| 모델 | Success Rate | Median Loops | 측정일 |
|------|-------------|-------------|--------|
| claude-sonnet-4-6 (공식 baseline) | **98.0%** | 1 | 2026-05-13 |
| qwen3.6-35b-a3b (GPUStack) | **85.0%** | 1 | 2026-05-18 |

- total_runs: 300 (100문제 × 3 runs)
- passed: 255
- Always FAIL: 11문제 (contract 2 + performance 3 + edge 1 + integration 1 + algorithm 2 + system 1 + contract 1)
- Sometimes FAIL (비결정적): 8문제

### 주요 발견

1. **Qwen3 thinking mode 문제**: 기본 설정으로는 `// [TRUNCATED]` 코드가 컴파일되어 `@bmb_user_main undefined` 링커 오류 반복. `chat_template_kwargs.enable_thinking=false` 필수.
2. **max_tokens 확장**: 4096 → 16384 (복잡한 문제에서도 코드 완성 보장)
3. **GPUStack 자동 감지**: `GPUSTACK_ENDPOINT` env var 감지 시 자동 설정 적용. `.env.local`만 있으면 별도 설정 불필요.
4. **Claude 대비 13pp 차이**: 35B MoE 로컬 모델 vs Claude Sonnet 4.6 (frontier). BMB reference로 85% 달성은 로컬 모델 기준 양호한 성능.

### 현재 상태
- 테스트: 변경 없음 (2388 tests)
- Bootstrap: 변경 없음 (S2==S3 유지)
- 벤치마크: B축 신규 데이터포인트 추가

### 미비/결함/개선 도출

| 유형 | 내용 | 심각도 |
|------|------|--------|
| 개선 | Always FAIL 11문제 분석 → BMB reference 개선 여지 탐색 | P3 |
| 개선 | 비결정적 FAIL 8문제 → `--runs 5` 재측정으로 정확한 통과율 파악 | P4 |
| 개선 | API timeout (99_bounded_queue_contract run1) → timeout 증가 고려 | P4 |

### 후속 단계
1. Always FAIL 문제 중 BMB reference 개선으로 해결 가능한 것 분석
2. Claude baseline 재측정 (stale 기한: 2026-08-13, 아직 유효)
3. 언어 갭 추가 해소 계속
