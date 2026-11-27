// How the owner fits a view to the work at hand (docs/entwurf-ansichten.md, "Anpassen"): columns dragged wider, the
// sidebar folded to a rail, a column folded away, the focus on one question, a document read large, and the size of
// the whole view. Invented data, no call to the core.
import type { ReactNode } from "react";
import { Button, Icon, Kbd, cn } from "../ui";
import { Dokumente, Frame, Page, PageTools, Pruefen, Umsaetze, panel } from "./Entwurf";

/** A document read large: the page over the whole window, its pages at the left, the tools in the top bar. */
function Lesen() {
  return (
    <Frame
      bare
      view="dokumente"
      path={["Dokumente", "2026-11-14 Stadtwerke Beispielstadt Rechnung"]}
      actions={
        <>
          <PageTools page="2 von 3" zoom="125 %" large />
          <Button kind="plain">
            Schließen <Kbd>Esc</Kbd>
          </Button>
        </>
      }
    >
      <main className="grid min-h-0 flex-1 grid-cols-[148px_minmax(0,1fr)]">
        <nav aria-label="Seiten" className="flex min-h-0 flex-col items-center gap-3 overflow-y-auto border-r border-line bg-sidebar px-4 py-5">
          {[1, 2, 3].map((number) => (
            <button key={number} type="button" aria-current={number === 2 ? "page" : undefined} className="flex flex-col items-center gap-1.5">
              <Page className={cn("w-[104px] !p-[6px] !shadow-panel", number === 2 && "outline-2 outline-offset-2 outline-accent")} />
              <span className={cn("text-small tabular-nums", number === 2 ? "font-medium text-fg" : "text-fg-muted")}>{number}</span>
            </button>
          ))}
        </nav>
        <div className="flex min-h-0 justify-center overflow-hidden bg-canvas px-10 pt-8">
          <Page className="w-[860px] shrink-0" />
        </div>
      </main>
    </Frame>
  );
}

/** The whole view larger or smaller: text, fields and charts grow together. The size is the zoom of the window (Klein
 * 90 %, Mittel 100 %, Groß 115 %), so the page draws nothing itself: to see it, zoom the browser with Strg and +. */
function Groesse({ size, children }: { size: string; children: ReactNode }) {
  return (
    <div className="relative h-full">
      {children}
      <div className="pointer-events-none absolute inset-x-0 top-[calc(var(--bar-height)+12px)] z-40 flex justify-center">
        <div className={cn(panel, "flex h-10 items-center gap-2.5 !bg-raised px-3 !shadow-raised")}>
          <Icon name="search" size={15} className="text-fg-muted" />
          <span className="font-medium">Größe: {size}</span>
          <span className="flex items-center gap-1 text-small text-fg-muted">
            <Kbd>Strg</Kbd>
            <Kbd>+</Kbd>
            <Kbd>−</Kbd>
            <Kbd>0</Kbd>
          </span>
        </div>
      </div>
    </div>
  );
}

export const anpassen = {
  "dokumente-ziehen": () => <Dokumente resize />,
  "dokumente-leiste": () => <Dokumente rail />,
  "dokumente-ohne-angaben": () => <Dokumente values={false} />,
  "dokumente-leiste-ohne-angaben": () => <Dokumente rail values={false} />,
  "umsaetze-ohne-detail": () => <Umsaetze detail={false} />,
  "pruefen-fokus": () => <Pruefen focus />,
  "dokument-lesen": () => <Lesen />,
  "dokumente-gross": () => (
    <Groesse size="Groß">
      <Dokumente />
    </Groesse>
  ),
  "dokumente-klein": () => (
    <Groesse size="Klein">
      <Dokumente />
    </Groesse>
  ),
} satisfies Record<string, () => ReactNode>;
