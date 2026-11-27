// Light, dark or as the system (#49, "Darstellung"). It belongs to the computer, not to the archive, so it is kept in
// the webview's storage and set on the root element before the first paint (main.tsx).
export type Theme = "system" | "light" | "dark";

const KEY = "finanzarchiv.theme";

export function storedTheme(storage: Pick<Storage, "getItem"> = window.localStorage): Theme {
  const value = storage.getItem(KEY);
  return value === "light" || value === "dark" ? value : "system";
}

export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement): void {
  if (theme === "system") delete root.dataset.theme;
  else root.dataset.theme = theme;
}

export function chooseTheme(theme: Theme, storage: Pick<Storage, "setItem" | "removeItem"> = window.localStorage): void {
  if (theme === "system") storage.removeItem(KEY);
  else storage.setItem(KEY, theme);
  applyTheme(theme);
}
