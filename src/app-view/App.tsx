import { useState, useEffect, useRef, useCallback } from "react";
import LeftSidebar from "../components/LeftSidebar";
import RightSidebar from "../components/RightSidebar";
import WorkView from "../components/WorkView";
import ReportView from "../components/ReportView";
import "./App.css";

export type Tab = "work" | "report";

const SAMPLE_DATES = ["2026-05-29", "2026-05-28", "2026-05-27"];
const MOCK_TASKS = ["기획서 작성", "코드 리뷰", "디자인 검토"];
const CHECK_IN_INTERVAL_MS = 60_000;

function App() {
  const [tab, setTab] = useState<Tab>("work");
  const [selectedDate, setSelectedDate] = useState<string>(SAMPLE_DATES[0]);
  const [tasks, setTasks] = useState<string[]>(MOCK_TASKS);
  const [activeTask, setActiveTask] = useState<string | null>(null);

  const tasksRef = useRef(tasks);
  const activeTaskRef = useRef(activeTask);
  useEffect(() => {
    tasksRef.current = tasks;
  }, [tasks]);
  useEffect(() => {
    activeTaskRef.current = activeTask;
  }, [activeTask]);

  const handleCheckInSubmit = useCallback((task: string) => {
    setTasks((prev) => (prev.includes(task) ? prev : [...prev, task]));
    setActiveTask(task);
  }, []);

  // Listen for submit result emitted by Rust
  useEffect(() => {
    let cancelled = false;
    let unlistenSubmit: (() => void) | null = null;

    async function setup() {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        console.log("[App] registering listen(checkin://submit)");
        const unlisten = await listen<string>("checkin://submit", (event) => {
          console.log("[App] received checkin://submit, payload:", event.payload);
          handleCheckInSubmit(event.payload);
        });
        if (cancelled) {
          unlisten();
        } else {
          unlistenSubmit = unlisten;
          console.log("[App] listen(checkin://submit) registered");
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
  }, [handleCheckInSubmit]);

  const openCheckIn = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      console.log("[App] invoking open_checkin, tasks:", tasksRef.current, "activeTask:", activeTaskRef.current);
      await invoke("open_checkin", {
        tasks: tasksRef.current,
        activeTask: activeTaskRef.current,
      });
      console.log("[App] open_checkin returned");
    } catch (e) {
      console.error("[App] open_checkin error:", e);
    }
  }, []);

  // Timer: open/show checkin window
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
        {tab === "work" ? <WorkView tasks={tasks} activeTask={activeTask} /> : <ReportView date={selectedDate} />}
      </main>

      <RightSidebar />

      <button className="debug-checkin-btn" onClick={openCheckIn}>
        체크인
      </button>
    </div>
  );
}

export default App;
