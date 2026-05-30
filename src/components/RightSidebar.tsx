// mockup: background tracking이 감지한 업무 이벤트 로그 피드

type EventKind = "comm" | "web" | "app" | "note";

interface TrackedEvent {
  time: string; // HH:MM
  text: string;
  kind: EventKind;
}

// 이벤트 종류별 색상 (점)
const KIND_COLOR: Record<EventKind, string> = {
  comm: "#e67e22", // 메신저 등 커뮤니케이션
  web: "#9b59b6", // 웹/사내 게시판
  app: "#4f8cff", // 앱 실행/포커스
  note: "#2ecc71", // 노트 변경
};

// mockup: 샘플 이벤트 (최신순)
const EVENTS: TrackedEvent[] = [
  { time: "17:42", text: "메신저 접속함", kind: "comm" },
  { time: "17:35", text: "노트 수정 — '리포트 정리' 추가", kind: "note" },
  { time: "17:12", text: "스프레드시트 on", kind: "app" },
  { time: "16:48", text: "사내 게시판 접속", kind: "web" },
  { time: "16:20", text: "노트 수정 — 체크박스 완료", kind: "note" },
  { time: "15:50", text: "VS Code 포커스", kind: "app" },
  { time: "15:05", text: "메신저 접속함", kind: "comm" },
  { time: "14:30", text: "사내 게시판 접속", kind: "web" },
];

function RightSidebar() {
  return (
    <aside className="right-sidebar">
      <div className="events-header">
        <span className="events-title">Events</span>
        <span className="tracking-status">
          <span className="tracking-dot" />
          live
        </span>
      </div>

      <ul className="event-list">
        {EVENTS.map((ev, i) => (
          <li key={`${ev.time}-${i}`} className="event-item">
            <span
              className="event-dot"
              style={{ background: KIND_COLOR[ev.kind] }}
            />
            <div className="event-body">
              <span className="event-time">{ev.time}</span>
              <span className="event-text">{ev.text}</span>
            </div>
          </li>
        ))}
      </ul>
    </aside>
  );
}

export default RightSidebar;
