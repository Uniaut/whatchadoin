interface Props {
  date: string;
}

// mockup: 업무 시간 범위 (09:00 ~ 18:00)
const START_HOUR = 9;
const END_HOUR = 18;
const SPAN = END_HOUR - START_HOUR;

interface GanttTask {
  label: string;
  start: number; // 소수 시각 (예: 10.5 = 10:30)
  end: number;
  color: string;
}

// mockup: 샘플 작업 타임라인 (중간 빈 구간 = 추적 공백)
const TASKS: GanttTask[] = [
  { label: "환경 셋업", start: 9, end: 10.5, color: "#4f8cff" },
  { label: "mockup 레이아웃", start: 10.5, end: 12, color: "#2ecc71" },
  { label: "칸반 → 간트 변경", start: 13, end: 14.5, color: "#e67e22" },
  { label: "tracking 스키마", start: 15, end: 16.5, color: "#9b59b6" },
  { label: "리포트 정리", start: 16.5, end: 18, color: "#e74c3c" },
];

const HOURS = Array.from({ length: SPAN + 1 }, (_, i) => START_HOUR + i);

// 시각 → 트랙 내 좌측 위치(%)
const toPct = (time: number) => ((time - START_HOUR) / SPAN) * 100;
const fmtHour = (h: number) => `${String(h).padStart(2, "0")}:00`;
const fmtTime = (t: number) => {
  const h = Math.floor(t);
  const m = Math.round((t - h) * 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
};

function ReportView({ date }: Props) {
  return (
    <article className="report-view">
      <header className="report-header">
        <h1>Report</h1>
        <time className="report-date">{date}</time>
      </header>

      <section className="report-section">
        <h2>Timeline</h2>

        <figure className="gantt">
          {/* 시간축 */}
          <div className="gantt-row gantt-axis">
            <div className="gantt-label" />
            <div className="gantt-track gantt-axis-track">
              {HOURS.map((h) => (
                <span
                  key={h}
                  className="gantt-tick"
                  style={{ left: `${toPct(h)}%` }}
                >
                  {fmtHour(h)}
                </span>
              ))}
            </div>
          </div>

          {/* 작업 막대 */}
          {TASKS.map((task) => (
            <div key={task.label} className="gantt-row">
              <div className="gantt-label">{task.label}</div>
              <div className="gantt-track">
                <div
                  className="gantt-bar"
                  style={{
                    left: `${toPct(task.start)}%`,
                    width: `${toPct(task.end) - toPct(task.start)}%`,
                    background: task.color,
                  }}
                  title={`${fmtTime(task.start)} – ${fmtTime(task.end)}`}
                >
                  <span className="gantt-bar-time">
                    {fmtTime(task.start)}–{fmtTime(task.end)}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </figure>

        <p className="hint">빈 구간 = 추적 공백 (집중 분절)</p>
      </section>

      <section className="report-section">
        <h2>Insights</h2>
        <div className="insights-placeholder">
          <p>작업 분석 리포트가 여기에 표시됩니다.</p>
          <p className="hint">
            언제 어떤 업무에 집중했는지, 시간의 분절 없이 진행했는지 분석합니다.
          </p>
        </div>
      </section>
    </article>
  );
}

export default ReportView;
