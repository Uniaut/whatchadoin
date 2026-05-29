import { useState } from "react";
import LeftSidebar from "./components/LeftSidebar";
import WorkView from "./components/WorkView";
import ReportView from "./components/ReportView";
import "./App.css";

export type Tab = "work" | "report";

// mockup: 샘플 리포트 날짜 목록 (최신순)
const SAMPLE_DATES = ["2026-05-29", "2026-05-28", "2026-05-27"];

function App() {
  const [tab, setTab] = useState<Tab>("work");
  const [selectedDate, setSelectedDate] = useState<string>(SAMPLE_DATES[0]);

  return (
    <div className="layout">
      <LeftSidebar
        tab={tab}
        onTabChange={setTab}
        dates={SAMPLE_DATES}
        selectedDate={selectedDate}
        onSelectDate={setSelectedDate}
      />

      <main className="main">
        {tab === "work" ? <WorkView /> : <ReportView date={selectedDate} />}
      </main>

      <aside className="right-sidebar">
        <span className="placeholder-text">Placeholder</span>
      </aside>
    </div>
  );
}

export default App;
