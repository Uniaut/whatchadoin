import { useState } from "react";

const SAMPLE_NOTE = `# 오늘의 작업

## 진행 중
- [ ] 칸반 리포트 레이아웃 구현
- [ ] tracking 이벤트 스키마 정의

## 완료
- [x] Tauri 개발 환경 셋업
- [x] mockup 브랜치 생성

## 메모
화면만 띄워두지 말고 실제로 밀도있게...
`;

interface Props {
  tasks: string[];
  activeTask: string | null;
}

function WorkView({ tasks, activeTask }: Props) {
  const [note, setNote] = useState(SAMPLE_NOTE);

  return (
    <div className="work-view">
      <div className="editor-toolbar">
        <span className="editor-title">Note</span>
        <span className="tracking-status">
          <span className="tracking-dot" />
          15s마다 변경 추적 중
        </span>
      </div>

      {tasks.length > 0 && (
        <div className="task-bar">
          {tasks.map((task) => (
            <span
              key={task}
              className={`task-chip${task === activeTask ? " active" : ""}`}
            >
              {task}
            </span>
          ))}
        </div>
      )}

      <textarea
        className="markdown-editor"
        value={note}
        onChange={(e) => setNote(e.target.value)}
        spellCheck={false}
      />

      <div className="editor-footer">
        <span className="hint">WYSIWYG markdown editor (mockup)</span>
      </div>
    </div>
  );
}

export default WorkView;
