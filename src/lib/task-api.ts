import { invoke } from "@tauri-apps/api/core";
import { normalizeTaskStore, type Task, type TaskStore } from "./tasks";

export async function loadTaskStore() {
  const store = await invoke<TaskStore>("load_tasks");
  return normalizeTaskStore(store);
}

export async function saveTaskStore(tasks: Task[]) {
  const store = await invoke<TaskStore>("save_tasks", { tasks });
  return normalizeTaskStore(store);
}
