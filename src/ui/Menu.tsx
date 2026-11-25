// A menu that opens under its button, such as "Importieren" with its sources or the actions of a row. It is a native
// popover, so it lies in the top layer above everything and closes on Escape and on a click beside it. The arrow keys
// move between the entries, Home and End jump, and the focus returns to the button.
// The highlight that glides from entry to entry under the pointer and the focus is adapted from Beautiful UI "Glide
// Menu" (github.com/slev12397/beautiful-ui, components/primitives/GlideMenu.tsx, MIT, Copyright (c) 2026 Shane
// Levine): one layer measured against the row under the pointer, its timing taken from the duration tokens.
import { useEffect, useId, useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import { Icon, type IconName } from "./icons";
import { cn } from "./motion";

export type MenuEntry = { label: string; icon?: IconName; danger?: boolean; onSelect: () => void };

type MenuProps = {
  entries: MenuEntry[];
  /** Draws the button, given what it needs to open the menu. */
  trigger: (props: { popoverTarget: string; "aria-haspopup": "menu"; "aria-expanded": boolean; ref: (node: HTMLButtonElement | null) => void }) => ReactNode;
  align?: "start" | "end";
};

export function Menu({ entries, trigger, align = "start" }: MenuProps) {
  const id = useId();
  const menuRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const [open, setOpen] = useState(false);
  const [glide, setGlide] = useState<{ top: number; height: number } | null>(null);

  useEffect(() => {
    const menu = menuRef.current;
    if (!menu) return;
    const onToggle = (event: Event) => {
      const isOpen = (event as ToggleEvent).newState === "open";
      setOpen(isOpen);
      if (!isOpen) {
        setGlide(null);
        return;
      }
      const anchor = triggerRef.current?.getBoundingClientRect();
      if (anchor) {
        menu.style.top = `${anchor.bottom + 4}px`;
        menu.style.left = align === "start" ? `${anchor.left}px` : `${anchor.right - menu.offsetWidth}px`;
      }
      menu.querySelector<HTMLElement>("[role='menuitem']")?.focus();
    };
    menu.addEventListener("toggle", onToggle);
    return () => menu.removeEventListener("toggle", onToggle);
  }, [align]);

  const moveGlideTo = (target: EventTarget | null) => {
    const menu = menuRef.current;
    const row = target instanceof Element ? target.closest("[role='menuitem']") : null;
    if (!menu || !(row instanceof HTMLElement)) return;
    setGlide({ top: row.offsetTop, height: row.offsetHeight });
  };

  const onKeyDown = (event: KeyboardEvent) => {
    const items = [...(menuRef.current?.querySelectorAll<HTMLElement>("[role='menuitem']") ?? [])];
    const at = items.indexOf(document.activeElement as HTMLElement);
    const next = { ArrowDown: (at + 1) % items.length, ArrowUp: (at - 1 + items.length) % items.length, Home: 0, End: items.length - 1 }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    items[next]?.focus();
  };

  const choose = (entry: MenuEntry) => {
    menuRef.current?.hidePopover();
    triggerRef.current?.focus();
    entry.onSelect();
  };

  return (
    <>
      {trigger({
        popoverTarget: id,
        "aria-haspopup": "menu",
        "aria-expanded": open,
        ref: (node) => {
          triggerRef.current = node;
        },
      })}
      <div
        ref={menuRef}
        id={id}
        popover="auto"
        role="menu"
        onKeyDown={onKeyDown}
        onMouseOver={(event) => moveGlideTo(event.target)}
        onFocus={(event) => moveGlideTo(event.target)}
        onMouseLeave={() => {
          if (!menuRef.current?.contains(document.activeElement)) setGlide(null);
        }}
        className="fa-menu fixed m-0 min-w-48 rounded-panel bg-raised p-1 text-fg shadow-raised"
      >
        <span
          aria-hidden
          className="pointer-events-none absolute inset-x-1 rounded-row bg-selected transition-[top,height,opacity] duration-(--duration-move) ease-(--ease-out)"
          style={{ top: glide?.top ?? 0, height: glide?.height ?? 0, opacity: glide ? 1 : 0 }}
        />
        {entries.map((entry) => (
          <button
            key={entry.label}
            type="button"
            role="menuitem"
            tabIndex={-1}
            onClick={() => choose(entry)}
            className={cn(
              "relative z-10 flex h-8 w-full items-center gap-2 rounded-row px-2 text-left text-body outline-none",
              entry.danger ? "text-danger" : "text-fg",
            )}
          >
            {entry.icon && <Icon name={entry.icon} size={15} className={entry.danger ? undefined : "text-fg-muted"} />}
            {entry.label}
          </button>
        ))}
      </div>
    </>
  );
}
