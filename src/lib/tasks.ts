export type Task = {
  id: string;
  title: string;
  note: string;
  dueDate: string | null;
  done: boolean;
  createdAt: string;
  updatedAt: string;
};

export type TaskStore = {
  version: number;
  tasks: Task[];
};

export type TaskDraft = {
  title: string;
  note: string;
  dueDate: string;
};

export const TASK_STORE_VERSION = 1;

function pad(value: number) {
  return String(value).padStart(2, "0");
}

export function toDateInputValue(date = new Date()) {
  return [
    date.getFullYear(),
    pad(date.getMonth() + 1),
    pad(date.getDate()),
  ].join("-");
}

function addDays(date: Date, days: number) {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
}

export function isValidDueDate(value: string | null | undefined) {
  if (!value) {
    return true;
  }

  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) {
    return false;
  }

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);

  if (!Number.isInteger(year) || year <= 0) {
    return false;
  }

  if (!Number.isInteger(month) || month < 1 || month > 12) {
    return false;
  }

  const maxDay = new Date(year, month, 0).getDate();
  return Number.isInteger(day) && day >= 1 && day <= maxDay;
}

export function formatDueDate(value: string | null | undefined, now = new Date()) {
  if (!value) {
    return "无日期";
  }

  const today = toDateInputValue(now);
  const tomorrow = toDateInputValue(addDays(now, 1));
  const yesterday = toDateInputValue(addDays(now, -1));

  if (value === today) {
    return "今天";
  }

  if (value === tomorrow) {
    return "明天";
  }

  if (value === yesterday) {
    return "昨天";
  }

  return value;
}

export function sortTasks(tasks: Task[]) {
  return tasks
    .map((task, index) => ({ task, index }))
    .sort((left, right) => {
      if (left.task.done !== right.task.done) {
        return Number(left.task.done) - Number(right.task.done);
      }

      return left.index - right.index;
    })
    .map(({ task }) => task);
}

export function normalizeTaskStore(store: Partial<TaskStore> & { tasks?: Task[] }): TaskStore {
  return {
    version: TASK_STORE_VERSION,
    tasks: sortTasks(store.tasks ?? []),
  };
}

export function createTask(draft: TaskDraft, now = new Date()): Task {
  const timestamp = now.toISOString();
  const title = draft.title.trim();
  const note = draft.note.trim();
  const dueDate = draft.dueDate.trim();

  return {
    id: globalThis.crypto?.randomUUID?.() ?? `task-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    title,
    note,
    dueDate: dueDate.length > 0 ? dueDate : null,
    done: false,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export function toTaskDraft(task: Task): TaskDraft {
  return {
    title: task.title,
    note: task.note,
    dueDate: task.dueDate ?? "",
  };
}

export function summaryForTask(task: Task) {
  const note = task.note.trim();
  return note.length > 0 ? note : "无备注";
}
