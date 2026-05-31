import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { Task, TaskStore } from "./lib/tasks";

const mocks = vi.hoisted(() => {
  const store: TaskStore = {
    version: 1,
    tasks: [],
  };

  return {
    store,
    invoke: vi.fn(async (command: string, payload?: { tasks?: Task[] }) => {
      if (command === "load_tasks") {
        return structuredClone(store);
      }

      if (command === "save_tasks") {
        store.tasks = structuredClone(payload?.tasks ?? []);
        return structuredClone(store);
      }

      throw new Error(`Unexpected invoke: ${command}`);
    }),
    minimize: vi.fn(async () => undefined),
    close: vi.fn(async () => undefined),
    destroy: vi.fn(async () => undefined),
    defaultWindowIcon: vi.fn(async () => "mock-window-icon"),
    setAlwaysOnTop: vi.fn(async () => undefined),
    setIcon: vi.fn(async () => undefined),
    startDragging: vi.fn(async () => undefined),
    show: vi.fn(async () => undefined),
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/app", () => ({
  defaultWindowIcon: mocks.defaultWindowIcon,
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: mocks.minimize,
    close: mocks.close,
    destroy: mocks.destroy,
    setAlwaysOnTop: mocks.setAlwaysOnTop,
    setIcon: mocks.setIcon,
    startDragging: mocks.startDragging,
    show: mocks.show,
  }),
}));

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: "task-1",
    title: "整理周报",
    note: "周五前完成",
    dueDate: "2026-06-01",
    done: false,
    createdAt: "2026-05-31T00:00:00.000Z",
    updatedAt: "2026-05-31T00:00:00.000Z",
    ...overrides,
  };
}

describe("App", () => {
  beforeEach(() => {
    mocks.store.version = 1;
    mocks.store.tasks = [];
    mocks.invoke.mockClear();
    mocks.minimize.mockClear();
    mocks.close.mockClear();
    mocks.destroy.mockClear();
    mocks.defaultWindowIcon.mockClear();
    mocks.setAlwaysOnTop.mockClear();
    mocks.setIcon.mockClear();
    mocks.startDragging.mockClear();
    mocks.show.mockClear();
    localStorage.clear();
  });

  it("creates and persists a task", async () => {
    const user = userEvent.setup();
    render(<App />);

    expect(await screen.findByText("还没有任务")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "新建任务" }));
    await user.type(await screen.findByLabelText("任务标题"), "写计划");
    await user.type(screen.getByLabelText("备注"), "保持轻量");
    await user.type(screen.getByLabelText("截止日期"), "2026-06-01");
    await user.click(screen.getByRole("button", { name: "保存任务" }));

    expect(await screen.findByText("写计划")).toBeInTheDocument();
    expect(mocks.store.tasks).toHaveLength(1);
    expect(mocks.store.tasks[0]).toMatchObject({
      title: "写计划",
      note: "保持轻量",
      dueDate: "2026-06-01",
      done: false,
    });
  });

  it("shows the native window after the first task load finishes", async () => {
    render(<App />);

    expect(await screen.findByText("还没有任务")).toBeInTheDocument();
    await waitFor(() => expect(mocks.show).toHaveBeenCalledTimes(1));
  });

  it("applies the packaged window icon on startup", async () => {
    render(<App />);

    expect(await screen.findByText("还没有任务")).toBeInTheDocument();
    await waitFor(() =>
      expect(mocks.defaultWindowIcon).toHaveBeenCalledTimes(1),
    );
    expect(mocks.setIcon).toHaveBeenCalledWith("mock-window-icon");
  });

  it("edits, completes, and deletes an existing task", async () => {
    const user = userEvent.setup();
    mocks.store.tasks = [makeTask()];

    render(<App />);

    expect(await screen.findByText("整理周报")).toBeInTheDocument();

    await user.click(
      screen.getByRole("checkbox", { name: "切换 整理周报 完成状态" }),
    );
    expect(mocks.store.tasks[0].done).toBe(true);

    await user.click(screen.getByRole("button", { name: "编辑 整理周报" }));
    const titleInput = await screen.findByLabelText("任务标题");
    await user.clear(titleInput);
    await user.type(titleInput, "整理发布清单");
    await user.click(screen.getByRole("button", { name: "保存更改" }));

    expect(await screen.findByText("整理发布清单")).toBeInTheDocument();
    expect(mocks.store.tasks[0].title).toBe("整理发布清单");

    const row = screen.getByText("整理发布清单").closest("article");
    expect(row).not.toBeNull();

    await user.click(
      within(row as HTMLElement).getByRole("button", {
        name: "删除 整理发布清单",
      }),
    );

    expect(screen.queryByText("整理发布清单")).not.toBeInTheDocument();
    expect(mocks.store.tasks).toHaveLength(0);
  });

  it("wires custom window controls", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("还没有任务");
    await user.click(screen.getByRole("button", { name: "最小化窗口" }));
    await user.click(screen.getByRole("button", { name: "关闭窗口" }));

    expect(mocks.minimize).toHaveBeenCalledTimes(1);
    expect(mocks.close).toHaveBeenCalledTimes(1);
    expect(mocks.destroy).not.toHaveBeenCalled();
  });

  it("toggles and remembers always-on-top", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("还没有任务");
    await user.click(screen.getByRole("button", { name: "窗口置顶" }));

    expect(mocks.setAlwaysOnTop).toHaveBeenLastCalledWith(true);
    expect(localStorage.getItem("record.alwaysOnTop")).toBe("true");

    await user.click(screen.getByRole("button", { name: "取消窗口置顶" }));

    expect(mocks.setAlwaysOnTop).toHaveBeenLastCalledWith(false);
    expect(localStorage.getItem("record.alwaysOnTop")).toBe("false");
  });

  it("force-destroys the window when close is rejected", async () => {
    const user = userEvent.setup();
    mocks.close.mockRejectedValueOnce(new Error("permission rejected"));

    render(<App />);

    await screen.findByText("还没有任务");
    await user.click(screen.getByRole("button", { name: "关闭窗口" }));

    expect(mocks.close).toHaveBeenCalledTimes(1);
    expect(mocks.destroy).toHaveBeenCalledTimes(1);
  });

  it("starts native dragging from the titlebar but not from buttons", async () => {
    const user = userEvent.setup();
    render(<App />);

    await screen.findByText("还没有任务");
    fireEvent.pointerDown(screen.getByTestId("titlebar"), { button: 0 });
    expect(mocks.startDragging).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: "新建" }));
    expect(mocks.startDragging).toHaveBeenCalledTimes(1);
  });
});
