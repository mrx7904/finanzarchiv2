import { cn } from "./motion";
import { Icon, type IconName } from "./icons";

/** A row of the sidebar: a view of the app with its icon and, when something waits there, a count. */
export function SidebarRow({ label, icon, count, selected, onSelect }: { label: string; icon: IconName; count?: number; selected: boolean; onSelect: () => void }) {
  return (
    <button
      type="button"
      aria-current={selected ? "page" : undefined}
      onClick={onSelect}
      className={cn("relative flex h-8 w-full items-center gap-2.5 rounded-row px-2 text-left transition-quick", selected ? "bg-selected font-medium text-fg" : "text-fg hover:bg-hover")}
    >
      {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
      <Icon name={icon} size={16} className={selected ? "text-fg-accent" : "text-fg-muted"} />
      <span className="flex-1">{label}</span>
      {count !== undefined && count > 0 && <span className="rounded-full bg-accent px-1.5 text-small font-semibold text-on-accent tabular-nums">{count}</span>}
    </button>
  );
}
