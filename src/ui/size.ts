// The size of the whole view (#53, rule 27): Klein, Mittel or Groß as the zoom of the window, so a view folds by the
// width it really has (a window of 1440 px at Groß behaves as one of 1252 px). In the Tauri window it is the zoom of the
// webview (`setZoom`, permission core:webview:allow-set-webview-zoom); where that is not there, such as the stand-in of
// the core in a browser, the CSS zoom of the root.
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { SIZES, type Size } from "./layout";

export async function applySize(size: Size): Promise<void> {
  const scale = SIZES[size];
  try {
    await getCurrentWebview().setZoom(scale);
    document.documentElement.style.removeProperty("zoom");
  } catch {
    document.documentElement.style.setProperty("zoom", String(scale));
  }
}
