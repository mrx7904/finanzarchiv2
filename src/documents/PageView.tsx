import { type ReactNode, useEffect, useRef, useState } from "react";
import { Button, Icon, cn } from "../ui";
import { PageImage } from "./PageImage";
import { type Zoom, pageStep, requestWidth, zoomLabel, zoomStep } from "./pages";

// The page of a document with its tools under it (#54): the page and the count with back and forward, smaller and
// larger from "Ganze Seite", and "Groß lesen". A document of one page shows no paging. The page is asked from the core
// at the width it is shown, so it stays sharp at 200 %. The column keeps the room of its scroll bar, so a page that
// grows higher than its box does not narrow it and ask again (#55). `overlay` draws on the image of a page, such as the
// marked place of a question in Prüfen, and is asked per page, so a mark shows only on its own page (#57).

function useWidth() {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(640);
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => setWidth(Math.round(entry?.contentRect.width ?? 640)));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return [ref, width] as const;
}

type PageViewProps = {
  document: number;
  name: string;
  pages: number;
  page?: number;
  onLarge?: () => void;
  overlay?: (page: number) => ReactNode;
  /** Prüfen starts at the column's width, so the marked place stands on the page. */
  startZoom?: Zoom;
};

export function PageView({ document, name, pages, page: start = 1, onLarge, overlay, startZoom = "fit" }: PageViewProps) {
  const [page, setPage] = useState(start);
  const [zoom, setZoom] = useState<Zoom>(startZoom);
  const [ref, width] = useWidth();
  useEffect(() => setPage(start), [document, start]);
  // Turning to the page of another reading keeps the zoom, another document starts at its own.
  useEffect(() => setZoom(startZoom), [document, startZoom]);
  return (
    <div className="flex min-h-0 w-full flex-1 flex-col items-center gap-2">
      <div ref={ref} className={cn("relative min-h-0 w-full flex-1", zoom === "fit" ? "flex items-center justify-center overflow-hidden" : "overflow-auto [scrollbar-gutter:stable]")}>
        <Page document={document} name={name} page={page} zoom={zoom} column={width} overlay={overlay} />
      </div>
      <PageTools page={page} pages={pages} zoom={zoom} onPage={setPage} onZoom={setZoom} onLarge={onLarge} />
    </div>
  );
}

function Page({ document, name, page, zoom, column, overlay }: { document: number; name: string; page: number; zoom: Zoom; column: number; overlay?: (page: number) => ReactNode }) {
  const ratio = typeof window === "undefined" ? 1 : window.devicePixelRatio;
  const request = requestWidth(column, zoom, ratio);
  if (zoom === "fit") {
    return (
      <div className="relative flex h-full max-h-full items-center justify-center">
        <PageImage document={document} page={page} width={request} label={`Seite ${page} von ${name}`} fit overlay={overlay?.(page)} />
      </div>
    );
  }
  return (
    <div className="relative mx-auto" style={{ width: (column * zoom) / 100 }}>
      <PageImage document={document} page={page} width={request} label={`Seite ${page} von ${name}`} className="rounded-control shadow-panel" overlay={overlay?.(page)} />
    </div>
  );
}

