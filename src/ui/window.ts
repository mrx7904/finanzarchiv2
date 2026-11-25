// The window behind the top bar. Tauri's window API where the page runs in the app, nothing in a plain browser (the
// showcase on the Vite server), so the bar can be looked at anywhere. Each call needs its permission in
// src-tauri/capabilities/default.json, named in knowledge/decision-app-shell.md.
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type WindowSystem = "mac" | "windows" | "other";

/** macOS draws its own traffic lights over the bar, Windows gets the app's buttons, other systems keep their frame. */
export function windowSystem(platform: string = typeof navigator === "undefined" ? "" : navigator.platform): WindowSystem {
  if (/^Mac/i.test(platform)) return "mac";
  if (/^Win/i.test(platform)) return "windows";
  return "other";
}

function withWindow(action: (window: ReturnType<typeof getCurrentWindow>) => Promise<unknown>): void {
  if (!isTauri()) return;
  action(getCurrentWindow()).catch(() => {
    // A refused or failed window call leaves the window as it is, there is nothing the owner could do about it.
  });
}

export const minimizeWindow = () => withWindow((w) => w.minimize());
export const toggleMaximizeWindow = () => withWindow((w) => w.toggleMaximize());
export const closeWindow = () => withWindow((w) => w.close());

/** Whether the window fills the screen, so the middle button can show "Wiederherstellen". False outside the app. */
export async function isWindowMaximized(): Promise<boolean> {
  if (!isTauri()) return false;
  return getCurrentWindow()
    .isMaximized()
    .catch(() => false);
}
