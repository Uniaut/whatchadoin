# Agent Pipeline Task Tracking Design

**Date:** 2026-05-31
**Status:** Approved

## Overview

사용자는 자유형 마크다운 노트를 작성한다. AI 다중 에이전트 파이프라인이 노트 diff를 읽어 내부 task graph를 유지한다. 사용자는 구조에 맞출 필요 없이 노트만 쓰면 되고, 구조화 데이터는 AI가 추출한다.

## 커버 유즈케이스

- **업무 확장 (Task Expansion):** 진행 중 기존 task에 새 항목이 추가될 때 감지
- **업무 분해 (Task Decomposition):** 단일 task가 들여쓰기 subtask로 나뉠 때 감지

## 아키텍처

```
[User] 자유형 마크다운 노트 작성/편집
         │
         │ 15s 스냅샷
         ▼
[Collection] before/after diff 추출
         │
         ▼
[Agent 1: Diff Parser]
  입력: raw text diff
  출력: ParsedChange[] (추가/삭제/수정, 들여쓰기 레벨 포함)
         │
         ▼
[Agent 2: Task Mapper]
  입력: ParsedChange[] + 현재 task graph
  출력: TaskGraphCommand[] (CreateTask | UpdateTask | AddMemo)
         │
         ▼
[Agent 3: Pattern Classifier]
  입력: TaskGraphCommand[] + 변경 전후 task 상태
  출력: PatternEvent[] (Expansion | Decomposition | StatusChange | NewTask)
         │
         ▼
[Task Graph Store]
         │
         ├─► Kanban / Mainboard
         ├─► Right Sidebar 이벤트 스트림
         └─► Report 세션 분석
```

## 데이터 모델

### Task

```rust
Task {
  id: Uuid,
  label: String,              // AI가 추출한 task명
  parent_id: Option<Uuid>,    // 분해 계층
  status: TaskStatus,         // Todo | InProgress | Done
  created_at: DateTime,
  cumulative_ms: u64,         // 누적 작업 시간
  timeline_memos: Vec<TimelineMemo>,
  source_text: String,        // 원본 노트 텍스트
}
```

### TimelineMemo

```rust
TimelineMemo {
  timestamp: DateTime,
  event_type: EventType,      // Expansion | Decomposition | StatusChange | NewTask
  description: String,        // AI 생성 설명
}
```

### Agent 인터페이스

```rust
// Agent 1 출력
ParsedChange {
  change_type: Added | Removed | Modified,
  text: String,
  context: String,            // 전후 3줄
}

// Agent 2 출력
TaskGraphCommand {
  CreateTask { label, parent_id, source_text },
  UpdateTask { id, status?, parent_id? },
  AddMemo   { task_id, description },
}

// Agent 3 출력
PatternEvent {
  event_type: EventType,
  task_id: Uuid,
  affected_ids: Vec<Uuid>,
}
```

### NoteSnapshot

```rust
NoteSnapshot {
  captured_at: DateTime,
  content: String,
}
```

## 에이전트 프롬프트 전략

### Agent 1 — Diff Parser

- 체크박스 상태 변경 `[ ]` → `[x]` 를 별도 항목으로 추출
- 들여쓰기 레벨 보존 (계층 파악용)
- 50자 미만 diff는 컨텍스트(전후 3줄) 포함
- 공백/줄바꿈 변경 무시

### Agent 2 — Task Mapper

- 기존 task 매핑 우선 (텍스트 유사도 + 들여쓰기 위치)
- 부모 추론: 들여쓰기 상위 항목 = parent_id
- 신규 task 생성 조건: 매핑 신뢰도 < 0.7
- 구조화 출력 강제 (JSON schema)

### Agent 3 — Pattern Classifier

| 이벤트 | 분류 기준 |
|--------|-----------|
| EXPANSION | 기존 task에 같은 레벨 새 항목 추가 |
| DECOMPOSITION | 기존 task 아래 들여쓰기 증가한 하위 항목 생성 |
| STATUS_CHANGE | `[ ]` ↔ `[x]` 상태 전환 |
| NEW_TASK | 매핑 없이 신규 생성된 최상위 항목 |

## UI 연동

### Kanban / Mainboard

- 최상위 task = 칸반 카드
- subtask = 카드 내 체크리스트
- status별 컬럼 (Todo | In Progress | Done)

### Right Sidebar (이벤트 스트림)

PatternEvent 실시간 표시:
- `"10:32 — '이슈 3035' 아래 subtask 3개로 분해됨"`
- `"11:15 — 'API 구현' 범위 확장 감지 (2→5개 항목)"`

### Report

- 세션별 task 타임라인 (칸반보드)
- EXPANSION / DECOMPOSITION 이벤트 비율
- cumulative_ms 기반 업무 밀도 측정

## 구현 순서 (점진적)

1. NoteSnapshot + diff 추출 (Collection 구현)
2. Task 데이터 모델 + 저장소 (core-shared)
3. Agent 1 + Agent 2 (Diff Parser + Task Mapper)
4. Kanban UI (task graph 연동)
5. Agent 3 (Pattern Classifier) + Right Sidebar 이벤트
6. Report 세션 분석
