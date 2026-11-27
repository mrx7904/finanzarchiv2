// The button of the app in four kinds: `primary` for the one main action of a view or dialog, `secondary` beside it,
// `plain` for actions in a row or a toolbar, `danger` for what removes something. While its action runs it keeps its
// size and shows a spinner instead of its icon, so the row beside it does not jump.
// The busy state is after the idea of Aceternity UI "Stateful Button" (ui.aceternity.com/components/stateful-button,
// Aceternity License: its source must not be re-distributed, so only the idea is taken). The spinner is after the
// idea of RareUI "Loading Spinner" (rareui.in, now ShitUI, MIT with Commons Clause and an attribution duty since
// 2026, so its code is not copied): a still track and one turning arc. The pressed state of .98 is the one 1.0 took.
import type { ButtonHTMLAttributes, ReactNode, Ref } from "react";
import { Icon, type IconName } from "./icons";
import { cn } from "./motion";

export type ButtonKind = "primary" | "secondary" | "plain" | "danger";

const kinds: Record<ButtonKind, string> = {
  primary: "bg-accent text-on-accent hover:bg-accent-hover",
  secondary: "bg-surface text-fg shadow-panel hover:bg-hover",
  plain: "bg-transparent text-fg hover:bg-hover",
  danger: "bg-danger-soft text-danger hover:brightness-[0.97]",
};

type ButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  kind?: ButtonKind;
  size?: "regular" | "small";
  icon?: IconName;
  busy?: boolean;
  /** An icon-only button needs its name for a screen reader. */
  label?: string;
  children?: ReactNode;
  /** So a button can be the trigger of a `Menu` (#45). */
  ref?: Ref<HTMLButtonElement>;
};

export function Button({ kind = "secondary", size = "regular", icon, busy = false, label, className, disabled, children, type, ref, ...props }: ButtonProps) {
  return (
    <button
      ref={ref}
      data-kind={kind}
      type={type ?? "button"}
      aria-label={label}
      aria-busy={busy || undefined}
      disabled={disabled || busy}
      className={cn(
        "inline-flex shrink-0 items-center justify-center gap-1.5 rounded-control font-medium whitespace-nowrap select-none transition-quick",
        "active:scale-[0.98] disabled:cursor-default disabled:opacity-50 disabled:active:scale-100",
        size === "regular" ? "h-(--control-height) text-body" : "h-6 text-small",
        children ? (size === "regular" ? "px-3" : "px-2") : size === "regular" ? "w-(--control-height)" : "w-6",
        kinds[kind],
        className,
      )}
      {...props}
    >
      {busy ? <Spinner /> : icon && <Icon name={icon} size={size === "regular" ? 16 : 14} />}
      {children}
    </button>
  );
}

/** A still track and one turning arc in the current text color. Under reduced motion it stands still. */
export function Spinner({ size = 14, className }: { size?: number; className?: string }) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden className={cn("animate-spin [animation-duration:0.9s]", className)}>
      <circle cx="8" cy="8" r="6.25" fill="none" stroke="currentColor" strokeOpacity="0.25" strokeWidth="1.75" />
      <path d="M8 1.75a6.25 6.25 0 0 1 6.25 6.25" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" />
    </svg>
  );
}
