import {
  lazy,
  Suspense,
  useEffect,
  useMemo,
  useState,
  type PointerEvent,
} from "react";
import { defaultWindowIcon } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CirclePlus, ListTodo, RefreshCw, Sparkles } from "lucide-react";
import { loadTaskStore, saveTaskStore } from "./lib/task-api";
import {
  createTask,
  sortTasks,
  type Task,
  type TaskDraft,
  type TaskStore,
} from "./lib/tasks";
import { Button } from "./components/ui/button";
import { TaskRow } from "./components/task-row";
import { WindowControls } from "./components/window-controls";
import { cn } from "./lib/utils";

const TaskDialog = lazy(() =>
  import("./components/task-dialog").then((module) => ({
    default: module.TaskDialog,
  })),
);

function EmptyState({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 px-6 py-10 text-center">
      <div className="flex size-12 items-center justify-center rounded-2xl border border-[var(--border)] bg-[var(--panel-muted)] text-[var(--accent)]">
        <ListTodo className="size-5" />
      </div>
      <div className="space-y-1">
        <h2 className="text-sm font-semibold">还没有任务</h2>
        <p className="max-w-xs text-sm text-[var(--muted-foreground)]">
          新建一条任务，标题、备注和日期就能开始用了。
        </p>
      </div>
      <Button onClick={onCreate}>
        <CirclePlus className="size-4" />
        新建任务
      </Button>
    </div>
  );
}

function LoadingState() {
  return (
    <div className="space-y-3 px-4 py-4">
      {Array.from({ length: 4 }).map((_, index) => (
        <div
          key={index}
          className="h-[4.75rem] animate-pulse rounded-lg border border-[var(--border)] bg-[var(--panel-muted)]"
        />
      ))}
    </div>
  );
}

