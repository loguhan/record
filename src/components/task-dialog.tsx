import { useEffect, useMemo, useState, type FormEvent } from "react";
import { CalendarDays } from "lucide-react";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Textarea } from "./ui/textarea";
import { isValidDueDate, toTaskDraft, type Task, type TaskDraft } from "../lib/tasks";

type TaskDialogProps = {
  open: boolean;
  task: Task | null;
  onOpenChange: (open: boolean) => void;
  onSave: (task: Task | TaskDraft) => Promise<void>;
};

function makeEmptyDraft(): TaskDraft {
  return {
    title: "",
    note: "",
    dueDate: "",
  };
}

export function TaskDialog({ open, task, onOpenChange, onSave }: TaskDialogProps) {
  const [draft, setDraft] = useState<TaskDraft>(makeEmptyDraft);
  const [error, setError] = useState("");
  const mode = useMemo(() => (task ? "edit" : "create"), [task]);

  useEffect(() => {
    if (open) {
      setDraft(task ? toTaskDraft(task) : makeEmptyDraft());
      setError("");
    }
  }, [open, task]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const title = draft.title.trim();
    if (!title) {
      setError("请输入任务标题");
      return;
    }

    if (!isValidDueDate(draft.dueDate)) {
      setError("日期格式必须是 YYYY-MM-DD");
      return;
    }

    setError("");

    if (task) {
      await onSave({
        ...task,
        title,
        note: draft.note.trim(),
        dueDate: draft.dueDate.trim().length > 0 ? draft.dueDate.trim() : null,
        updatedAt: new Date().toISOString(),
      });
    } else {
      await onSave({
        title,
        note: draft.note.trim(),
        dueDate: draft.dueDate.trim(),
      });
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[calc(100vh-2rem)] overflow-y-auto">
        <DialogHeader className="pr-8">
          <DialogTitle>{mode === "edit" ? "编辑任务" : "新建任务"}</DialogTitle>
          <DialogDescription>保持字段精简，标题、备注和日期就够了。</DialogDescription>
        </DialogHeader>

        <form className="mt-5 space-y-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="task-title">任务标题</Label>
            <Input
              id="task-title"
              value={draft.title}
              onChange={(event) => setDraft((current) => ({ ...current, title: event.target.value }))}
              placeholder="例如：整理周报"
              autoFocus
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="task-note">备注</Label>
            <Textarea
              id="task-note"
              value={draft.note}
              onChange={(event) => setDraft((current) => ({ ...current, note: event.target.value }))}
              placeholder="补充一点上下文，也可以留空"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="task-due-date" className="flex items-center gap-2">
              <CalendarDays className="size-3.5 text-[var(--muted-foreground)]" />
              截止日期
            </Label>
            <Input
              id="task-due-date"
              type="date"
              value={draft.dueDate}
              onChange={(event) => setDraft((current) => ({ ...current, dueDate: event.target.value }))}
            />
          </div>

          {error ? <p className="text-sm text-[var(--danger)]">{error}</p> : null}

          <div className="flex items-center justify-end gap-2 pt-2">
            <Button variant="secondary" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit">{task ? "保存更改" : "保存任务"}</Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
