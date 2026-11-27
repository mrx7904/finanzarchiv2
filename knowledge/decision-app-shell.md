# Decision: the top bar of the app is the title bar of the window

**Decided:** 2026-11-25 by the owner (#07, FA-11) · applies to `src/ui/AppShell.tsx`, `src/ui/window.ts`, the window section of `src-tauri/tauri.conf.json`, `src-tauri/tauri.windows.conf.json` and `src-tauri/capabilities/default.json`

## Decision

The owner, in their words: "dass die Topbar (wo auch minimieren, maximieren und schließen ist) von anfang an ein fester bestandteil der App sein soll für Seitentitel, buttons und sonstiges".

- The window has no title strip of the system. The app's own top bar (`TopBar` inside `AppShell`) is the title bar: the path of the page at the left (see "The path of the page"), the view's actions at the right, and the region by which the window is dragged. A double click on it maximizes, as on a title bar of the system.
- **macOS:** the native traffic lights stay and lie over the left end of the bar. `titleBarStyle: "Overlay"` and `hiddenTitle: true` in `tauri.conf.json`, `trafficLightPosition` `{ x: 20, y: 23 }` to center them in the bar of 47 px (first 52 px, the owner asked for it about 10 % slimmer after trying it on 2026-11-25). The sidebar's top (or the bar, when a view has no sidebar) leaves `--traffic-lights-width` (80 px) free for them.
- **Windows:** `decorations: false` in `tauri.windows.conf.json`, and the app draws minimize, maximize (restore when maximized) and close at the right end of the bar, 46 px wide like the caption buttons of Windows 11, close turning the red of Windows (`--window-close`).
- **Linux and other systems:** keep the frame of the system for now, the bar is then a toolbar below it. Settled when Linux comes.
- `AppShell` is the frame of every view from the start, the empty shell of #01 included. A view gives it its path, its actions and its sidebar, it never draws a header of its own.

## The path of the page

**Decided:** 2026-11-25 by the owner, in their words: "Bei der Topbar sollte gerne (wo im Screenshot \"Dokumente\" steht, jweils der Pfad der geweiligen seite dabei sein. Sprich wenn ich mich in einem untermenu befinde, möchte ich sehen welche seiten davor sind und auch per klick auf diese kommen." Seen and approved by the owner in the dev app on Windows with paths of one, three and six pages.

- `AppShell` takes a `path` (`{ label, onSelect? }[]`) instead of a title. The last segment is the page itself, set like the title before and marked `aria-current="page"`, not a button. Every segment before it is a button (keyboard reachable, focus ring) that leads back to its page. A quiet chevron separates them. The list is a `nav` named "Pfad". With one segment the bar shows a title, as before.
- **The path stays on one line.** `PathNav` (`src/ui/Path.tsx`) shows every segment, and while the list is wider than the room the bar leaves it, it folds the middle segment farthest from the current page into "…", again on every change of the width. "…" opens the `Menu` of the design system with the folded pages. The first and the current segment always stay. The current page shortens with "…" only once nothing is left to fold, since a page that shortens never lets the list overflow.
- **The path is never part of the drag region.** The bar and the free gap after the path carry `data-tauri-drag-region`, the path does not.
- **No router yet.** The views of stage 2 build the path and give each earlier segment its `onSelect`. A routing package needs a decision of its own first. Settled on 2026-11-27: no package, the app keeps its own navigation state ([`decision-views.md`](decision-views.md), "How one moves"). Built with #44: `src/navigation.ts` is a reducer of the view, the place inside each view and the way back of jumps; the sidebar leaves the way back, a jump adds to it (at most 30 places), the arrow at the left of the path and Alt+Left go back. A view puts its actions into the top bar with `BarActions`, a portal into a slot of the bar.
- `src/ui/path.test.tsx` checks that the last segment is not a button, that the others and the folded entries call their `onSelect`, and that folding keeps the first and the current segment.

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

## Fitting a view (#53)

**Decided:** 2026-11-27 · applies to `src/ui/layout.ts`, `src/ui/Columns.tsx`, `src/ui/size.ts`, `src-tauri/capabilities/default.json`

- **The size is the zoom of the webview.** Klein, Mittel and Groß (90, 100, 115 %) call `getCurrentWebview().setZoom(scale)` of `@tauri-apps/api/webview`, so the media queries see the width the view really has. Checked at v2.tauri.app on 2026-11-27: `setZoom(scaleFactor)`, 1 being 100 %, needs the permission below, which `core:webview:default` does not hold. Where the call is not there (the stand-in of the core in a browser) the root gets the CSS `zoom` instead. **Not verified:** whether `setZoom` works on WKWebView; the docs name no platform. It is looked at with the Mac (#08).

| Permission | Since | Why |
|---|---|---|
| `core:webview:allow-set-webview-zoom` | #53 | the sizes Klein, Mittel and Groß as the zoom of the webview |

- **Kept on the computer, not in the archive.** Column widths, the rail, the folded columns and the size are kept in the webview's storage under `finanzarchiv.layout.<key>`, one key per thing and per view (`documents.list`, `bookings.detail`, `rail`, `size`, …), and read with their default when nothing or something broken is stored. A value equal to the default is removed, not stored.
