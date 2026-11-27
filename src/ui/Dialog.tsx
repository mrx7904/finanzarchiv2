// A dialog on the native `<dialog>` of the webview, opened with `showModal()`: the browser keeps the focus inside,
// makes the rest of the window inert, closes on Escape and returns the focus to where it was. Three widths, `narrow`
// for a question, `regular` for a form, `wide` for a list or a preview.
// The entry after the idea of Aceternity UI "Animated Modal" (ui.aceternity.com/components/animated-modal, Aceternity
// License: its source must not be re-distributed, so only the idea is taken): the backdrop fades in and the panel
// grows from .98. Its 3D tilt and spring bounce are left out. The entry is CSS (`@starting-style`), so it needs no
// package and stops with every other transition under reduced motion.
import { useEffect, useId, useRef, type ReactNode } from "react";
import { Button } from "./Button";
import { cn } from "./motion";

const widths = {
  narrow: "w-[min(92vw,28rem)]",
  regular: "w-[min(92vw,40rem)]",
  wide: "w-[min(94vw,72rem)]",
} as const;

/**
 * The focus of an opened dialog (#45): its first field, else its safe action (the first button of its foot that does
 * not remove and is not the main action), else the main action. Never the close button.
 */
export function focusFirst(dialog: HTMLElement): void {
  const field = dialog.querySelector<HTMLElement>("[data-autofocus], input:not([type='hidden']), textarea, select");
  const buttons = [...dialog.querySelectorAll<HTMLElement>("footer button")];
  const safe = buttons.find((button) => button.dataset.kind !== "danger" && button.dataset.kind !== "primary");
  (field ?? safe ?? buttons.at(-1))?.focus();
}

/** What a dialog that removes or moves lists below its question: what goes and what stays (rule 18). */
export function Consequences({ goes, stays }: { goes: string[]; stays?: string[] }) {
  return (
    <div className="flex flex-col gap-3 text-body">
      <ConsequenceList title="Geht" items={goes} dot="bg-danger" />
      {stays && stays.length > 0 && <ConsequenceList title="Bleibt" items={stays} dot="bg-ok" />}
    </div>
  );
}

function ConsequenceList({ title, items, dot }: { title: string; items: string[]; dot: string }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-small font-medium text-fg-muted">{title}</span>
      <ul className="flex flex-col gap-1">
        {items.map((item) => (
          <li key={item} className="flex items-start gap-2 text-fg">
            <span aria-hidden className={cn("mt-2 size-1.5 shrink-0 rounded-full", dot)} />
            {item}
          </li>
        ))}
      </ul>
    </div>
  );
}

type DialogProps = {
  open: boolean;
  onClose: () => void;
  title: string;
  /** One sentence under the title. */
  description?: string;
  size?: keyof typeof widths;
  /** The buttons at the bottom right, the main action last. */
  actions?: ReactNode;
  children?: ReactNode;
};

export function Dialog({ open, onClose, title, description, size = "regular", actions, children }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const descriptionId = useId();

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open && !dialog.open) {
      dialog.showModal();
      focusFirst(dialog);
    }
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      aria-describedby={description ? descriptionId : undefined}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClick={(event) => {
        // A click on the backdrop lands on the dialog element itself, outside its panel.
        if (event.target === event.currentTarget) onClose();
      }}
      className={cn(
        "fa-dialog m-auto max-h-[min(88vh,56rem)] overflow-hidden rounded-dialog bg-raised p-0 text-fg shadow-raised",
        widths[size],
      )}
    >
      {open && (
        <div className="flex max-h-[inherit] flex-col">
          <header className="flex items-start gap-3 px-5 pt-5 pb-3">
            <div className="flex min-w-0 flex-1 flex-col gap-1">
              <h2 id={titleId} className="text-heading font-semibold">
                {title}
              </h2>
              {description && (
                <p id={descriptionId} className="text-fg-muted">
                  {description}
                </p>
              )}
            </div>
            <Button kind="plain" size="small" icon="close" label="Schließen" onClick={onClose} className="-mt-0.5 -mr-1.5" />
          </header>
          {children && <div className="min-h-0 overflow-y-auto px-5 pb-4">{children}</div>}
          {actions && <footer className="flex justify-end gap-2 border-t border-line px-5 py-3">{actions}</footer>}
        </div>
      )}
    </dialog>
  );
}
