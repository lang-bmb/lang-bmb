# ISSUE: Track N — MCP Server (Chatter)

> **트랙**: N (MCP Server)
> **마일스톤**: M2 (AI-Ready Infrastructure)
> **현 상태**: ~50% 구현 (`ecosystem/bmb-mcp/` Chatter 별도 프로젝트)
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 4.2

## 현 상태

- ✅ `ecosystem/bmb-mcp/` (Chatter) 디렉토리 존재
- ✅ README.md 명시: "MCP server for the BMB programming language"
- ✅ docs/ROADMAP.md (서브 ROADMAP)
- ⚠️ 활성 상태/완성도/테스트 커버리지 미점검
- ⚠️ `bmb mcp` 서브명령 통합 vs 별도 도구 정책 미결

## 잔여 작업

1. **현 상태 점검 (Phase 1 — 인벤토리)**
   - `ecosystem/bmb-mcp/` 코드 정독
   - 노출 리소스 파악 (BMB 문법, 컴파일 피드백, contract verification)
   - 실제 LLM 호스트 (Claude Desktop/Code) 통합 시연

2. **통합 정책 결정 (Phase 2 — 결정)**
   - 옵션 A: `bmb mcp serve` 서브명령 (메인 컴파일러 통합)
   - 옵션 B: `bmb-mcp` 별도 패키지 유지 (현재)
   - 옵션 C: 양쪽 (코어 = 별도, edge case = 컴파일러 옵션)
   - 결정 근거: 의존성 (MCP SDK), 빌드 복잡도, 사용자 친숙도

3. **공식화 (Phase 3 — 문서/배포)**
   - 설치 가이드 (Claude Desktop config, Claude Code skill)
   - 사용 예시 (LLM이 BMB 작성 → MCP 서버가 verifier 결과 반환)
   - 배포 채널 (npm/pip/cargo)

## 완료 조건 (M2 정합)

- [ ] Chatter 인벤토리 문서 (`ecosystem/bmb-mcp/INVENTORY.md` 또는 cycle log)
- [ ] 통합 정책 결정 + ROADMAP에 기록
- [ ] 1개 host (Claude Desktop 또는 Code) 실제 통합 시연
- [ ] 사용자 가이드 작성

## 추정 사이클

3-5 cycles. Phase 1 = 1 cycle, Phase 2 = 정책 결정 (사용자 승인 가능), Phase 3 = 1-2 cycles.
