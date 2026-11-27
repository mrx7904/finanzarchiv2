import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { windowSystem } from "./ui/window";

// The top bar of the app is the title bar of the window (decision-app-shell.md). These tests hold the window
// configuration of each system to that.
type WindowConfig = Record<string, unknown> & { label: string };
const read = (file: string) => JSON.parse(readFileSync(file, "utf8")) as { app: { windows: WindowConfig[] } };
const base = read("src-tauri/tauri.conf.json").app.windows[0];
const windowsFile = "src-tauri/tauri.windows.conf.json";
const capability = JSON.parse(readFileSync("src-tauri/capabilities/default.json", "utf8")) as { windows: string[]; permissions: string[] };
const shell = readFileSync("src/ui/AppShell.tsx", "utf8");
// Options only macOS reads. Windows ignores them, so its entry leaves them out.
const MAC_ONLY = ["titleBarStyle", "hiddenTitle", "trafficLightPosition"];

describe("the window of macOS", () => {
  it("keeps the traffic lights over the app's own bar and shows no title strip", () => {
    expect(base?.titleBarStyle).toBe("Overlay");
    expect(base?.hiddenTitle).toBe(true);
    expect(base?.decorations).not.toBe(false);
  });
});

describe("the window of Windows", () => {
  it("has no frame of the system, the app draws the bar and its buttons", () => {
    expect(existsSync(windowsFile)).toBe(true);
    const own = read(windowsFile).app.windows;
    expect(own).toHaveLength(1);
    expect(own[0]?.decorations).toBe(false);
  });

  it("repeats every other setting of the main window, since the platform file replaces the list whole", () => {
    const own = read(windowsFile).app.windows[0];
    const rest = Object.fromEntries(Object.entries(own ?? {}).filter(([key]) => key !== "decorations"));
    const shared = Object.fromEntries(Object.entries(base ?? {}).filter(([key]) => !MAC_ONLY.includes(key)));
    expect(rest).toEqual(shared);
  });
});

describe("the top bar", () => {
  it("is the region the window is dragged by", () => {
    expect(shell.match(/data-tauri-drag-region/g)?.length).toBeGreaterThanOrEqual(3);
  });

  it("is granted dragging, the three window buttons, the events of #04 and the zoom of #53, and nothing else", () => {
    expect(capability.windows).toEqual(["main"]);
    expect([...capability.permissions].sort()).toEqual(
      [
        "core:event:allow-listen",
        "core:event:allow-unlisten",
        "core:window:allow-start-dragging",
        "core:window:allow-internal-toggle-maximize",
        "core:window:allow-minimize",
        "core:window:allow-toggle-maximize",
        "core:window:allow-is-maximized",
        "core:window:allow-close",
        // The sizes Klein, Mittel and Groß as the zoom of the webview (#53).
        "core:webview:allow-set-webview-zoom",
      ].sort(),
    );
  });

  it("draws the window buttons on Windows only and leaves room for the traffic lights on macOS", () => {
    expect(windowSystem("Win32")).toBe("windows");
    expect(windowSystem("MacIntel")).toBe("mac");
    expect(windowSystem("Linux x86_64")).toBe("other");
  });
});
