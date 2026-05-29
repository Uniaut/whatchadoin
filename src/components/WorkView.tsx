import { useState } from "react";

// mockup: 실제 WYSIWYG 마크다운 에디터 대신 styled textarea로 표현
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

function WorkView() {
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
