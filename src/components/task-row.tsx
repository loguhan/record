import { CalendarDays, PencilLine, Trash2 } from "lucide-react";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Checkbox } from "./ui/checkbox";
import { cn } from "../lib/utils";
import { formatDueDate, summaryForTask, type Task } from "../lib/tasks";

type TaskRowProps = {
  task: Task;
  onToggle: (task: Task) => void;
  onEdit: (task: Task) => void;
  onDelete: (task: Task) => void;
};

export function TaskRow({ task, onToggle, onEdit, onDelete }: TaskRowProps) {
  return (
    <article
      className={cn(
        "rounded-lg border border-[var(--border)] bg-[var(--panel)] px-3 py-3 shadow-[0_1px_0_rgba(0,0,0,0.02)] transition-colors",
        task.done && "opacity-75",
      )}
    >
      <div className="flex items-start gap-3">
        <Checkbox
          checked={task.done}
          aria-label={`切换 ${task.title} 完成状态`}
          onCheckedChange={(checked) => onToggle({ ...task, done: checked === true })}
          className="mt-0.5"
        />

        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex items-start gap-3">
            <div className="min-w-0 flex-1 space-y-1">
              <h3
                className={cn(
                  "truncate text-sm font-medium leading-5 text-[var(--foreground)]",
                  task.done && "text-[var(--muted-foreground)] line-through",
                )}
              >
                {task.title}
              </h3>
              <p className="truncate text-xs text-[var(--muted-foreground)]">{summaryForTask(task)}</p>
            </div>

            <div className="flex shrink-0 items-center gap-2">
              <Badge
                variant={task.done ? "subtle" : task.dueDate ? "default" : "subtle"}
                className="gap-1"
              >
                <CalendarDays className="size-3" />
                {formatDueDate(task.dueDate)}
              </Badge>
              <Button
                variant="ghost"
                size="icon"
                className="size-8"
                aria-label={`编辑 ${task.title}`}
                onClick={() => onEdit(task)}
              >
                <PencilLine className="size-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="size-8 text-[var(--danger)] hover:bg-[var(--danger-muted)] hover:text-[var(--danger)]"
                aria-label={`删除 ${task.title}`}
                onClick={() => onDelete(task)}
              >
                <Trash2 className="size-3.5" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </article>
  );
}
