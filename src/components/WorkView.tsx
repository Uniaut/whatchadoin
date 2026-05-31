import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export const SAMPLE_NOTE = `# 오늘의 작업

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
  note: string;
  onNoteChange: (note: string) => void;
}

function WorkView({ note, onNoteChange }: Props) {
  const noteRef = useRef(note);
  useEffect(() => { noteRef.current = note; }, [note]);
  useEffect(() => {
    const id = setInterval(() => {
      invoke("save_snapshot", { content: noteRef.current }).catch(console.error);
    }, 15_000);
    return () => clearInterval(id);
  }, []);

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
        onChange={(e) => onNoteChange(e.target.value)}
        spellCheck={false}
      />

      <div className="editor-footer">
        <span className="hint">WYSIWYG markdown editor (mockup)</span>
      </div>
    </div>
  );
}

export default WorkView;
