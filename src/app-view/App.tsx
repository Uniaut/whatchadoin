import { useState, useEffect, useCallback } from "react";
import LeftSidebar from "../components/LeftSidebar";
import RightSidebar from "../components/RightSidebar";
import WorkView, { SAMPLE_NOTE } from "../components/WorkView";
import TaskKanban from "../components/TaskKanban";
import ReportView from "../components/ReportView";
import "./App.css";

export type Tab = "work" | "report";

const SAMPLE_DATES = ["2026-05-29", "2026-05-28", "2026-05-27"];
const CHECK_IN_INTERVAL_MS = 60_000;

function App() {
  const [tab, setTab] = useState<Tab>("work");
  const [selectedDate, setSelectedDate] = useState<string>(SAMPLE_DATES[0]);
  const [note, setNote] = useState(SAMPLE_NOTE);

  useEffect(() => {
    let cancelled = false;
    let unlistenSubmit: (() => void) | null = null;

    async function setup() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        const unlisten = await listen<string>("checkin://submit", (event) => {
          console.log("[App] checkin submit:", event.payload);
        });
        if (cancelled) {
          unlisten();
        } else {
          unlistenSubmit = unlisten;
        }
      } catch (e) {
        console.error("[App] listen setup failed:", e);
      }
    }

    setup();
    return () => {
      cancelled = true;
      unlistenSubmit?.();
    };
  }, []);

  const openCheckIn = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("open_checkin", { tasks: [], activeTask: null });
    } catch (e) {
      console.error("[App] open_checkin error:", e);
    }
  }, []);

  useEffect(() => {
    const id = setInterval(openCheckIn, CHECK_IN_INTERVAL_MS);
    return () => clearInterval(id);
  }, [openCheckIn]);

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
        {tab === "work" ? (
          <div className="work-layout">
            <WorkView note={note} onNoteChange={setNote} />
            <TaskKanban />
          </div>
        ) : (
          <ReportView date={selectedDate} />
        )}
      </main>

      <RightSidebar />

      <button className="debug-checkin-btn" onClick={openCheckIn}>
        체크인
      </button>
    </div>
  );
}

export default App;
