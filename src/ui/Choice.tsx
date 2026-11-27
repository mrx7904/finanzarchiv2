// A switch for one setting that is on or off, and a segmented choice of two to four options, such as the size of the
// view or light and dark (#45). Both are reached by Tab, the switch flips with Space or Enter, the segments move with
// the arrow keys, and each carries its name for a screen reader.
// The switch after the idea of RareUI "Soft Button" and Magic UI's toggles, the segments after the idea of RareUI
// "Animated Tab" (rareui.in, MIT with Commons Clause since 2026, so idea only): a track with one moving part. Drawn
// for this app in the tokens of Graphit, the knob and the chosen segment move with the duration tokens.
import { useId, useRef, type KeyboardEvent } from "react";
import { cn } from "./motion";

type SwitchProps = {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
};

export function Switch({ label, checked, onChange, disabled = false }: SwitchProps) {
  const id = useId();
  return (
    <span className="inline-flex items-center gap-2.5">
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={cn(
          "relative inline-flex h-5 w-9 shrink-0 items-center rounded-full p-0.5 transition-quick disabled:opacity-50",
          checked ? "bg-accent" : "bg-line-strong",
        )}
      >
        <span
          aria-hidden
          className={cn(
            "size-4 rounded-full bg-surface shadow-panel transition-[translate] duration-(--duration-move) ease-(--ease-out)",
            checked ? "translate-x-4" : "translate-x-0",
          )}
        />
      </button>
      <label htmlFor={id} className="cursor-default text-body text-fg select-none">
        {label}
      </label>
    </span>
  );
}

/** The option an arrow key leads to, around the ends; `undefined` for any other key. */
export function nextOption<T extends string>(options: { value: T }[], value: T, key: string): { value: T } | undefined {
  const step = ({ ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 } as Record<string, number>)[key];
  if (step === undefined || options.length === 0) return undefined;
  const at = Math.max(0, options.findIndex((option) => option.value === value));
  return options[(at + step + options.length) % options.length];
}

type SegmentedProps<T extends string> = {
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
};

export function Segmented<T extends string>({ label, value, options, onChange }: SegmentedProps<T>) {
  const ref = useRef<HTMLDivElement>(null);
  const onKeyDown = (event: KeyboardEvent) => {
    const next = nextOption(options, value, event.key);
    if (!next) return;
    event.preventDefault();
    onChange(next.value);
    ref.current?.querySelector<HTMLElement>(`[data-value="${next.value}"]`)?.focus();
  };
  return (
    <div
      ref={ref}
      role="radiogroup"
      aria-label={label}
      onKeyDown={onKeyDown}
      className="inline-flex self-start rounded-control bg-canvas p-0.5 shadow-[inset_0_0_0_1px_var(--line)]"
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="radio"
          data-value={option.value}
          aria-checked={option.value === value}
          tabIndex={option.value === value ? 0 : -1}
          onClick={() => onChange(option.value)}
          className={cn(
            "h-6 rounded-[calc(var(--r-control)-2px)] px-3 text-small font-medium transition-quick",
            option.value === value ? "bg-surface text-fg shadow-panel" : "text-fg-muted hover:text-fg",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
