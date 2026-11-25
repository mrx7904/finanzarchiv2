// A field with its label above, a hint or an error below, and optionally an icon in front and a cross that empties
// it. The select is the same box with the up and down chevron of a pop-up button, so both line up in a form.
// The field with the leading icon and the clear action is adapted from Beautiful UI "Search List"
// (github.com/slev12397/beautiful-ui, components/primitives/SearchList.tsx, MIT, Copyright (c) 2026 Shane Levine):
// its input row and the round clear button, rewritten to the tokens, a label and German texts.
import { useId, type InputHTMLAttributes, type ReactNode, type SelectHTMLAttributes } from "react";
import { Icon, type IconName } from "./icons";
import { cn } from "./motion";

const box =
  "flex h-(--control-height) w-full items-center gap-2 rounded-control bg-field px-2.5 text-body text-fg shadow-[inset_0_0_0_1px_var(--line-strong)] transition-quick " +
  "focus-within:shadow-[inset_0_0_0_1px_var(--accent)] focus-within:outline-3 focus-within:outline-(--focus)";

type FrameProps = { id: string; label: string; hint?: string; error?: string; children: ReactNode; className?: string };

function FieldFrame({ id, label, hint, error, children, className }: FrameProps) {
  return (
    <div className={cn("flex min-w-0 flex-col gap-1", className)}>
      <label htmlFor={id} className="text-small font-medium text-fg-muted">
        {label}
      </label>
      {children}
      {error ? (
        <p id={`${id}-note`} className="text-small text-danger">
          {error}
        </p>
      ) : (
        hint && (
          <p id={`${id}-note`} className="text-small text-fg-muted">
            {hint}
          </p>
        )
      )}
    </div>
  );
}

type TextFieldProps = Omit<InputHTMLAttributes<HTMLInputElement>, "id" | "onChange" | "value"> & {
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: string;
  error?: string;
  icon?: IconName;
  /** Shows a cross that empties the field once it holds text. */
  clearable?: boolean;
  /** Text after the value, such as "€". */
  suffix?: string;
};

export function TextField({ label, value, onChange, hint, error, icon, clearable = false, suffix, className, ...props }: TextFieldProps) {
  const id = useId();
  return (
    <FieldFrame id={id} label={label} hint={hint} error={error} className={className}>
      <div className={cn(box, error && "shadow-[inset_0_0_0_1px_var(--danger)]")}>
        {icon && <Icon name={icon} size={15} className="shrink-0 text-fg-muted" />}
        <input
          id={id}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          aria-invalid={error ? true : undefined}
          aria-describedby={error || hint ? `${id}-note` : undefined}
          className="min-w-0 flex-1 bg-transparent text-body text-fg outline-none placeholder:text-fg-muted"
          {...props}
        />
        {suffix && <span className="shrink-0 text-fg-muted">{suffix}</span>}
        {clearable && value && (
          <button
            type="button"
            aria-label="Leeren"
            onClick={() => onChange("")}
            className="-mr-1 flex size-5 shrink-0 items-center justify-center rounded-full text-fg-muted transition-quick hover:bg-hover hover:text-fg"
          >
            <Icon name="close" size={11} strokeWidth={2.2} />
          </button>
        )}
      </div>
    </FieldFrame>
  );
}

type SelectFieldProps = Omit<SelectHTMLAttributes<HTMLSelectElement>, "id" | "onChange" | "value"> & {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
  hint?: string;
  error?: string;
};

export function SelectField({ label, value, onChange, options, hint, error, className, ...props }: SelectFieldProps) {
  const id = useId();
  return (
    <FieldFrame id={id} label={label} hint={hint} error={error} className={className}>
      <div className={cn(box, "relative pr-0")}>
        <select
          id={id}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          aria-describedby={error || hint ? `${id}-note` : undefined}
          className="h-full min-w-0 flex-1 appearance-none bg-transparent pr-7 text-body text-fg outline-none"
          {...props}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <Icon name="upDown" size={14} className="pointer-events-none absolute right-2 text-fg-muted" />
      </div>
    </FieldFrame>
  );
}
