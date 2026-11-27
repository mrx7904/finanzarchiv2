// The frame of every view: the top bar is the title bar of the window, there is no strip of the system above it
// (asked for by the owner on 2026-11-25: the bar is part of the app from the start, for the page title, its buttons
// and whatever else a view needs). It carries the path of the page at the left (Path.tsx), the view's actions at the right, and is the
// region by which the window is dragged and, by a double click, maximized. On macOS the native traffic lights lie
// over its left end (the window's title bar style is "Overlay"), on Windows the app draws minimize, maximize and close
// at its right end in the size and order of Windows 11.
// No piece of the six UI sources is a window title bar, so this is after the idea of the custom title bar of the Tauri
// guide (v2.tauri.app/learn/window-customization, MIT): a drag region and three buttons calling the window API. The
// buttons follow the caption buttons of Windows 11 (46 px wide, close turning red), the bar the unified toolbar of
// macOS apps.
import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Icon } from "./icons";
import { cn } from "./motion";
import { PathNav, type PathSegment } from "./Path";
import { closeWindow, isWindowMaximized, minimizeWindow, toggleMaximizeWindow, windowSystem, type WindowSystem } from "./window";

type AppShellProps = {
  /** The path of the page in the bar, its last segment the page itself. One segment reads as a title. */
  path: PathSegment[];
  /** Buttons and other controls of the view, at the right end of the bar. */
  actions?: ReactNode;
  /** The sidebar's content. Without one the bar spans the window. */
  sidebar?: ReactNode;
  /** The name in the sidebar's top, beside the traffic lights on macOS. */
  appName?: ReactNode;
  children: ReactNode;
  /** For the showcase, which draws the bar of another system. */
  system?: WindowSystem;
  /** Leads back after a jump; without it the arrow at the left of the path is gone (#44). */
  onBack?: () => void;
};

/** Where a view puts its actions: the top bar (rule 2 of knowledge/decision-views.md). */
const BarSlot = createContext<HTMLElement | null>(null);

/** The actions of a view, drawn into the top bar wherever the view renders them. */
export function BarActions({ children }: { children: ReactNode }) {
  const slot = useContext(BarSlot);
  return slot ? createPortal(children, slot) : null;
}

export function AppShell({ path, actions, sidebar, appName, children, system = windowSystem(), onBack }: AppShellProps) {
  const [slot, setSlot] = useState<HTMLElement | null>(null);
  return (
    <div className="flex h-full min-h-0 overflow-hidden bg-canvas font-sans text-body text-fg">
      {sidebar && (
        <aside className="flex w-(--sidebar-width) shrink-0 flex-col bg-sidebar shadow-[inset_-1px_0_0_var(--line)]">
          <div
            data-tauri-drag-region
            className={cn("flex h-(--bar-height) shrink-0 items-center gap-2 select-none", system === "mac" ? "pl-(--traffic-lights-width)" : "pl-5")}
          >
            {appName}
          </div>
          {sidebar}
        </aside>
      )}
      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar path={path} actions={actions} system={system} insetForTrafficLights={!sidebar && system === "mac"} onBack={onBack} slot={setSlot} />
        <BarSlot.Provider value={slot}>
          <div data-content className="flex min-h-0 flex-1 flex-col">{children}</div>
        </BarSlot.Provider>
      </div>
    </div>
  );
}

type TopBarProps = {
  path: PathSegment[];
  actions?: ReactNode;
  system: WindowSystem;
  insetForTrafficLights: boolean;
  onBack?: () => void;
  /** Receives the element the actions of a view are drawn into. */
  slot?: (element: HTMLElement | null) => void;
};

export function TopBar({ path, actions, system, insetForTrafficLights, onBack, slot }: TopBarProps) {
  const buttons = system === "windows";
  return (
    <header
      data-tauri-drag-region
      className={cn(
        "flex h-(--bar-height) shrink-0 items-center gap-3 border-b border-line bg-canvas select-none",
        insetForTrafficLights ? "pl-(--traffic-lights-width)" : "pl-6",
        buttons ? "pr-0" : "pr-4",
      )}
    >
      {/* A drag region holds only on the element that carries it: the bar and the free gap carry it, the path does not,
          so its segments stay buttons. */}
      {onBack && (
        <button
          type="button"
          aria-label="Zurück"
          title="Zurück (Alt+←)"
          onClick={onBack}
          className="-mr-1 flex size-7 shrink-0 items-center justify-center rounded-control text-fg-muted transition-quick hover:bg-hover hover:text-fg"
        >
          <Icon name="back" size={16} />
        </button>
      )}
      <PathNav path={path} />
      <div data-tauri-drag-region className="h-full min-w-4 flex-1" />
      <div ref={slot} className="flex shrink-0 items-center gap-2 empty:hidden" />
      {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
      {buttons && <WindowButtons />}
    </header>
  );
}

function WindowButtons() {
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    const update = () => void isWindowMaximized().then(setMaximized);
    update();
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);

  const button = "flex h-full w-(--window-button-width) items-center justify-center text-fg transition-quick";
  return (
    <div className="ml-2 flex h-full shrink-0 self-start">
      <button type="button" aria-label="Minimieren" title="Minimieren" onClick={minimizeWindow} className={cn(button, "hover:bg-hover")}>
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
          <path d="M0 5.5h10" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
      <button
        type="button"
        aria-label={maximized ? "Wiederherstellen" : "Maximieren"}
        title={maximized ? "Wiederherstellen" : "Maximieren"}
        onClick={toggleMaximizeWindow}
        className={cn(button, "hover:bg-hover")}
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1" aria-hidden>
          {maximized ? (
            <>
              <rect x="0.5" y="2.5" width="7" height="7" rx="1" />
              <path d="M2.5 2.5V1.5a1 1 0 0 1 1-1h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-1" />
            </>
          ) : (
            <rect x="0.5" y="0.5" width="9" height="9" rx="1" />
          )}
        </svg>
      </button>
      <button type="button" aria-label="Schließen" title="Schließen" onClick={closeWindow} className={cn(button, "hover:bg-window-close hover:text-on-window-close")}>
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
          <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
    </div>
  );
}