export function PageTools({ page, pages, zoom, onPage, onZoom, onLarge }: { page: number; pages: number; zoom: Zoom; onPage: (page: number) => void; onZoom: (zoom: Zoom) => void; onLarge?: () => void }) {
  const key = "flex size-7 items-center justify-center rounded-[6px] text-fg-muted transition-quick hover:bg-hover hover:text-fg disabled:opacity-40";
  return (
    <div className="flex h-9 shrink-0 items-center gap-0.5 rounded-row bg-raised px-1 text-small whitespace-nowrap shadow-raised">
      {pages > 1 && (
        <>
          <button type="button" aria-label="Vorige Seite" disabled={page <= 1} onClick={() => onPage(pageStep(page, pages, -1))} className={key}>
            <Icon name="back" size={15} />
          </button>
          <span className="px-1.5 font-medium tabular-nums">
            {page} von {pages}
          </span>
          <button type="button" aria-label="Nächste Seite" disabled={page >= pages} onClick={() => onPage(pageStep(page, pages, 1))} className={key}>
            <Icon name="chevronRight" size={15} />
          </button>
          <span aria-hidden className="mx-1 h-4 w-px bg-line-strong" />
        </>
      )}
      <button type="button" aria-label="Kleiner" disabled={zoom === "fit"} onClick={() => onZoom(zoomStep(zoom, -1))} className={key}>
        <span aria-hidden className="text-[16px] leading-none">−</span>
      </button>
      <button type="button" onClick={() => onZoom("fit")} className="min-w-22 px-1 text-center font-medium tabular-nums">
        {zoomLabel(zoom)}
      </button>
      <button type="button" aria-label="Größer" disabled={zoom === 300} onClick={() => onZoom(zoomStep(zoom, 1))} className={key}>
        <Icon name="plus" size={15} />
      </button>
      {onLarge && (
        <>
          <span aria-hidden className="mx-1 h-4 w-px bg-line-strong" />
          <button type="button" aria-label="Groß lesen" title="Groß lesen (Leertaste)" onClick={onLarge} className={key}>
            <Icon name="upDown" size={15} />
          </button>
        </>
      )}
    </div>
  );
}

/**
 * "Groß lesen": the document over the whole window without the sidebar, the pages as small images at the left, the
 * page at its zoom, the tools in a bar at the top. Esc and "Schließen" lead back; the arrow and page keys turn pages,
 * Strg with the wheel zooms.
 */
export function ReadLarge({ document, name, pages, onClose }: { document: number; name: string; pages: number; onClose: () => void }) {
  const [page, setPage] = useState(1);
  const [zoom, setZoom] = useState<Zoom>("fit");
  const [ref, width] = useWidth();
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
      else if (event.key === "ArrowRight" || event.key === "PageDown" || event.key === "ArrowDown") setPage((at) => pageStep(at, pages, 1));
      else if (event.key === "ArrowLeft" || event.key === "PageUp" || event.key === "ArrowUp") setPage((at) => pageStep(at, pages, -1));
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [pages, onClose]);
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={`${name} groß lesen`}
      className="fixed inset-0 z-40 flex flex-col bg-canvas"
      onWheel={(event) => {
        if (!event.ctrlKey) return;
        event.preventDefault();
        setZoom((at) => zoomStep(at, event.deltaY < 0 ? 1 : -1));
      }}
    >
      <header className="flex h-(--bar-height) shrink-0 items-center gap-3 border-b border-line px-4">
        <span className="min-w-0 flex-1 truncate font-medium">{name}</span>
        <PageTools page={page} pages={pages} zoom={zoom} onPage={setPage} onZoom={setZoom} />
        <Button kind="primary" onClick={onClose}>
          Schließen
        </Button>
      </header>
      <div className="flex min-h-0 flex-1">
        {pages > 1 && (
          <nav aria-label="Seiten" className="flex w-36 shrink-0 flex-col gap-2 overflow-y-auto border-r border-line p-3">
            {Array.from({ length: pages }, (_, at) => at + 1).map((number) => (
              <button key={number} type="button" aria-label={`Seite ${number}`} aria-current={number === page ? "page" : undefined} onClick={() => setPage(number)} className={cn("rounded-[4px] p-0.5", number === page ? "ring-2 ring-accent" : "hover:bg-hover")}>
                <PageImage document={document} page={number} width={160} label={`Seite ${number}`} className="rounded-[3px] shadow-panel" />
              </button>
            ))}
          </nav>
        )}
        <div ref={ref} className={cn("min-h-0 flex-1 p-6", zoom === "fit" ? "flex items-center justify-center overflow-hidden" : "overflow-auto [scrollbar-gutter:stable]")}>
          <Page document={document} name={name} page={page} zoom={zoom} column={width - 48} />
        </div>
      </div>
    </div>
  );
}
