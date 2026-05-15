<h1 align="center">claude-dejavu</h1>

<p align="center">
  <strong>Claude가 자기 실수를 기억합니다.</strong>
  <br />
  세션 로그에서 안티패턴을 감지하고 CLAUDE.md를 자동으로 패치합니다.
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README.ko.md">한국어</a>
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/claude-dejavu"><img src="https://img.shields.io/npm/v/claude-dejavu" alt="npm version" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License" /></a>
  <a href="https://github.com/cmg8431/claude-dejavu"><img src="https://img.shields.io/github/stars/cmg8431/claude-dejavu?style=social" alt="GitHub Stars" /></a>
</p>

---

## 문제

Claude는 세션마다 같은 실수를 반복합니다. pnpm 프로젝트에서 `npm install`을 또 실행하고, 방금 고친 코드를 또 되돌리고, 스타일 취향을 또 무시합니다.

세션은 매번 제로에서 시작합니다. 과거 실패에 대한 기억이 없습니다.

## 해결책

**dejavu**는 Claude Code 세션을 백그라운드에서 감시합니다. 반복되는 안티패턴을 감지하면 `CLAUDE.md`에 룰을 작성합니다 — Claude가 세션 시작 시 항상 읽는 바로 그 파일입니다.

```
Day 1:  Claude가 `npm install` 실행 → 실패 → 당신이 고침
Day 3:  Claude가 `npm install` 실행 → 실패 → 당신이 또 고침
Day 5:  Claude가 `npm install` 실행 → 실패
        ↓
        dejavu: "이 프로젝트는 pnpm을 사용합니다." → CLAUDE.md
        ↓
Day 6:  Claude가 CLAUDE.md 읽음 → `pnpm install` 실행 → 성공
```

루프가 스스로 닫힙니다.

## 설치

```bash
npm install -g claude-dejavu
claude-dejavu install
```

이게 전부입니다. 두 줄이면 끝. `install` 명령이:

1. GitHub Release에서 플랫폼별 Rust 바이너리 다운로드
2. SQLite 데이터베이스 초기화
3. `~/.claude/settings.local.json`에 훅 자동 등록
4. 이후 모든 것이 자동

## 작동 방식

```
   [세션 로그]                [패턴 감지]              [룰 제안]
   .jsonl 파일    ──────▶   5개 디텍터    ──▶  CLAUDE.md 패치
        ▲                                          │
        │              [효과 추적]                    │
        └──────  fire 카운팅  ◀───────────────────┘
```

1. **수집** — 훅이 툴 사용, 에러, 사용자 정정을 실시간 캡처
2. **감지** — Rust 디텍터가 세션 간 패턴 발견
3. **학습** — 높은 신뢰도 패턴이 CLAUDE.md 룰이 됨
4. **추적** — 룰 fire 횟수 기록, 죽은 룰 정리 제안

모든 처리는 로컬에서 수행됩니다. API 호출, 클라우드, 텔레메트리 없음.

## 디텍터

### ① 되돌림 순환 (Revert Cycle)

Claude가 파일을 편집하고, 누군가 되돌리고, Claude가 같은 방식으로 다시 편집합니다.

### ② 반복 에러 (Repeated Error)

같은 에러가 세션마다 계속 나타나고, 매번 같은 수정이 뒤따릅니다.

### ③ 조용한 수정 (Silent Fix)

킬러 피처. Claude가 작업을 끝내면, 사용자가 **말 없이 같은 파일을 수정**합니다. 사용자가 말이 아닌 행동으로 Claude를 정정하는 것입니다.

이것이 dejavu를 다른 모든 것과 구분 짓습니다. 대부분의 도구는 **명시적** 정정에서만 학습합니다. dejavu는 **행동**에서 학습합니다.

### ④ 사용자 정정 (User Correction)

정규식 기반 명시적 정정 감지. 영어, 한국어, 일본어 지원.

```
"no, use pnpm not npm"       → 룰: Use pnpm, not npm.
"npm 말고 pnpm 쓰세요"        → 룰: Use pnpm, not npm.
"覚えて: pnpmを使うこと"       → 룰: Use pnpm, not npm.
```

### ⑤ 긴 Bash 세션 (Long Bash Session)

Claude가 과도한 Bash 호출로 디버깅 늪에 빠진 것을 감지합니다.

## 명령어

| 명령어 | 설명 |
|--------|------|
| `claude-dejavu install` | 설치: DB 생성 + 훅 등록 |
| `claude-dejavu uninstall` | 훅 제거 (학습된 룰은 보존) |
| `claude-dejavu scan` | 대화형 스캔: 패턴 감지, 룰 승인 [y/n/edit] |
| `claude-dejavu list` | 품질 등급(A/B/C/D)과 함께 룰 목록 표시 |
| `claude-dejavu stats` | 효과 통계 |
| `claude-dejavu watch` | 데몬 모드: 30초마다 자동 스캔 |
| `claude-dejavu cleanup` | 죽은 룰 찾기 (14일간 fire 없음) |
| `claude-dejavu ui` | 웹 대시보드 (localhost:7777) |

## 슬래시 커맨드

| 커맨드 | 설명 |
|--------|------|
| `/dejavu` | 제안된 룰 검토 및 승인/거부 |
| `/dejavu-status` | 빠른 상태 확인 |
| `/dejavu-scan` | 수동 패턴 스캔 |
| `/dejavu-report` | 주간 효과 보고서 |
| `/how-it-works` | dejavu 작동 원리 설명 |

## 웹 대시보드

```bash
claude-dejavu ui
```

`http://localhost:7777`에서 열립니다. 따뜻한 다크 테마.

- **Overview** — 통계 그리드 + 패턴/룰 피드
- **Rules** — 품질 등급, 신뢰도 바, fire 횟수 테이블
- **Console** — 레벨/컴포넌트 필터가 있는 로그 뷰어

## 설정

선택사항. `~/.config/claude-dejavu/config.toml` 생성:

```toml
confidence_threshold = 0.5     # 최소 신뢰도
dead_rule_days = 14            # 죽은 룰 판정 일수
dashboard_port = 7777          # 대시보드 포트
excluded_paths = ["**/node_modules/**", "*.env"]  # 제외 경로
```

## 비교

| | claude-reflect | claude-mem | **claude-dejavu** |
|---|---|---|---|
| 트리거 | 수동 (`/reflect`) | 자동 | **자동** |
| 학습 소스 | 사용자 정정만 | 기록만 | **정정 + 행동 패턴** |
| 출력 | CLAUDE.md | SQLite | **CLAUDE.md + 효과 추적** |
| 대시보드 | 없음 | 피드 뷰어 | **피드 + 룰 + 콘솔** |
| 조용한 수정 감지 | 아니오 | 아니오 | **예** |
| 다국어 | 부분 | 없음 | **한/영/일** |

## 기여

[CONTRIBUTING.md](CONTRIBUTING.md)를 참고해주세요.

## 라이선스

MIT

---

<p align="center">
  <sub>Claude는 같은 실수를 반복합니다. 이제 기억합니다.</sub>
</p>
