import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./app-view/App.css";
import "./CheckInWindow.css";

interface CheckInData {
  tasks: string[];
  active_task: string | null;
}

function CheckInWindow() {
  const [data, setData] = useState<CheckInData | null>(null);
  const [selected, setSelected] = useState<string>("");
  const [newTask, setNewTask] = useState("");
  const [showNewTaskInput, setShowNewTaskInput] = useState(false);

  const applyCheckInData = useCallback((d: CheckInData) => {
    setData(d);
    setSelected(d.active_task ?? "");
    setNewTask("");
    setShowNewTaskInput(d.active_task === null && d.tasks.length === 0);
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlistenDataUpdated: (() => void) | null = null;

    async function setup() {
      try {
        console.log("[CheckInWindow] mounted, calling get_checkin_data");
        const d = await invoke<CheckInData>("get_checkin_data");
        console.log("[CheckInWindow] get_checkin_data returned:", d);
        if (!cancelled) {
          applyCheckInData(d);
        }
      } catch (e) {
        console.error("[CheckInWindow] get_checkin_data error:", e);
      }

      try {
        const { listen } = await import("@tauri-apps/api/event");
        const unlisten = await listen<CheckInData>("checkin://data-updated", (event) => {
          console.log("[CheckInWindow] received checkin://data-updated:", event.payload);
          applyCheckInData(event.payload);
        });
        if (cancelled) {
          unlisten();
        } else {
          unlistenDataUpdated = unlisten;
        }
      } catch (e) {
        console.error("[CheckInWindow] data update listener setup failed:", e);
      }
    }

    setup();
    return () => {
      cancelled = true;
      unlistenDataUpdated?.();
    };
  }, [applyCheckInData]);

  async function handleSubmit() {
    const task = showNewTaskInput ? newTask.trim() : selected;
    if (!task) return;

    console.log("[CheckInWindow] invoking submit_checkin, task:", task);
    try {
      await invoke("submit_checkin", { task });
      console.log("[CheckInWindow] submit_checkin returned");
      await getCurrentWindow().hide();
    } catch (e) {
      console.error("[CheckInWindow] submit_checkin error:", e);
    }
  }

  if (!data) {
    return <div className="checkin-loading">로딩 중...</div>;
  }

  const submitDisabled = showNewTaskInput ? !newTask.trim() : !selected;

  return (
    <div className="checkin-window">
      <p className="checkin-title">지금 뭐 하고 있어?</p>

      {!showNewTaskInput ? (
        <div className="checkin-task-list">
          {data.tasks.map((task) => (
            <button
              key={task}
              className={`checkin-task-btn${selected === task ? " selected" : ""}`}
              onClick={() => setSelected(task)}
            >
              {task}
            </button>
          ))}
        </div>
      ) : (
        <input
          className="checkin-input"
          type="text"
          placeholder="새 작업 입력..."
          value={newTask}
          onChange={(e) => setNewTask(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
          autoFocus
        />
      )}

      <div className="checkin-actions">
        {!showNewTaskInput ? (
          <button
            className="checkin-new-btn"
            onClick={() => {
              setShowNewTaskInput(true);
              setNewTask("");
            }}
          >
            새로운 작업
          </button>
        ) : data.tasks.length > 0 ? (
          <button
            className="checkin-new-btn"
            onClick={() => setShowNewTaskInput(false)}
          >
            뒤로
          </button>
        ) : null}
        <button
          className="checkin-submit-btn"
          onClick={handleSubmit}
          disabled={submitDisabled}
        >
          제출
        </button>
      </div>
    </div>
  );
}

export default CheckInWindow;
