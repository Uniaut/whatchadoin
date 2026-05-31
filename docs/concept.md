# Whatchadoin - 업무 밀도 트래커

할 일을 할 때, 내가 정말 일처리를 밀도있게 하고 있는 것인지 - 아니면 화면만 띄워두고 멍때리거나 파악하는 시늉만 하는 것인지 명확하지 않아보일 때가 있다.
업무 시간동안 작업의 진행 상황 및 속도를 파악하고, self피드백을 위해 진행된 작업 내용에 대한 분석 리포트를 받고자 한다.

----------
phase 1 - 직접 입력 + AI 에이전트 파이프라인

[개요]

사용자는 자유형 마크다운 노트를 작성한다. 구조에 맞출 필요 없다.
WYSIWYG markdown 에디터의 형태를 지향한다.

15s마다 노트 스냅샷을 찍어 before/after diff를 추출한다.
AI 다중 에이전트 파이프라인이 diff를 읽어 내부 task graph를 유지한다.

[AI 에이전트 파이프라인]

Agent 1 (Diff Parser): raw diff → 의미있는 변경 항목 추출 (들여쓰기 레벨 포함)
Agent 2 (Task Mapper): 변경 항목 → 기존 task 매핑 or 신규 task 생성, parent-child 관계 설정
Agent 3 (Pattern Classifier): task 변경 패턴 분류
  - EXPANSION: 기존 task에 같은 레벨 항목 추가 (범위 확장)
  - DECOMPOSITION: 기존 task 아래 subtask 생성 (업무 분해)
  - STATUS_CHANGE: 체크박스 상태 전환
  - NEW_TASK: 독립 신규 task

task graph 스키마: id, label, parent_id, status, created_at, cumulative_ms, timeline_memos[]
노트는 사용자 영역. task graph는 AI 내부 영역 (사용자가 직접 편집 불가).

[리포트 컨셉]
칸반보드와 insight 섹션으로 나뉜다.
칸반보드는 task graph 기반으로 렌더링된다. 최상위 task = 카드, subtask = 체크리스트.
칸반보드는 언제 어떤 업무에 집중하였는지 track하기 위해 존재하며, tracker가 시간의 분절 없이 진행했는지를 캐치한다.
right sidebar는 PatternEvent 실시간 스트림 (확장/분해/상태변경 이벤트 로그).

----------
phase 2 - tracking screenshot

TO BE UPDATED

----------
phase 3 - tracking mouse & keyboard

TO BE UPDATED
