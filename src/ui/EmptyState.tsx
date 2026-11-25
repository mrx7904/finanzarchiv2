// What a list shows when it has nothing: an icon in a small box, one line that says what is missing, one that says what
// to do, and the action that does it. It rises in once with a short fade, so it reads as the answer to what was just
// done (a filter, an import) and not as a broken view.
// The look of the empty answer (icon in an inset box, title, hint) is adapted from Beautiful UI "Search List"
// (github.com/slev12397/beautiful-ui, components/primitives/SearchList.tsx, MIT, Copyright (c) 2026 Shane Levine),
// the calm panel with its actions is after the idea of Aceternity UI "Empty State With Stacked Cards"
// (ui.aceternity.com/blocks/empty-states, Aceternity License, idea only, its preview tiles left out). The rise is
// adapted from Magic UI "Blur Fade" (magicui.design/r/blur-fade.json, MIT), with the blur off: at 6 px it smeared text.
import type { ReactNode } from "react";
import { Icon, type IconName } from "./icons";
import { cn, m, quick } from "./motion";

type EmptyStateProps = {
  icon: IconName;
  title: string;
  hint: string;
  /** One or two buttons. */
  actions?: ReactNode;
  className?: string;
};

export function EmptyState({ icon, title, hint, actions, className }: EmptyStateProps) {
  return (
    <m.div
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ ...quick, duration: 0.3, delay: 0.04 }}
      className={cn("flex flex-col items-center justify-center gap-1 px-6 py-12 text-center", className)}
    >
      <span className="mb-2 flex size-10 items-center justify-center rounded-control bg-canvas text-fg-muted shadow-panel">
        <Icon name={icon} size={18} />
      </span>
      <p className="font-medium text-fg">{title}</p>
      <p className="max-w-80 text-fg-muted">{hint}</p>
      {actions && <div className="mt-4 flex gap-2">{actions}</div>}
    </m.div>
  );
}