function humanizeError(error: unknown) {
  if (typeof error === "string") {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "发生了未知错误";
}

function App() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);

  useEffect(() => {
    let active = true;

    defaultWindowIcon()
      .then((icon) => {
        if (!active || !icon) {
          return;
        }

        return getCurrentWindow().setIcon(icon);
      })
      .catch(() => {
        // Browser preview and unsupported platforms can ignore runtime icon setup.
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let active = true;

    loadTaskStore()
      .then((store: TaskStore) => {
        if (!active) {
          return;
        }

        setTasks(sortTasks(store.tasks));
        setError("");
      })
      .catch((loadError) => {
        if (active) {
          setError(humanizeError(loadError));
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (loading) {
      return;
    }

    const frame = requestAnimationFrame(() => {
      getCurrentWindow()
        .show()
        .catch(() => {
          // Browser preview does not expose the Tauri window API.
        });
    });

    return () => cancelAnimationFrame(frame);
  }, [loading]);

  const counts = useMemo(() => {
    const completed = tasks.filter((task) => task.done).length;
    const open = tasks.length - completed;

    return { completed, open };
  }, [tasks]);

  async function persist(nextTasks: Task[]) {
    setSaving(true);
    setError("");

    try {
      const store = await saveTaskStore(nextTasks);
      setTasks(sortTasks(store.tasks));
      return true;
    } catch (saveError) {
      setError(humanizeError(saveError));
      return false;
    } finally {
      setSaving(false);
    }
  }

  async function handleSave(task: Task | TaskDraft) {
    const isExistingTask = "createdAt" in task;

    if (isExistingTask) {
      const nextTasks = tasks.map((currentTask) =>
        currentTask.id === task.id
          ? {
              ...task,
            }
          : currentTask,
      );

      const saved = await persist(nextTasks);
      if (saved) {
        setDialogOpen(false);
        setEditingTask(null);
      }

      return;
    }

    const nextTask = createTask(task);

    const saved = await persist(sortTasks([...tasks, nextTask]));
    if (saved) {
      setDialogOpen(false);
      setEditingTask(null);
    }
  }

  async function handleToggle(task: Task) {
    const nextTasks = tasks.map((currentTask) =>
      currentTask.id === task.id
        ? {
            ...currentTask,
            done: task.done,
            updatedAt: new Date().toISOString(),
          }
        : currentTask,
    );

    await persist(nextTasks);
  }

  async function handleDelete(task: Task) {
    await persist(tasks.filter((currentTask) => currentTask.id !== task.id));
  }

  function openCreateDialog() {
    setEditingTask(null);
    setDialogOpen(true);
  }

  function openEditDialog(task: Task) {
    setEditingTask(task);
    setDialogOpen(true);
  }

  async function handleTitlebarPointerDown(event: PointerEvent<HTMLElement>) {
    if (event.button !== 0) {
      return;
    }

    const target = event.target as HTMLElement | null;
    if (target?.closest("button, input, textarea, select, a, [data-no-drag]")) {
      return;
    }

    try {
      await getCurrentWindow().startDragging();
    } catch {
      // Browser preview and denied IPC both end up here; the app remains usable.
    }
  }

  return (
    <div className="flex h-full min-h-screen flex-col bg-[var(--background)] text-[var(--foreground)]">
      <header
        className="flex cursor-default select-none items-center gap-3 border-b border-[var(--border)] bg-[var(--panel)] px-4 py-3"
        data-testid="titlebar"
        onPointerDown={handleTitlebarPointerDown}
      >
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex size-9 items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--panel-muted)] text-[var(--accent)]">
            <ListTodo className="size-4" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h1 className="truncate text-sm font-semibold">Record</h1>
              {saving ? (
                <span className="inline-flex items-center gap-1 text-[11px] text-[var(--muted-foreground)]">
                  <Sparkles className="size-3" />
                  保存中
                </span>
              ) : null}
            </div>
            <p className="truncate text-xs text-[var(--muted-foreground)]">
              {counts.open} 个待办 · {counts.completed} 个已完成
            </p>
          </div>
        </div>

        <div className="min-w-2 flex-1 self-stretch" />

        <div className="flex items-center gap-2" data-no-drag>
          <Button variant="secondary" size="sm" onClick={openCreateDialog}>
            <CirclePlus className="size-4" />
            新建
          </Button>
          <WindowControls />
        </div>
      </header>

      {error ? (
        <div className="border-b border-[var(--border)] bg-[var(--danger-muted)] px-4 py-2 text-sm text-[var(--danger)]">
          <div className="flex items-center justify-between gap-3">
            <span className="truncate">{error}</span>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-[var(--danger)] hover:bg-white/20"
              onClick={() => {
                setError("");
                setLoading(true);
                loadTaskStore()
                  .then((store) => {
                    setTasks(sortTasks(store.tasks));
                  })
                  .catch((loadError) => {
                    setError(humanizeError(loadError));
                  })
                  .finally(() => {
                    setLoading(false);
                  });
              }}
            >
              <RefreshCw className="size-3.5" />
              重试
            </Button>
          </div>
        </div>
      ) : null}

      <main className={cn("flex min-h-0 flex-1 flex-col")}>
        {loading ? (
          <LoadingState />
        ) : tasks.length === 0 ? (
          <EmptyState onCreate={openCreateDialog} />
        ) : (
          <div className="flex-1 overflow-y-auto">
            <div className="space-y-2 px-4 py-4">
              {tasks.map((task) => (
                <TaskRow
                  key={task.id}
                  task={task}
                  onToggle={handleToggle}
                  onEdit={openEditDialog}
                  onDelete={handleDelete}
                />
              ))}
            </div>
          </div>
        )}
      </main>

      {dialogOpen ? (
        <Suspense fallback={null}>
          <TaskDialog
            open={dialogOpen}
            task={editingTask}
            onOpenChange={(nextOpen) => {
              setDialogOpen(nextOpen);
              if (!nextOpen) {
                setEditingTask(null);
              }
            }}
            onSave={handleSave}
          />
        </Suspense>
      ) : null}
    </div>
  );
}

export default App;
