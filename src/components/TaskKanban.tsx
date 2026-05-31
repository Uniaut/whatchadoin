import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Task {
  id: number;
  label: string;
  status: "Todo" | "InProgress" | "Done";
  parent_id: number | null;
  source_text: string;
  cumulative_ms: number;
  created_at: number;
}

const COLUMNS: Task["status"][] = ["Todo", "InProgress", "Done"];
const COL_LABEL: Record<Task["status"], string> = {
  Todo: "Todo",
  InProgress: "In Progress",
  Done: "Done",
};

function TaskKanban() {
  const [tasks, setTasks] = useState<Task[]>([]);

  const load = () =>
    invoke<Task[]>("get_tasks").then(setTasks).catch(console.error);

  useEffect(() => {
    load();
    let unlisten: (() => void) | undefined;
    listen("task-graph-updated", load).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const move = (id: number, status: Task["status"]) =>
    invoke("update_task_status_cmd", { taskId: id, status }).catch(console.error);

  const renderCard = (task: Task, isSub: boolean) => (
    <div key={task.id} className={isSub ? "task-card subtask" : "task-card"}>
      <span className="task-label">{task.label}</span>
      <div className="task-actions">
        {task.status !== "Todo" && (
          <button
            className="btn-status"
            onClick={() => move(task.id, task.status === "Done" ? "InProgress" : "Todo")}
          >
            {task.status === "Done" ? "← InProgress" : "← Todo"}
          </button>
        )}
        {task.status !== "Done" && (
          <button
            className="btn-status"
            onClick={() => move(task.id, task.status === "Todo" ? "InProgress" : "Done")}
          >
            {task.status === "Todo" ? "→ InProgress" : "→ Done"}
          </button>
        )}
      </div>
    </div>
  );

  return (
    <div className="task-kanban">
      {COLUMNS.map((col) => {
        const roots = tasks.filter((t) => t.parent_id === null && t.status === col);
        return (
          <div key={col} className="kanban-col">
            <h3 className="kanban-col-header">{COL_LABEL[col]}</h3>
            {roots.map((root) => (
              <div key={root.id}>
                {renderCard(root, false)}
                {tasks
                  .filter((t) => t.parent_id === root.id)
                  .map((sub) => renderCard(sub, true))}
              </div>
            ))}
          </div>
        );
      })}
    </div>
  );
}

export default TaskKanban;
