import { useEffect, useState } from "react";
import { Minus, Square, Copy, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function WindowControls({ className }: { className?: string }) {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    if (!isTauri) return;
    try {
      const appWindow = getCurrentWindow();
      appWindow.isMaximized().then(setIsMaximized).catch(() => {});
      const unlistenPromise = appWindow.onResized(() => {
        appWindow.isMaximized().then(setIsMaximized).catch(() => {});
      });
      return () => {
        unlistenPromise.then((unlisten) => unlisten()).catch(() => {});
      };
    } catch {
      // ignore
    }
  }, []);

  if (!isTauri) {
    return null;
  }

  const handleMinimize = () => {
    getCurrentWindow().minimize().catch(() => {});
  };

  const handleToggleMaximize = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.toggleMaximize();
      const max = await appWindow.isMaximized();
      setIsMaximized(max);
    } catch {
      // ignore
    }
  };

  const handleClose = () => {
    getCurrentWindow().close().catch(() => {});
  };

  return (
    <div className={`flex items-center select-none ${className ?? ""}`}>
      <button
        type="button"
        onClick={handleMinimize}
        title="最小化"
        className="flex h-9 w-11 items-center justify-center text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
      >
        <Minus className="h-3.5 w-3.5" />
      </button>
      <button
        type="button"
        onClick={handleToggleMaximize}
        title={isMaximized ? "向下还原" : "最大化"}
        className="flex h-9 w-11 items-center justify-center text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
      >
        {isMaximized ? (
          <Copy className="h-3 w-3" />
        ) : (
          <Square className="h-3 w-3" />
        )}
      </button>
      <button
        type="button"
        onClick={handleClose}
        title="关闭"
        className="flex h-9 w-11 items-center justify-center text-muted-foreground hover:bg-red-500 hover:text-white transition-colors"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
