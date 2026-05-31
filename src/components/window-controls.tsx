import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Pin, X } from "lucide-react";
import { Button } from "./ui/button";
import { cn } from "../lib/utils";

const ALWAYS_ON_TOP_KEY = "record.alwaysOnTop";

function readStoredAlwaysOnTop() {
  try {
    return window.localStorage.getItem(ALWAYS_ON_TOP_KEY) === "true";
  } catch {
    return false;
  }
}

function writeStoredAlwaysOnTop(alwaysOnTop: boolean) {
  try {
    window.localStorage.setItem(ALWAYS_ON_TOP_KEY, String(alwaysOnTop));
  } catch {
    // Storage can be unavailable in restricted previews.
  }
}

export async function minimizeWindow() {
  await getCurrentWindow().minimize();
}

export async function closeWindow() {
  const window = getCurrentWindow();

  try {
    await window.close();
  } catch {
    await window.destroy();
  }
}

export async function setWindowAlwaysOnTop(alwaysOnTop: boolean) {
  await getCurrentWindow().setAlwaysOnTop(alwaysOnTop);
}

export function WindowControls() {
  const [alwaysOnTop, setAlwaysOnTop] = useState(readStoredAlwaysOnTop);

  async function runWindowAction(action: () => Promise<void>) {
    try {
      await action();
    } catch {
      // No-op when running outside Tauri.
    }
  }

  useEffect(() => {
    runWindowAction(() => setWindowAlwaysOnTop(alwaysOnTop));
  }, []);

  function toggleAlwaysOnTop() {
    const nextAlwaysOnTop = !alwaysOnTop;
    setAlwaysOnTop(nextAlwaysOnTop);
    writeStoredAlwaysOnTop(nextAlwaysOnTop);
    runWindowAction(() => setWindowAlwaysOnTop(nextAlwaysOnTop));
  }

  return (
    <div className="flex items-center gap-1" data-no-drag>
      <Button
        variant="ghost"
        size="icon"
        className={cn(
          "size-8 text-[var(--muted-foreground)]",
          alwaysOnTop && "bg-[var(--panel-muted)] text-[var(--accent)]",
        )}
        aria-label={alwaysOnTop ? "取消窗口置顶" : "窗口置顶"}
        title={alwaysOnTop ? "取消窗口置顶" : "窗口置顶"}
        onClick={toggleAlwaysOnTop}
      >
        <Pin className="size-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        className="size-8 text-[var(--muted-foreground)]"
        aria-label="最小化窗口"
        onClick={() => runWindowAction(minimizeWindow)}
      >
        <Minus className="size-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        className="size-8 text-[var(--muted-foreground)] hover:bg-[var(--danger-muted)] hover:text-[var(--danger)]"
        aria-label="关闭窗口"
        onClick={() => runWindowAction(closeWindow)}
      >
        <X className="size-3.5" />
      </Button>
    </div>
  );
}
