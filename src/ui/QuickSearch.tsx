// The quick search on ⌘K (Ctrl+K on Windows): a field at the top, the hits below in groups, the arrow keys move,
// Enter opens. It matches folding case, accents and ß (search.ts).
// Built on cmdk (cmdk.paco.me, MIT), the pick of the skill pick-ui-library for command menus. It lies in the app's own
// `Dialog` frame rather than cmdk's Radix dialog, so there is one dialog in the app. The rows and the empty answer
// follow Beautiful UI "Search List" (github.com/slev12397/beautiful-ui, MIT, Copyright (c) 2026 Shane Levine). The
// keys are drawn after the idea of "Kbd" by shadcn on 21st.dev (licence listed as unknown there, so nothing copied).
import { Command } from "cmdk";
import { useEffect, useId, useRef, type ReactNode } from "react";
import { Icon, type IconName } from "./icons";
import { cn } from "./motion";
import { matchScore } from "./search";

/** `keywords` are words the entry is found by without showing them, such as the owner's words for a page of Einstellungen. */
export type SearchEntry = { id: string; group: string; label: string; detail?: string; keywords?: string[]; icon: IconName; onSelect: () => void };

export function Kbd({ children }: { children: ReactNode }) {
  return (
    <kbd className="inline-flex h-5 min-w-5 items-center justify-center rounded-[4px] bg-canvas px-1 font-sans text-small text-fg-muted shadow-[inset_0_-1px_0_var(--line-strong),0_0_0_1px_var(--line)]">
      {children}
    </kbd>
  );
}

/** Opens the search on ⌘K or Ctrl+K from anywhere in the window. */
export function useQuickSearchKey(open: () => void) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        open();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);
}

export const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

type QuickSearchProps = { open: boolean; onClose: () => void; entries: SearchEntry[] };

export function QuickSearch({ open, onClose, entries }: QuickSearchProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const labelId = useId();
  const groups = [...new Set(entries.map((entry) => entry.group))];

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open && !dialog.open) {
      dialog.showModal();
      // The field mounts with the dialog, before showModal; the autofocus of React does not survive that order.
      dialog.querySelector<HTMLInputElement>("input")?.focus();
    }
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      aria-labelledby={labelId}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
      className="fa-dialog mx-auto mt-[12vh] w-[min(92vw,40rem)] overflow-hidden rounded-dialog bg-raised p-0 text-fg shadow-raised"
    >
      {open && (
        <Command label="Schnellsuche" filter={matchScore} loop>
          <span id={labelId} className="sr-only">
            Schnellsuche
          </span>
          <div className="flex h-12 items-center gap-2.5 border-b border-line px-4">
            <Icon name="search" size={16} className="shrink-0 text-fg-muted" />
            <Command.Input placeholder="Suchen nach Dokumenten, Absendern, Ansichten" className="min-w-0 flex-1 bg-transparent text-heading text-fg outline-none placeholder:text-fg-muted" />
            <Kbd>Esc</Kbd>
          </div>
          <Command.List className="max-h-[min(60vh,26rem)] overflow-y-auto p-1.5">
            <Command.Empty className="flex flex-col items-center gap-1 px-4 py-10 text-center">
              <span className="mb-1.5 flex size-8 items-center justify-center rounded-control bg-canvas text-fg-muted shadow-panel">
                <Icon name="search" size={15} />
              </span>
              <span className="font-medium text-fg">Nichts gefunden</span>
              <span className="text-small text-fg-muted">Ein anderes Wort oder ein Teil des Namens hilft oft</span>
            </Command.Empty>
            {groups.map((group) => (
              <Command.Group
                key={group}
                heading={group}
                className="[&_[cmdk-group-heading]]:px-2.5 [&_[cmdk-group-heading]]:pt-2 [&_[cmdk-group-heading]]:pb-1 [&_[cmdk-group-heading]]:text-small [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-fg-muted"
              >
                {entries
                  .filter((entry) => entry.group === group)
                  .map((entry) => (
                    <Command.Item
                      key={entry.id}
                      value={entry.id}
                      keywords={[entry.label, ...(entry.detail ? [entry.detail] : []), ...(entry.keywords ?? [])]}
                      onSelect={() => {
                        onClose();
                        entry.onSelect();
                      }}
                      className={cn(
                        "flex h-10 cursor-default items-center gap-3 rounded-row px-2.5 text-body text-fg select-none transition-quick",
                        "data-[selected=true]:bg-selected",
                      )}
                    >
                      <Icon name={entry.icon} size={16} className="shrink-0 text-fg-muted" />
                      <span className="min-w-0 flex-1 truncate">{entry.label}</span>
                      {entry.detail && <span className="shrink-0 text-small text-fg-muted">{entry.detail}</span>}
                    </Command.Item>
                  ))}
              </Command.Group>
            ))}
          </Command.List>
          <div className="flex items-center gap-3 border-t border-line px-4 py-2 text-small text-fg-muted">
            <span className="flex items-center gap-1">
              <Kbd>↑</Kbd>
              <Kbd>↓</Kbd> auswählen
            </span>
            <span className="flex items-center gap-1">
              <Kbd>↵</Kbd> öffnen
            </span>
          </div>
        </Command>
      )}
    </dialog>
  );
}
