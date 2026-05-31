import { describe, expect, it } from "vitest";
import {
  createTask,
  formatDueDate,
  isValidDueDate,
  normalizeTaskStore,
  sortTasks,
  toDateInputValue,
  type Task,
} from "./tasks";

function task(id: string, done: boolean): Task {
  return {
    id,
    title: id,
    note: "",
    dueDate: null,
    done,
    createdAt: "2026-05-31T00:00:00.000Z",
    updatedAt: "2026-05-31T00:00:00.000Z",
  };
}

describe("task helpers", () => {
  it("keeps open tasks before completed tasks without shuffling each group", () => {
    expect(sortTasks([task("done-a", true), task("open-a", false), task("done-b", true)])).toEqual([
      task("open-a", false),
      task("done-a", true),
      task("done-b", true),
    ]);
  });

  it("normalizes store versions and task order", () => {
    expect(
      normalizeTaskStore({
        version: 99,
        tasks: [task("done", true), task("open", false)],
      }),
    ).toMatchObject({
      version: 1,
      tasks: [task("open", false), task("done", true)],
    });
  });

  it("formats local input dates", () => {
    expect(toDateInputValue(new Date(2026, 4, 31))).toBe("2026-05-31");
  });

  it("validates YYYY-MM-DD due dates", () => {
    expect(isValidDueDate(null)).toBe(true);
    expect(isValidDueDate("2026-05-31")).toBe(true);
    expect(isValidDueDate("2024-02-29")).toBe(true);
    expect(isValidDueDate("2026-02-29")).toBe(false);
    expect(isValidDueDate("2026-5-31")).toBe(false);
  });

  it("labels nearby dates compactly", () => {
    const now = new Date(2026, 4, 31);

    expect(formatDueDate(null, now)).toBe("无日期");
    expect(formatDueDate("2026-05-31", now)).toBe("今天");
    expect(formatDueDate("2026-06-01", now)).toBe("明天");
    expect(formatDueDate("2026-05-30", now)).toBe("昨天");
  });

  it("creates complete task records from drafts", () => {
    const created = createTask(
      {
        title: "  写计划  ",
        note: "  保持轻量  ",
        dueDate: "2026-06-01",
      },
      new Date("2026-05-31T00:00:00.000Z"),
    );

    expect(created).toMatchObject({
      title: "写计划",
      note: "保持轻量",
      dueDate: "2026-06-01",
      done: false,
      createdAt: "2026-05-31T00:00:00.000Z",
      updatedAt: "2026-05-31T00:00:00.000Z",
    });
    expect(created.id).toBeTruthy();
  });
});
