# Decision: the top bar of the app is the title bar of the window

**Decided:** 2026-11-25 by the owner (#07, FA-11) · applies to `src/ui/AppShell.tsx`, `src/ui/window.ts`, the window section of `src-tauri/tauri.conf.json`, `src-tauri/tauri.windows.conf.json` and `src-tauri/capabilities/default.json`

## Decision

The owner, in their words: "dass die Topbar (wo auch minimieren, maximieren und schließen ist) von anfang an ein fester bestandteil der App sein soll für Seitentitel, buttons und sonstiges".

- The window has no title strip of the system. The app's own top bar (`TopBar` inside `AppShell`) is the title bar: the page title at the left, the view's actions at the right, and the region by which the window is dragged. A double click on it maximizes, as on a title bar of the system.
- **macOS:** the native traffic lights stay and lie over the left end of the bar. `titleBarStyle: "Overlay"` and `hiddenTitle: true` in `tauri.conf.json`, `trafficLightPosition` `{ x: 20, y: 23 }` to center them in the bar of 47 px (first 52 px, the owner asked for it about 10 % slimmer after trying it on 2026-11-25). The sidebar's top (or the bar, when a view has no sidebar) leaves `--traffic-lights-width` (80 px) free for them.
- **Windows:** `decorations: false` in `tauri.windows.conf.json`, and the app draws minimize, maximize (restore when maximized) and close at the right end of the bar, 46 px wide like the caption buttons of Windows 11, close turning the red of Windows (`--window-close`).
- **Linux and other systems:** keep the frame of the system for now, the bar is then a toolbar below it. Settled when Linux comes.
- `AppShell` is the frame of every view from the start, the empty shell of #01 included. A view gives it a title, its actions and its sidebar, it never draws a header of its own.

## Why per system

v2.tauri.app/learn/window-customization (read on 2026-11-25) offers both ways. Without a frame on macOS the window would lose the traffic lights and with them what the system gives a window: moving it into a full screen space, tiling, the menu of the green button. The overlay keeps them. Windows has no overlay style in Tauri, so there the frame goes and the app draws the buttons. 1.0 hid the title bar on both systems with Electron (`hiddenInset` on macOS, the system's buttons drawn over the top strip on Windows).

`app.windows` is one list, and a platform file replaces it whole (JSON Merge Patch, v2.tauri.app/develop/configuration-files, read on 2026-11-25). So the `app.windows` of `tauri.windows.conf.json` (beside the `bundle.resources` of PDFium from #04) repeats every setting of the main window, leaves out the three options only macOS reads, and adds `decorations: false`. `src/window.test.ts` checks that both entries are equal apart from those keys, so a change to the window in one file cannot be forgotten in the other.

## Permissions

The capability of the main window granted `core:event:allow-listen` and `core:event:allow-unlisten` before (#04, the result of a drop import). The bar adds exactly:

| Permission | For |
|---|---|
| `core:window:allow-start-dragging` | dragging the window by the bar (`data-tauri-drag-region`) |
| `core:window:allow-internal-toggle-maximize` | a double click on the bar maximizes or restores |
| `core:window:allow-minimize` | the minimize button on Windows |
| `core:window:allow-toggle-maximize` | the maximize button on Windows |
| `core:window:allow-is-maximized` | the maximize button shows "Wiederherstellen" when the window fills the screen |
| `core:window:allow-close` | the close button on Windows |

`src/window.test.ts` holds the capability to these six and the two of #04, and the description of the capability names each with its reason.

## Known limits

- **Snap layouts of Windows 11.** The menu of layouts under the maximize button belongs to the button of the system. With the app's own button it does not appear, Win+Z and dragging to an edge still work.
- **A drag region holds only on the element that carries it.** The bar, its title and the gap carry `data-tauri-drag-region`, the buttons do not, so they stay clickable.
- **UNVERIFIED on the Mac:** the overlay, the position of the traffic lights and the drag region were not seen on a Mac. The owner moved the look on the Mac to #08 (`decision-ui-libraries.md`). The showcase draws the traffic lights on other systems to preview the bar of the Mac.

## Rejected

- **No frame on both systems with drawn traffic lights on macOS.** Loses the system's window functions on the Mac, and drawn traffic lights never behave like the real ones.
- **Removing the frame from JavaScript at start** (`setDecorations(false)`): the frame of the system would flash at every start, and it needs one more permission.
