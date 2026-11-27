// The draft of the views (docs/entwurf-ansichten.md): every view of the map drawn at real size with the pieces of
// the design system and invented data, so the owner can look and choose before the views are built. Nothing here
// calls the core and nothing is part of the app: the page is opened in the dev server at /src/entwurf/index.html,
// with ?v=<view> and ?t=light or dark. No name, number or amount is real (decision-public-repository.md).
import { useState, type ReactNode } from "react";
import { AppShell, Button, Capsule, Icon, Kbd, List, ListRow, Menu, SelectField, Spinner, TextField, cn, type IconName, type MenuEntry, type PathSegment, type Tone } from "../ui";

export type ViewId = "start" | "pruefen" | "dokumente" | "konten" | "umsaetze" | "auswertungen" | "steuern" | "chat" | "einstellungen";

// Three icons the app does not have yet, on the grid and stroke of src/ui/icons.tsx.
const glyphs = {
  chart: <path d="M3.5 16.5h13M6 13.5v-4M10 13.5v-9M14 13.5v-6" />,
  chat: <path d="M4.5 4h11a1 1 0 0 1 1 1v7.5a1 1 0 0 1-1 1H9l-3.5 3v-3h-1a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z" />,
  back: <path d="M11.5 5 6.5 10l5 5" />,
  sidebar: <path d="M3.5 4.5h13a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1h-13a1 1 0 0 1-1-1v-9a1 1 0 0 1 1-1zM7.5 4.5v11" />,
  values: <path d="M3.5 4.5h13a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1h-13a1 1 0 0 1-1-1v-9a1 1 0 0 1 1-1zM12.5 4.5v11" />,
  expand: <path d="M11.5 3.5h5v5M8.5 16.5h-5v-5M16.5 3.5l-5.5 5.5M3.5 16.5 9 11" />,
  minus: <path d="M4 10h12" />,
};

export function Glyph({ name, size = 16, className }: { name: keyof typeof glyphs; size?: number; className?: string }) {
  return (
    <svg className={cn("shrink-0", className)} width={size} height={size} viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      {glyphs[name]}
    </svg>
  );
}

type NavEntry = { id: ViewId; label: string; icon: IconName | keyof typeof glyphs; count?: number };

const navigation: NavEntry[][] = [
  [
    { id: "start", label: "Start", icon: "home" },
    { id: "pruefen", label: "Prüfen", icon: "inbox", count: 4 },
  ],
  [
    { id: "dokumente", label: "Dokumente", icon: "document" },
    { id: "konten", label: "Konten", icon: "card" },
  ],
  [
    { id: "auswertungen", label: "Auswertungen", icon: "chart" },
    { id: "steuern", label: "Steuern", icon: "scale" },
    { id: "chat", label: "Chat", icon: "chat" },
  ],
];

function NavRow({ entry, selected }: { entry: Omit<NavEntry, "id">; selected: boolean }) {
  const icon = entry.icon in glyphs ? <Glyph name={entry.icon as keyof typeof glyphs} className={selected ? "text-fg-accent" : "text-fg-muted"} /> : <Icon name={entry.icon as IconName} size={16} className={selected ? "text-fg-accent" : "text-fg-muted"} />;
  return (
    <button type="button" aria-current={selected ? "page" : undefined} className={cn("relative flex h-8 w-full items-center gap-2.5 rounded-row px-2 text-left transition-quick", selected ? "bg-selected font-medium text-fg" : "text-fg hover:bg-hover")}>
      {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
      {icon}
      <span className="flex-1">{entry.label}</span>
      {entry.count !== undefined && <span className="rounded-full bg-accent px-1.5 text-small font-semibold text-on-accent tabular-nums">{entry.count}</span>}
    </button>
  );
}

/** The frame every view stands in: the same sidebar, the path and the actions of the view in the bar. */
export function Frame({ view, path, actions, back, intake = false, rail = false, bare = false, children }: { view: ViewId; path: string[]; actions?: ReactNode; back?: string; intake?: boolean; rail?: boolean; bare?: boolean; children: ReactNode }) {
  const segments: PathSegment[] = path.map((label, at) => (at < path.length - 1 ? { label, onSelect: () => undefined } : { label }));
  const here = view === "umsaetze" ? "konten" : view;
  if (bare) {
    return (
      <AppShell system="windows" path={segments} actions={actions}>
        {children}
      </AppShell>
    );
  }
  const toggle = (
    <button type="button" aria-label={rail ? "Seitenleiste ausklappen" : "Seitenleiste einklappen"} title={rail ? "Seitenleiste ausklappen (Strg+B)" : "Seitenleiste einklappen (Strg+B)"} className="flex size-7 shrink-0 items-center justify-center rounded-control text-fg-muted hover:bg-hover hover:text-fg">
      <Glyph name="sidebar" />
    </button>
  );
  return (
    <div className={cn("relative h-full", back && "[&_header]:pl-14", rail && "[--sidebar-width:56px] [&_aside>div:first-child]:justify-center [&_aside>div:first-child]:pl-0")}>
      {back && (
        <button type="button" aria-label={`Zurück zu ${back}`} title={`Zurück zu ${back}`} className="absolute top-[calc((var(--bar-height)-28px)/2)] left-[calc(var(--sidebar-width)+16px)] z-10 flex size-7 items-center justify-center rounded-control text-fg hover:bg-hover">
          <Glyph name="back" size={18} />
        </button>
      )}
      {intake && <IntakeOpen />}
      <AppShell
      system="windows"
      path={segments}
      actions={actions}
      appName={
        rail ? (
          toggle
        ) : (
          <>
            <span className="flex size-6 items-center justify-center rounded-[6px] bg-accent text-on-accent">
              <Icon name="folder" size={14} />
            </span>
            <span className="flex-1 font-semibold">Finanzarchiv</span>
            <span className="mr-2.5">{toggle}</span>
          </>
        )
      }
      sidebar={
        rail ? (
          <Rail here={here} settings={view === "einstellungen"} />
        ) : (
        <div className="flex min-h-0 flex-1 flex-col gap-4 px-3 pb-3">
          <button type="button" className="flex h-(--control-height) items-center gap-2 rounded-control bg-field px-2.5 text-fg-muted shadow-[inset_0_0_0_1px_var(--line)]">
            <Icon name="search" size={15} />
            <span className="flex-1 text-left">Suchen</span>
            <Kbd>Strg</Kbd>
            <Kbd>K</Kbd>
          </button>
          <nav aria-label="Ansichten" className="flex flex-1 flex-col gap-4">
            {navigation.map((group) => (
              <ul key={group[0]?.id} className="flex flex-col gap-px">
                {group.map((entry) => (
                  <li key={entry.id}>
                    <NavRow entry={entry} selected={entry.id === here} />
                  </li>
                ))}
              </ul>
            ))}
          </nav>
          <div className="flex flex-col gap-px">
            <button type="button" className={cn("flex h-8 w-full items-center gap-2.5 rounded-row px-2 text-left text-fg-muted hover:bg-hover", intake && "bg-selected text-fg")}>
              <Spinner size={16} />
              <span className="flex-1">3 werden gelesen</span>
            </button>
            <NavRow entry={{ label: "Einstellungen", icon: "gear" }} selected={view === "einstellungen"} />
          </div>
        </div>
        )
      }
    >
      {children}
      </AppShell>
    </div>
  );
}

/** The sidebar folded to a rail of icons: the count of "Prüfen" and the intake stay in sight. */
function Rail({ here, settings }: { here: ViewId; settings: boolean }) {
  const cell = "relative flex size-8 items-center justify-center rounded-row";
  const icon = (entry: Omit<NavEntry, "id">, selected: boolean) =>
    entry.icon in glyphs ? <Glyph name={entry.icon as keyof typeof glyphs} className={selected ? "text-fg-accent" : "text-fg-muted"} /> : <Icon name={entry.icon as IconName} size={16} className={selected ? "text-fg-accent" : "text-fg-muted"} />;
  return (
    <div className="flex min-h-0 flex-1 flex-col items-center gap-4 pb-3">
      <button type="button" aria-label="Suchen" title="Suchen (Strg+K)" className={cn(cell, "bg-field text-fg-muted shadow-[inset_0_0_0_1px_var(--line)]")}>
        <Icon name="search" size={15} />
      </button>
      <nav aria-label="Ansichten" className="flex flex-1 flex-col gap-4">
        {navigation.map((group) => (
          <ul key={group[0]?.id} className="flex flex-col gap-px">
            {group.map((entry) => (
              <li key={entry.id}>
                <button type="button" aria-label={entry.label} title={entry.label} aria-current={entry.id === here ? "page" : undefined} className={cn(cell, entry.id === here ? "bg-selected" : "hover:bg-hover")}>
                  {icon(entry, entry.id === here)}
                  {entry.count !== undefined && <span className="absolute -top-0.5 -right-1 rounded-full bg-accent px-1 text-[10px] leading-[14px] font-semibold text-on-accent tabular-nums">{entry.count}</span>}
                </button>
              </li>
            ))}
          </ul>
        ))}
      </nav>
      <div className="flex flex-col gap-px">
        <button type="button" aria-label="Eingang, 3 werden gelesen" title="Eingang, 3 werden gelesen" className={cn(cell, "text-fg-muted hover:bg-hover")}>
          <Spinner size={16} />
        </button>
        <button type="button" aria-label="Einstellungen" title="Einstellungen" className={cn(cell, settings ? "bg-selected" : "hover:bg-hover")}>
          {icon({ label: "Einstellungen", icon: "gear" }, settings)}
        </button>
      </div>
    </div>
  );
}

/** A button of the top bar that shows or hides a column and stays pressed while the column is in sight. */
export function Toggle({ glyph, label, on }: { glyph: keyof typeof glyphs; label: string; on: boolean }) {
  return (
    <button type="button" aria-label={label} title={label} aria-pressed={on} className={cn("flex size-(--control-height) shrink-0 items-center justify-center rounded-control transition-quick hover:bg-hover", on ? "bg-selected text-fg" : "text-fg-muted")}>
      <Glyph name={glyph} />
    </button>
  );
}

/** The line between two columns while the owner drags it. */
function Divider() {
  return (
    <span role="separator" aria-orientation="vertical" aria-label="Breite der Liste" className="absolute top-3 -right-[12px] bottom-3 z-10 flex w-1 cursor-col-resize items-center justify-center rounded-full bg-accent">
      <span className="absolute top-2 left-3 rounded-control bg-raised px-2 py-1 text-small whitespace-nowrap text-fg shadow-raised">470 px, Doppelklick stellt zurück</span>
    </span>
  );
}

/** The filter of the states open, drawn still: the states to filter by, and below a line the choice to hide them. */
function StateChoice() {
  const row = "relative z-10 flex h-8 w-full items-center gap-2 rounded-row px-2 text-left";
  const states: [string, string, string][] = [
    ["bg-warn", "prüfen", "4"],
    ["bg-danger", "doppelt?", "1"],
    ["bg-ok", "bestätigt", "212"],
    ["bg-fg-muted", "gelesen", "1.067"],
  ];
  return (
    <div role="listbox" aria-label="Zustand" className="absolute top-[calc(100%+4px)] right-0 left-0 z-20 flex flex-col rounded-panel bg-raised p-1 shadow-raised">
      <div className={row}>
        <Icon name="check" size={14} />
        <span className="flex-1 font-medium">Jeder Zustand</span>
      </div>
      {states.map(([dot, label, count]) => (
        <div key={label} className={row}>
          <span className="flex w-3.5 justify-center">
            <span className={cn("size-2 rounded-full", dot)} />
          </span>
          <span className="flex-1">{label}</span>
          <span className="text-small text-fg-muted tabular-nums">{count}</span>
        </div>
      ))}
      <span aria-hidden className="mx-2 my-1 h-px bg-line" />
      <div className={cn(row, "bg-selected")}>
        <span className="w-3.5" />
        <span className="flex-1">Zustand ausblenden</span>
      </div>
    </div>
  );
}

/** Under the page of a document: its pages, its zoom, and the way into reading it large. */
export function PageTools({ page = "1 von 3", zoom = "Ganze Seite", large = false }: { page?: string; zoom?: string; large?: boolean }) {
  const key = "flex size-7 items-center justify-center rounded-[6px] text-fg-muted hover:bg-hover hover:text-fg";
  return (
    <div className="flex h-9 shrink-0 items-center gap-0.5 rounded-row bg-raised px-1 text-small whitespace-nowrap shadow-raised">
      <button type="button" aria-label="Vorige Seite" className={key}>
        <Glyph name="back" size={15} />
      </button>
      <span className="px-1.5 font-medium tabular-nums">{page}</span>
      <button type="button" aria-label="Nächste Seite" className={key}>
        <Icon name="chevronRight" size={15} />
      </button>
      <span aria-hidden className="mx-1 h-4 w-px bg-line-strong" />
      <button type="button" aria-label="Kleiner" className={key}>
        <Glyph name="minus" size={15} />
      </button>
      <span className="px-1 text-center font-medium tabular-nums">{zoom}</span>
      <button type="button" aria-label="Größer" className={key}>
        <Icon name="plus" size={15} />
      </button>
      {!large && (
        <>
          <span aria-hidden className="mx-1 h-4 w-px bg-line-strong" />
          <button type="button" aria-label="Groß lesen" title="Groß lesen (Leertaste)" className={key}>
            <Glyph name="expand" size={15} />
          </button>
        </>
      )}
    </div>
  );
}

/** The intake opened from the foot of the sidebar: what is read right now, what was refused and why, and the pause. */
function IntakeOpen() {
  const dot = "flex w-3.5 shrink-0 justify-center";
  return (
    <div className="absolute bottom-[92px] left-3 z-20 flex w-[380px] flex-col rounded-dialog bg-raised shadow-raised">
      <div className="flex items-center justify-between px-4 pt-3.5 pb-2">
        <span className="text-heading font-semibold">Eingang</span>
        <Button size="small">Pausieren</Button>
      </div>
      <div className="flex flex-col px-4 pb-2">
        {[
          ["Rechnung_Nov.pdf", "wird gelesen"],
          ["Kontoauszug_10_2026.pdf", "Buchungen werden geprüft"],
          ["Scan_0031.pdf", "wartet"],
        ].map(([name, stage]) => (
          <div key={name} className="flex h-9 items-center gap-2.5">
            {stage === "wartet" ? (
              <span className={dot}>
                <span className="size-1.5 rounded-full bg-fg-muted" />
              </span>
            ) : (
              <Spinner />
            )}
            <span className="min-w-0 flex-1 truncate">{name}</span>
            <span className="text-small text-fg-muted">{stage}</span>
          </div>
        ))}
      </div>
      <div className="flex flex-col gap-1 border-t border-line px-4 py-3">
        <span className="text-small font-medium text-fg-muted">Nicht angenommen</span>
        <div className="flex items-center gap-2.5">
          <span className={dot}>
            <span className="size-2 rounded-full bg-danger" />
          </span>
          <span className="min-w-0 flex-1 truncate">Urlaub_2026.xlsx</span>
          <span className="text-small text-fg-muted">Tabellen liest das Archiv nicht</span>
        </div>
        <div>
          <Button size="small" kind="plain" className="-ml-2">
            Ausblenden
          </Button>
        </div>
      </div>
      <div className="border-t border-line px-4 py-2.5 text-small text-fg-muted">Heute: 23 Dokumente gelesen, 19 ohne dich abgelegt</div>
    </div>
  );
}

export const panel = "rounded-panel bg-surface shadow-panel";

const nothing = () => undefined;

/** The menu behind "…": everything a view or a piece can do beyond its one or two buttons. */
export function More({ entries, small = false }: { entries: MenuEntry[]; small?: boolean }) {
  return (
    <Menu
      align="end"
      entries={entries}
      trigger={(props) => (
        <button type="button" aria-label="Weitere Aktionen" {...props} className={cn("flex shrink-0 items-center justify-center rounded-control text-fg transition-quick hover:bg-hover aria-expanded:bg-selected", small ? "size-6" : "size-(--control-height)")}>
          <Icon name="more" size={small ? 14 : 16} />
        </button>
      )}
    />
  );
}

export function Heading({ children, aside }: { children: ReactNode; aside?: ReactNode }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <h2 className="text-heading font-semibold">{children}</h2>
      {aside ? <span className="text-small text-fg-muted">{aside}</span> : null}
    </div>
  );
}

/** A drawn invoice in place of the first page of a document, with the line a question is about marked. */
export function Page({ marked = false, className }: { marked?: boolean; className?: string }) {
  return (
    <div className={cn("flex aspect-[210/297] flex-col overflow-hidden bg-white p-[7%] font-serif leading-[1.25] text-[#18181b] shadow-raised [container-type:inline-size]", className)}>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-[3.4cqw] font-bold">Stadtwerke Beispielstadt</p>
          <p className="mt-[1.4cqw] text-[1.9cqw]">Musterweg 1, 12345 Beispielstadt</p>
        </div>
        <span className="size-[8.5cqw] rounded-full bg-[#e4e4e7]" />
      </div>
      <p className="mt-[11cqw] text-[4cqw] font-bold">Rechnung</p>
      <p className="mt-[2.4cqw] text-[2.2cqw]">Nr. SW-2026-0413 vom 14.11.2026</p>
      <div className="mt-[6cqw] flex flex-col gap-[2cqw]">
        {[48, 36, 52, 30].map((width) => (
          <div key={width} className="flex items-center justify-between">
            <span className="h-[1.3cqw] rounded-full bg-[#e4e4e7]" style={{ width: `${width}%` }} />
            <span className="h-[1.3cqw] w-[12%] rounded-full bg-[#e4e4e7]" />
          </div>
        ))}
      </div>
      <div className="relative mt-[4cqw] flex items-center justify-between border-t border-[#d4d4d8] pt-[3.4cqw] text-[2.4cqw] font-bold">
        <span>Gesamtbetrag</span>
        <span className="relative">
          {marked && <span aria-hidden className="absolute -inset-x-[1.4cqw] -inset-y-[0.9cqw] rounded-[3px] bg-[#18181b]/8 shadow-[0_0_0_2px_#18181b]" />}
          86,40 €
        </span>
      </div>
    </div>
  );
}

export function Segments({ options, value }: { options: string[]; value: string }) {
  return (
    <div className="flex h-(--control-height) items-center gap-px rounded-control bg-canvas p-0.5 shadow-[inset_0_0_0_1px_var(--line)]">
      {options.map((option) => (
        <span key={option} className={cn("flex h-full items-center rounded-[6px] px-2.5 text-small font-medium", option === value ? "bg-surface text-fg shadow-panel" : "text-fg-muted")}>
          {option}
        </span>
      ))}
    </div>
  );
}

// ---------- Start ----------

const steps: { icon: IconName; title: string; detail: string; action: string; tone: Tone }[] = [
  { icon: "inbox", title: "4 Dokumente warten auf deine Antwort", detail: "Zwei Beträge, eine Art, ein mögliches Doppel", action: "Prüfen", tone: "warn" },
  { icon: "card", title: "Im Girokonto fehlt der Kontoauszug vom September", detail: "Zwischen dem 31.08. und dem 01.10. liegt keine Buchung", action: "Importieren …", tone: "danger" },
  { icon: "receipt", title: "2 Bestellungen bei Amazon ohne Rechnung", detail: "Bestellt am 02.11. und am 09.11.2026", action: "Ansehen", tone: "warn" },
  { icon: "scale", title: "Zwei Abos für Musik laufen gleichzeitig", detail: "Tonspur und Klangwelt, zusammen 21,98 € im Monat", action: "Ansehen", tone: "neutral" },
];

const filed: [string, string, string][] = [
  ["Stadtwerke Beispielstadt", "Rechnung, bezahlt am 18.11. vom Girokonto", "86,40 €"],
  ["Beispielwerk GmbH", "Gehaltsabrechnung Oktober, passt zum Eingang vom 27.10.", "3.480,00 €"],
  ["Musterbank", "Kontoauszug Oktober, 31 Buchungen, Saldo stimmt", ""],
  ["Tonspur", "Rechnung aus dem Postfach, bezahlt über PayPal", "10,99 €"],
];

const months = [
  [31, 24], [31, 27], [31, 22], [33, 30], [31, 25], [31, 21], [36, 34], [31, 26], [31, 23], [31, 28], [34, 19], [0, 0],
] as const;

export function MonthBars({ height, chosen }: { height: number; chosen?: number }) {
  return (
    <div className="flex items-end gap-[3%]" style={{ height }}>
      {months.map(([income, spending], at) => (
        <div key={at} className="flex h-full flex-1 items-end gap-[8%]">
          <span className="flex-1 rounded-t-[3px] bg-line-strong" style={{ height: `${(income / 36) * 100}%` }} />
          <span className={cn("flex-1 rounded-t-[3px]", chosen === undefined || chosen === at ? "bg-accent" : "bg-fg-muted")} style={{ height: `${(spending / 36) * 100}%` }} />
        </div>
      ))}
    </div>
  );
}

function Figure({ label, value, note }: { label: string; value: string; note: string }) {
  return (
    <button type="button" className="group flex w-full items-end justify-between gap-3 border-t border-line py-4 text-left first:border-t-0 first:pt-0">
      <span className="flex flex-col">
        <span className="text-small text-fg-muted">{label}</span>
        <span className="text-title font-semibold tracking-[-0.025em] tabular-nums">{value}</span>
        <span className="text-small text-fg-muted">{note}</span>
      </span>
      <Icon name="chevronRight" size={16} className="mb-1.5 text-fg-muted" />
    </button>
  );
}

function Start({ intake = false }: { intake?: boolean }) {
  return (
    <Frame view="start" path={["Start"]} intake={intake}>
      <main className="min-h-0 flex-1 overflow-y-auto px-10 py-9">
        <div className="grid max-w-[1120px] grid-cols-1 gap-10 min-[1300px]:grid-cols-[minmax(0,1fr)_320px] min-[1300px]:gap-14">
          <section className="flex flex-col gap-6">
            <header className="flex flex-col gap-1">
              <h1 className="text-title font-semibold tracking-[-0.025em]">Vier Dinge warten auf dich.</h1>
              <p className="text-fg-muted">Seit Montag kamen 23 Dokumente, 19 davon sind ohne dich abgelegt.</p>
            </header>
            <div className={cn(panel, "flex flex-col divide-y divide-line")}>
              {steps.map((step, at) => (
                <div key={step.title} className="flex items-center gap-4 px-4 py-3.5">
                  <span className="relative flex size-9 shrink-0 items-center justify-center rounded-control bg-canvas text-fg-muted">
                    <Icon name={step.icon} size={17} />
                    <span aria-hidden className={cn("absolute -top-0.5 -right-0.5 size-2.5 rounded-full ring-2 ring-surface", step.tone === "danger" ? "bg-danger" : step.tone === "warn" ? "bg-warn" : "bg-fg-muted")} />
                  </span>
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate font-medium">{step.title}</span>
                    <span className="truncate text-small text-fg-muted">{step.detail}</span>
                  </span>
                  <Button kind={at === 0 ? "primary" : "secondary"}>{step.action}</Button>
                </div>
              ))}
            </div>
            <div className="flex flex-col gap-2 pt-2">
              <Heading aside="19 seit Montag">Ohne dich abgelegt</Heading>
              <div className="flex flex-col divide-y divide-line">
                {filed.map(([name, detail, amount]) => (
                  <div key={name} className="flex h-10 items-center gap-3">
                    <span aria-hidden className="size-2 shrink-0 rounded-full bg-fg-muted" />
                    <span className="w-48 shrink-0 truncate font-medium min-[1300px]:w-64">{name}</span>
                    <span className="min-w-0 flex-1 truncate text-small text-fg-muted">{detail}</span>
                    <span className="font-medium tabular-nums">{amount}</span>
                  </div>
                ))}
              </div>
              <div>
                <Button size="small" kind="plain">
                  Alle 19 ansehen
                </Button>
              </div>
            </div>
          </section>
          <aside className="flex flex-col gap-5 pt-1.5">
            <Heading aside="bis 27.11.">2026</Heading>
            <div className="grid grid-cols-3 gap-8 min-[1300px]:flex min-[1300px]:flex-col min-[1300px]:gap-0 max-[1299px]:[&>*]:border-t-0 max-[1299px]:[&>*]:py-0">
              <Figure label="Einnahmen" value="38.412,60 €" note="2.140,00 € mehr als 2025 bis November" />
              <Figure label="Ausgaben" value="29.870,14 €" note="1.312,48 € weniger als 2025 bis November" />
              <Figure label="Auf allen Konten" value="12.604,33 €" note="Stand der Kontoauszüge vom 31.10." />
            </div>
            <div className="hidden flex-col gap-2 border-t border-line pt-4 min-[1300px]:flex">
              <MonthBars height={72} />
              <div className="flex justify-between text-small text-fg-muted">
                <span>Jan</span>
                <span>Einnahmen hell, Ausgaben dunkel</span>
                <span>Dez</span>
              </div>
            </div>
          </aside>
        </div>
      </main>
    </Frame>
  );
}

// ---------- Prüfen ----------

export function Value({ label, value, source, tone = "ok" }: { label: string; value: string; source: string; tone?: Tone }) {
  return (
    <div className="flex items-center gap-3 py-2.5">
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-small text-fg-muted">{label}</span>
        <span className="truncate font-medium tabular-nums">{value}</span>
      </span>
      <span className="flex items-center gap-1.5 text-small text-fg-muted">
        <span aria-hidden className={cn("size-2 rounded-full", tone === "ok" ? "bg-ok" : tone === "warn" ? "bg-warn" : "bg-fg-muted")} />
        {source}
      </span>
    </div>
  );
}

export function Pruefen({ focus = false }: { focus?: boolean }) {
  return (
    <Frame
      bare={focus}
      view="pruefen"
      path={["Prüfen", "Betrag unklar", "Stadtwerke Beispielstadt"]}
      actions={
        <>
          <span className="text-small text-fg-muted tabular-nums">Frage 2 von 4</span>
          <Button>Überspringen</Button>
          {focus ? (
            <Button kind="plain">
              Fokus beenden <Kbd>Esc</Kbd>
            </Button>
          ) : (
            <button type="button" aria-label="Fokus" title="Fokus: nur Seite und Frage (F)" className="flex size-(--control-height) shrink-0 items-center justify-center rounded-control text-fg-muted hover:bg-hover hover:text-fg">
              <Glyph name="expand" />
            </button>
          )}
        </>
      }
    >
      <main className={cn("grid min-h-0 flex-1 gap-5 p-5", focus ? "mx-auto w-full max-w-[1500px] grid-cols-[minmax(0,1fr)_480px]" : "grid-cols-[minmax(0,1fr)_440px]")}>
        <section aria-label="Seite" className={cn(panel, "flex min-h-0 items-center justify-center overflow-hidden p-5")}>
          <div className="flex size-full items-center justify-center [container-type:size]">
            <Page marked className="h-[min(100cqh,141.4cqw)]" />
          </div>
        </section>
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex flex-col gap-5 p-6">
            <div className="flex flex-col gap-2">
              <h1 className="text-title font-semibold tracking-[-0.025em]">Ist 86,40 € der Betrag dieser Rechnung?</h1>
              <p className="text-fg-muted">Die Positionen ergeben zusammen 84,60 €. Die markierte Stelle nennt 86,40 €.</p>
            </div>
            <div className="flex items-center gap-2">
              <Button kind="primary" className="min-w-24">
                Ja
              </Button>
              <Button className="min-w-24">Nein</Button>
              <span className="ml-auto flex items-center gap-1.5 text-small text-fg-muted">
                <Kbd>J</Kbd>
                <Kbd>N</Kbd>
              </span>
            </div>
          </div>
          <div className="flex flex-col border-t border-line px-6 py-3">
            <p className="pt-1.5 pb-1 text-small font-medium text-fg-muted">Schon sicher</p>
            <div className="flex flex-col divide-y divide-line">
              <Value label="Absender" value="Stadtwerke Beispielstadt" source="bekannt" />
              <Value label="Rechnungsdatum" value="14.11.2026" source="geprüft" />
              <Value label="Nummer" value="SW-2026-0413" source="geprüft" />
            </div>
          </div>
          <div className="mt-auto flex items-center justify-between border-t border-line px-6 py-3 text-small text-fg-muted">
            <span>Danach: Telefon Nord GmbH</span>
            <span className="flex items-center gap-1.5">
              <Kbd>Esc</Kbd> zurück zur Liste
            </span>
          </div>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Dokumente ----------

const documents: { title: string; detail: string; figure: string; state: { tone: Tone; label: string }; icon: IconName }[] = [
  { title: "Stadtwerke Beispielstadt", detail: "Rechnung vom 14.11.2026", figure: "86,40 €", state: { tone: "neutral", label: "gelesen" }, icon: "receipt" },
  { title: "Telefon Nord GmbH", detail: "Rechnung vom 12.11.2026", figure: "39,99 €", state: { tone: "warn", label: "prüfen" }, icon: "receipt" },
  { title: "Versicherung Muster AG", detail: "Beitragsrechnung vom 08.11.2026", figure: "214,50 €", state: { tone: "danger", label: "doppelt?" }, icon: "receipt" },
  { title: "Beispielwerk GmbH", detail: "Gehaltsabrechnung Oktober 2026", figure: "3.480,00 €", state: { tone: "neutral", label: "gelesen" }, icon: "document" },
  { title: "Autohaus Lindenweg", detail: "Rechnung vom 03.11.2026", figure: "489,12 €", state: { tone: "ok", label: "bestätigt" }, icon: "receipt" },
  { title: "Musterbank", detail: "Kontoauszug Oktober 2026", figure: "", state: { tone: "neutral", label: "gelesen" }, icon: "card" },
  { title: "Praxis am Markt", detail: "Quittung vom 29.10.2026", figure: "25,00 €", state: { tone: "ok", label: "bestätigt" }, icon: "receipt" },
  { title: "Bücherei Ahorn", detail: "Rechnung vom 21.10.2026", figure: "18,90 €", state: { tone: "neutral", label: "gelesen" }, icon: "receipt" },
  { title: "Tonspur", detail: "Rechnung vom 15.10.2026, aus dem Postfach", figure: "10,99 €", state: { tone: "neutral", label: "gelesen" }, icon: "mail" },
];

/** `states`: "hidden" lists the documents without their state, "choosing" shows the filter of the states open, where the owner hides them. */
export function Dokumente({ back, search = false, rail = false, resize = false, values = true, states = "shown" }: { back?: string; search?: boolean; rail?: boolean; resize?: boolean; values?: boolean; states?: "shown" | "hidden" | "choosing" }) {
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("");
  const [state, setState] = useState(states === "hidden" ? "none" : "");
  return (
    <Frame
      view="dokumente"
      path={["Dokumente"]}
      back={back}
      rail={rail}
      actions={
        <>
          <Toggle glyph="values" label={values ? "Angaben ausblenden" : "Angaben einblenden"} on={values} />
          <More entries={[{ label: "Bestellungen", icon: "receipt", onSelect: nothing }, { label: "Als Tabelle exportieren …", icon: "upload", onSelect: nothing }, { label: "Namen einstellen", icon: "gear", onSelect: nothing }]} />
          <Menu
            align="end"
            entries={[
              { label: "Dateien auswählen …", icon: "upload", onSelect: nothing },
              { label: "Ordner einmal einlesen …", icon: "folder", onSelect: nothing },
              { label: "Kontoauszug als CSV …", icon: "card", onSelect: nothing },
              { label: "Jetzt im Postfach nachsehen", icon: "mail", onSelect: nothing },
            ]}
            trigger={(props) => (
              <button type="button" {...props} className="inline-flex h-(--control-height) shrink-0 items-center gap-1.5 rounded-control bg-accent px-3 font-medium text-on-accent transition-quick hover:bg-accent-hover">
                <Icon name="plus" size={16} />
                Importieren
                <Icon name="chevronDown" size={14} />
              </button>
            )}
          />
        </>
      }
    >
      <main className={cn("grid min-h-0 flex-1 grid-cols-[340px_minmax(0,1fr)] gap-5 p-5", !values ? "min-[1300px]:grid-cols-[400px_minmax(0,1fr)]" : resize ? "min-[1300px]:grid-cols-[470px_minmax(0,1fr)_340px]" : "min-[1300px]:grid-cols-[400px_minmax(0,1fr)_360px]")}>
        {search && <SearchOpen />}
        <section className={cn(panel, "relative flex min-h-0 flex-col")}>
          {resize && <Divider />}
          <div className="flex flex-col gap-2 border-b border-line p-3">
            <TextField label="Suchen" icon="search" placeholder="Absender, Nummer, Betrag" value={query} onChange={setQuery} />
            <div className="grid grid-cols-2 gap-2">
              <SelectField label="Art" value={kind} onChange={setKind} options={[{ value: "", label: "Alle Arten" }, { value: "r", label: "Rechnungen" }]} />
              <div className="relative">
                <SelectField label="Zustand" value={state} onChange={setState} options={[{ value: "", label: "Jeder Zustand" }, { value: "p", label: "prüfen" }, { value: "none", label: "Ausgeblendet" }]} />
                {states === "choosing" && <StateChoice />}
              </div>
            </div>
            <p className="pt-0.5 text-small text-fg-muted">1.284 Dokumente, 4 warten auf dich</p>
          </div>
          <List label="Dokumente" className="min-h-0 flex-1 overflow-y-auto">
            {documents.map((row, at) => (
              <ListRow key={row.title} title={row.title} detail={row.detail} figure={row.figure} state={states === "hidden" ? undefined : row.state} leading={<Icon name={row.icon} size={16} />} selected={at === 0} onSelect={() => undefined} />
            ))}
          </List>
        </section>
        <div className="flex min-h-0 flex-col gap-5 overflow-y-auto min-[1300px]:contents">
        <section aria-label="Seite" className={cn(panel, "relative flex h-[360px] min-h-0 shrink-0 items-center justify-center overflow-hidden p-5 pb-16 min-[1300px]:h-auto")}>
          <div className="flex size-full items-center justify-center [container-type:size]">
            <Page className="h-[min(100cqh,141.4cqw)]" />
          </div>
          <div className="absolute inset-x-0 bottom-3.5 flex justify-center">
            <PageTools />
          </div>
        </section>
        <section className={cn(panel, "flex shrink-0 flex-col min-[1300px]:min-h-0 min-[1300px]:shrink", !values && "hidden")}>
          <div className="flex flex-col gap-0.5 px-5 pt-5 pb-3">
            <div className="flex items-start justify-between gap-2">
              <h2 className="text-heading font-semibold">2026-11-14 Stadtwerke Beispielstadt Rechnung</h2>
              <More
                small
                entries={[
                  { label: "Umbenennen …", icon: "document", onSelect: nothing },
                  { label: "Datei im Ordner zeigen", icon: "folder", onSelect: nothing },
                  { label: "Löschen …", icon: "trash", danger: true, onSelect: nothing },
                ]}
              />
            </div>
            <p className="text-small text-fg-muted">Rechnung, 1 Seite, kam am 14.11. aus dem Ordner Eingang</p>
          </div>
          <div className="flex flex-col divide-y divide-line border-t border-line px-5 py-1">
            <Value label="Absender" value="Stadtwerke Beispielstadt" source="bekannt" />
            <Value label="Rechnungsdatum" value="14.11.2026" source="geprüft" />
            <Value label="Nummer" value="SW-2026-0413" source="geprüft" />
            <Value label="Betrag" value="86,40 €" source="geprüft" />
            <Value label="Kategorie" value="Wohnen und Energie" source="vom Absender" tone="neutral" />
          </div>
          <div className="flex flex-col gap-2 border-t border-line px-5 py-4">
            <p className="text-small font-medium text-fg-muted">Bezahlt</p>
            <button type="button" className="flex items-center gap-3 rounded-row bg-canvas px-3 py-2.5 text-left">
              <Icon name="card" size={16} className="text-fg-muted" />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="font-medium">Girokonto, 18.11.2026</span>
                <span className="truncate text-small text-fg-muted">Lastschrift SW-2026-0413</span>
              </span>
              <span className="font-medium tabular-nums">86,40 €</span>
              <Icon name="chevronRight" size={14} className="text-fg-muted" />
            </button>
          </div>
          <div className="mt-auto flex items-center gap-2 border-t border-line px-5 py-3">
            <span className="flex-1 text-small text-fg-muted">Abgelegt ohne Rückfrage</span>
            <Button size="small" kind="plain">
              Warum?
            </Button>
            <Button size="small" kind="plain">
              Rückgängig
            </Button>
          </div>
        </section>
        </div>
      </main>
    </Frame>
  );
}

// ---------- Konten ----------

const accounts: { title: string; detail: string; figure: string; state: { tone: Tone; label: string } }[] = [
  { title: "Girokonto", detail: "Die Musterbank nennt 4.812,07 € am 31.10.2026, der September fehlt", figure: "4.812,07 €", state: { tone: "warn", label: "prüfen" } },
  { title: "Tagesgeld", detail: "Die Musterbank nennt 7.500,00 € am 31.10.2026", figure: "7.500,00 €", state: { tone: "ok", label: "stimmt" } },
  { title: "Kreditkarte", detail: "Die Kartenbank Beispiel nennt 318,24 € im Soll am 25.10.2026", figure: "−318,24 €", state: { tone: "ok", label: "stimmt" } },
  { title: "PayPal", detail: "PayPal nennt 610,50 € am 31.10.2026", figure: "610,50 €", state: { tone: "ok", label: "stimmt" } },
  { title: "Bargeld", detail: "Ohne Kontoauszug, 6 Buchungen von Hand", figure: "ohne Stand", state: { tone: "neutral", label: "ohne Bank" } },
];

function Konten() {
  return (
    <Frame
      view="konten"
      path={["Konten"]}
      actions={
        <>
          <Button>Alle Umsätze</Button>
          <Button kind="primary" icon="plus">
            Kontoauszug importieren
          </Button>
        </>
      }
    >
      <main className="min-h-0 flex-1 overflow-y-auto px-10 py-9">
        <div className="flex max-w-[880px] flex-col gap-6">
          <header className="flex items-end justify-between gap-6">
            <div className="flex flex-col">
              <span className="text-small text-fg-muted">Auf allen Konten</span>
              <span className="text-[40px] leading-[44px] font-semibold tracking-[-0.03em] tabular-nums">12.604,33 €</span>
            </div>
            <span className="pb-1.5 text-small text-fg-muted">4 von 5 Konten stimmen mit der Bank überein</span>
          </header>
          <div className={panel}>
            <List label="Konten">
              {accounts.map((row) => (
                <ListRow key={row.title} title={row.title} detail={row.detail} figure={row.figure} state={row.state} leading={<Icon name="card" size={16} />} onSelect={() => undefined} />
              ))}
            </List>
          </div>
          <div className="grid grid-cols-2 gap-5">
            <button type="button" className={cn(panel, "flex items-center gap-3 px-4 py-3.5 text-left")}>
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="font-medium">Wiederkehrend</span>
                <span className="truncate text-small text-fg-muted">14 Zahlungen, 1.186,40 € im Monat</span>
              </span>
              <Icon name="chevronRight" size={16} className="text-fg-muted" />
            </button>
            <button type="button" className={cn(panel, "flex items-center gap-3 px-4 py-3.5 text-left")}>
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="font-medium">Umbuchungen</span>
                <span className="truncate text-small text-fg-muted">38 zwischen deinen Konten in 2026, in keiner Summe gezählt</span>
              </span>
              <Icon name="chevronRight" size={16} className="text-fg-muted" />
            </button>
          </div>
        </div>
      </main>
    </Frame>
  );
}

// ---------- Umsätze ----------

type Booking = { day: string; who: string; purpose: string; amount: string; incoming?: boolean; proof?: boolean; category: string };

const bookings: { month: string; sum: string; rows: Booking[] }[] = [
  {
    month: "November 2026",
    sum: "+3.480,00 €  und  1.204,71 €",
    rows: [
      { day: "25.11.", who: "Beispielwerk GmbH", purpose: "Gehalt November", amount: "+3.480,00 €", incoming: true, proof: true, category: "Gehalt" },
      { day: "18.11.", who: "Stadtwerke Beispielstadt", purpose: "Lastschrift SW-2026-0413", amount: "86,40 €", proof: true, category: "Wohnen und Energie" },
      { day: "17.11.", who: "Tagesgeld", purpose: "Umbuchung auf dein Tagesgeld", amount: "500,00 €", category: "Umbuchung" },
      { day: "15.11.", who: "Tonspur", purpose: "Abo November", amount: "10,99 €", proof: true, category: "Unterhaltung" },
      { day: "12.11.", who: "Telefon Nord GmbH", purpose: "Rechnung TN 55120", amount: "39,99 €", proof: true, category: "Telefon und Internet" },
      { day: "09.11.", who: "Amazon", purpose: "Bestellung vom 09.11.", amount: "64,90 €", category: "Ohne Kategorie" },
      { day: "05.11.", who: "Markthalle Süd", purpose: "Kartenzahlung", amount: "47,23 €", category: "Lebensmittel" },
      { day: "03.11.", who: "Autohaus Lindenweg", purpose: "Rechnung AL-7781", amount: "489,12 €", proof: true, category: "Auto" },
      { day: "01.11.", who: "Hausverwaltung Birke", purpose: "Miete November", amount: "780,00 €", category: "Wohnen und Energie" },
    ],
  },
  {
    month: "Oktober 2026",
    sum: "+3.480,00 €  und  2.611,05 €",
    rows: [
      { day: "29.10.", who: "Praxis am Markt", purpose: "Kartenzahlung", amount: "25,00 €", proof: true, category: "Gesundheit" },
      { day: "27.10.", who: "Beispielwerk GmbH", purpose: "Gehalt Oktober", amount: "+3.480,00 €", incoming: true, proof: true, category: "Gehalt" },
      { day: "21.10.", who: "Bücherei Ahorn", purpose: "Rechnung BA 3310", amount: "18,90 €", proof: true, category: "Bildung" },
    ],
  },
];

export function Umsaetze({ loading = false, detail = true }: { loading?: boolean; detail?: boolean }) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("");
  const [range, setRange] = useState("2026");
  return (
    <Frame view="umsaetze" path={["Konten", "Girokonto"]} actions={<><Toggle glyph="values" label={detail ? "Detail ausblenden" : "Detail einblenden"} on={detail} /><More entries={[{ label: "Buchung von Hand …", icon: "plus", onSelect: nothing }, { label: "Kontoauszug importieren …", icon: "upload", onSelect: nothing }, { label: "Als Tabelle exportieren …", icon: "document", onSelect: nothing }, { label: "Konto umbenennen …", icon: "card", onSelect: nothing }]} /></>}>
      <main className={cn("grid min-h-0 flex-1 gap-5 p-5", detail ? "grid-cols-[minmax(0,1fr)_280px] min-[1300px]:grid-cols-[minmax(0,1fr)_400px]" : "grid-cols-1")}>
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex flex-col gap-2 border-b border-line p-3">
            <div className="grid grid-cols-[minmax(0,1fr)_110px] items-end gap-2 min-[1300px]:grid-cols-[minmax(0,1fr)_200px_140px]">
              <TextField label="Suchen" icon="search" placeholder="Wer, Zweck, Betrag" value={query} onChange={setQuery} className="col-span-2 min-[1300px]:col-span-1" />
              <SelectField label="Kategorie" value={category} onChange={setCategory} options={[{ value: "", label: "Alle Kategorien" }]} />
              <SelectField label="Zeitraum" value={range} onChange={setRange} options={[{ value: "2026", label: "2026" }]} />
            </div>
            <p className="truncate pt-0.5 text-small text-fg-muted tabular-nums">{loading ? "\u00a0" : "214 Buchungen, Einnahmen 38.412,60 €, Ausgaben 29.870,14 €, Umbuchungen nicht mitgezählt"}</p>
          </div>
          <div role="listbox" aria-label="Umsätze" aria-busy={loading} className="min-h-0 flex-1 overflow-y-auto px-1 pb-1">
            {loading && <SkeletonRows />}
            {(loading ? [] : bookings).map((group) => (
              <div key={group.month}>
                <div className="sticky top-0 flex items-baseline justify-between bg-surface px-2.5 pt-3 pb-1.5 text-small text-fg-muted">
                  <span className="font-medium text-fg">{group.month}</span>
                  <span className="tabular-nums">{group.sum}</span>
                </div>
                {group.rows.map((row, at) => {
                  const selected = group.month === "November 2026" && at === 1;
                  return (
                    <div key={`${row.day}-${row.who}`} role="option" aria-selected={selected} className={cn("relative flex h-10 items-center gap-3 rounded-row px-2.5", selected ? "bg-selected" : "hover:bg-hover")}>
                      {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
                      <span className="w-12 shrink-0 text-small text-fg-muted tabular-nums">{row.day}</span>
                      <span className="min-w-0 flex-1 truncate font-medium min-[1300px]:w-44 min-[1300px]:flex-none">{row.who}</span>
                      <span className="hidden min-w-0 flex-1 truncate text-small text-fg-muted min-[1300px]:block">{row.purpose}</span>
                      <span className="hidden w-36 shrink-0 truncate text-small text-fg-muted min-[1300px]:block">{row.category}</span>
                      <span className="flex w-5 shrink-0 justify-center text-fg-muted">{row.proof ? <Icon name="document" size={14} /> : null}</span>
                      <span className="w-24 shrink-0 text-right font-medium tabular-nums min-[1300px]:w-28">{row.amount}</span>
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        </section>
        {loading && <section className={cn(panel, "flex min-h-0 items-center justify-center p-8 text-center text-small text-fg-muted")}>Wähle eine Buchung, um ihren Beleg zu sehen.</section>}
        <section className={cn(panel, "flex min-h-0 flex-col overflow-y-auto", (loading || !detail) && "hidden")}>
          <div className="flex flex-col gap-1 p-5">
            <span className="text-small text-fg-muted">18.11.2026, Girokonto</span>
            <h2 className="text-heading font-semibold">Stadtwerke Beispielstadt</h2>
            <span className="text-title font-semibold tracking-[-0.025em] tabular-nums">86,40 €</span>
            <span className="text-small text-fg-muted">Lastschrift SW-2026-0413 Abschlag November</span>
          </div>
          <div className="border-t border-line px-5 py-4">
            <SelectField label="Kategorie, vom Absender" value="w" onChange={() => undefined} options={[{ value: "w", label: "Wohnen und Energie" }]} />
          </div>
          <div className="flex flex-col gap-2 border-t border-line px-5 py-4">
            <p className="text-small font-medium text-fg-muted">Beleg</p>
            <button type="button" className="flex items-center gap-3 rounded-row bg-canvas p-2.5 text-left">
              <Page className="w-9 shrink-0 !p-[3px] !shadow-panel" />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="truncate font-medium">2026-11-14 Stadtwerke Beispielstadt Rechnung</span>
                <span className="text-small text-fg-muted">Rechnung über 86,40 €, Nummer im Zweck gefunden</span>
              </span>
              <Icon name="chevronRight" size={14} className="text-fg-muted" />
            </button>
          </div>
          <div className="flex flex-col gap-2 border-t border-line px-5 py-4">
            <p className="text-small font-medium text-fg-muted">Stadtwerke Beispielstadt in 2026</p>
            <div className="flex items-end gap-1.5" style={{ height: 44 }}>
              {[80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 88, 0].map((height, at) => (
                <span key={at} className={cn("flex-1 rounded-t-[3px]", at === 10 ? "bg-accent" : "bg-line-strong")} style={{ height: `${height}%` }} />
              ))}
            </div>
            <p className="text-small text-fg-muted tabular-nums">11 Buchungen, 950,40 €. Seit November 6,40 € mehr im Monat.</p>
          </div>
          <div className="mt-auto flex items-center gap-2 border-t border-line px-5 py-3">
            <span className="flex-1 text-small text-fg-muted">Kontoauszug November, Seite 2</span>
            <Button size="small" kind="plain">
              Stelle zeigen
            </Button>
          </div>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Auswertungen ----------

const byCategory: [string, string, number][] = [
  ["Wohnen und Energie", "10.412,80 €", 100],
  ["Lebensmittel", "5.230,14 €", 50],
  ["Auto", "3.871,40 €", 37],
  ["Versicherungen", "2.640,00 €", 25],
  ["Unterhaltung", "1.904,36 €", 18],
  ["Telefon und Internet", "879,78 €", 8],
  ["Ohne Kategorie", "4.931,66 €", 47],
];

const recurring: [string, string, string, Tone | null, string][] = [
  ["Hausverwaltung Birke", "monatlich, nächste am 01.12.", "780,00 €", null, ""],
  ["Stadtwerke Beispielstadt", "monatlich, seit November 6,40 € mehr", "86,40 €", "warn", "gestiegen"],
  ["Versicherung Muster AG", "jährlich, nächste im November 2027", "214,50 €", null, ""],
  ["Tonspur", "monatlich, nächste am 15.12.", "10,99 €", "warn", "doppelt?"],
  ["Klangwelt", "monatlich, nächste am 03.12.", "10,99 €", "warn", "doppelt?"],
];

function Auswertungen() {
  return (
    <Frame
      view="auswertungen"
      path={["Auswertungen"]}
      actions={
        <>
          <Segments options={["2024", "2025", "2026"]} value="2026" />
          <Button icon="more" label="Weitere Aktionen" kind="plain" />
        </>
      }
    >
      <main className="min-h-0 flex-1 overflow-y-auto px-10 py-9">
        <div className="flex max-w-[1120px] flex-col gap-8">
          <section className="grid grid-cols-[300px_minmax(0,1fr)] gap-12">
            <div className="flex flex-col">
              <Figure label="Einnahmen" value="38.412,60 €" note="2025 bis November: 36.272,60 €" />
              <Figure label="Ausgaben" value="29.870,14 €" note="2025 bis November: 31.182,62 €" />
              <Figure label="Bleibt" value="8.542,46 €" note="22 % der Einnahmen" />
            </div>
            <div className="flex flex-col gap-2">
              <Heading aside="Einnahmen hell, Ausgaben dunkel">Monat für Monat</Heading>
              <div className="flex flex-1 flex-col justify-end border-b border-line">
                <MonthBars height={200} chosen={6} />
              </div>
              <div className="flex text-small text-fg-muted">
                {["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"].map((month, at) => (
                  <span key={month} className={cn("flex-1 text-center", at === 6 && "font-medium text-fg")}>
                    {month}
                  </span>
                ))}
              </div>
              <p className="text-small text-fg-muted">Juli: Ausgaben 3.640,10 €, davon 1.180,00 € Autohaus Lindenweg</p>
            </div>
          </section>
          <section className="grid grid-cols-2 gap-5">
            <div className={cn(panel, "flex flex-col gap-3 p-5")}>
              <Heading aside="Ausgaben">Nach Kategorie</Heading>
              <div className="flex flex-col">
                {byCategory.map(([name, sum, share]) => (
                  <button key={name} type="button" className="grid grid-cols-[170px_minmax(0,1fr)_96px] items-center gap-3 py-1.5 text-left">
                    <span className={cn("truncate", name === "Ohne Kategorie" && "text-fg-muted")}>{name}</span>
                    <span className="h-2 rounded-full bg-canvas">
                      <span className={cn("block h-full rounded-full", name === "Ohne Kategorie" ? "bg-line-strong" : "bg-accent")} style={{ width: `${share}%` }} />
                    </span>
                    <span className="text-right font-medium tabular-nums">{sum}</span>
                  </button>
                ))}
              </div>
            </div>
            <div className={cn(panel, "flex flex-col gap-3 p-5")}>
              <Heading aside="14 Zahlungen, 1.186,40 € im Monat">Wiederkehrend</Heading>
              <div className="flex flex-col divide-y divide-line">
                {recurring.map(([name, rhythm, amount, tone, label]) => (
                  <div key={name} className="flex items-center gap-3 py-2">
                    <span className="flex min-w-0 flex-1 flex-col">
                      <span className="truncate font-medium">{name}</span>
                      <span className="truncate text-small text-fg-muted">{rhythm}</span>
                    </span>
                    <span className="font-medium tabular-nums">{amount}</span>
                    {tone ? <Capsule tone={tone}>{label}</Capsule> : <span className="w-24" />}
                  </div>
                ))}
              </div>
            </div>
          </section>
        </div>
      </main>
    </Frame>
  );
}

// ---------- Steuern ----------

const tax: { form: string; sum: string; lines: [string, string, string, boolean][] }[] = [
  {
    form: "Anlage N, Werbungskosten",
    sum: "2.318,40 €",
    lines: [
      ["Fortbildungskosten", "Dein Drittel des Studiums, 3 Belege", "1.240,00 €", true],
      ["Arbeitsmittel", "4 Belege", "418,40 €", false],
      ["Homeoffice-Pauschale", "110 Tage, Wert für 2026 geprüft am 04.11.", "660,00 €", false],
    ],
  },
  {
    form: "Haushaltsnahe Aufwendungen",
    sum: "372,00 €",
    lines: [["Handwerkerleistungen", "Arbeitsanteil aus 2 Rechnungen", "372,00 €", false]],
  },
  {
    form: "Sonderausgaben",
    sum: "180,00 €",
    lines: [["Spenden", "2 Bescheinigungen", "180,00 €", false]],
  },
];

function Steuern() {
  return (
    <Frame
      view="steuern"
      path={["Steuern"]}
      actions={
        <>
          <Segments options={["2024", "2025", "2026"]} value="2026" />
          <Button>Zusammenstellung sichern …</Button>
        </>
      }
    >
      <main className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_420px] gap-5 p-5">
        <section className={cn(panel, "flex min-h-0 flex-col overflow-y-auto")}>
          {tax.map((group) => (
            <div key={group.form} className="flex flex-col border-b border-line px-2 pt-4 pb-2 last:border-b-0">
              <div className="flex items-baseline justify-between px-3 pb-1.5">
                <h2 className="text-heading font-semibold">{group.form}</h2>
                <span className="font-semibold tabular-nums">{group.sum}</span>
              </div>
              {group.lines.map(([name, detail, sum, selected]) => (
                <div key={name} className={cn("relative flex min-h-(--row-height) items-center gap-3 rounded-row px-3", selected ? "bg-selected" : "hover:bg-hover")}>
                  {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="font-medium">{name}</span>
                    <span className="truncate text-small text-fg-muted">{detail}</span>
                  </span>
                  <span className="font-medium tabular-nums">{sum}</span>
                </div>
              ))}
            </div>
          ))}
          <p className="mt-auto border-t border-line px-5 py-3 text-small text-fg-muted">Eine Zusammenstellung für deine Erklärung in ELSTER. Du prüfst jede Angabe selbst, eine Erstattung wird nicht berechnet.</p>
        </section>
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex flex-col gap-1 p-5">
            <span className="text-small text-fg-muted">Anlage N, Werbungskosten</span>
            <h2 className="text-heading font-semibold">Fortbildungskosten</h2>
            <span className="text-title font-semibold tracking-[-0.025em] tabular-nums">1.240,00 €</span>
            <span className="text-small text-fg-muted">Von 3.720,00 € zählt dein Drittel. Zwei Drittel hat der Arbeitgeber erstattet.</span>
          </div>
          <div className="flex flex-col border-t border-line p-1">
            {[
              ["Hochschule Beispiel", "Semesterbeitrag, bezahlt am 02.03.2026", "1.860,00 €"],
              ["Hochschule Beispiel", "Semesterbeitrag, bezahlt am 01.09.2026", "1.860,00 €"],
            ].map(([who, detail, sum]) => (
              <div key={detail} className="flex min-h-(--row-height) items-center gap-3 rounded-row px-3 hover:bg-hover">
                <Page className="w-8 shrink-0 !p-[3px] !shadow-panel" />
                <span className="flex min-w-0 flex-1 flex-col">
                  <span className="font-medium">{who}</span>
                  <span className="truncate text-small text-fg-muted">{detail}</span>
                </span>
                <span className="font-medium tabular-nums">{sum}</span>
              </div>
            ))}
          </div>
          <div className="mt-auto flex items-center gap-2 border-t border-line px-5 py-3">
            <span className="flex-1 text-small text-fg-muted">Zählt im Jahr der Zahlung</span>
            <Button size="small" kind="plain" icon="plus">
              Beleg zuordnen
            </Button>
          </div>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Chat ----------

function Chat() {
  const [text, setText] = useState("");
  return (
    <Frame
      view="chat"
      path={["Chat", "Strom im Vergleich"]}
      actions={
        <Button kind="primary" icon="plus">
          Neues Gespräch
        </Button>
      }
    >
      <main className="grid min-h-0 flex-1 grid-cols-[280px_minmax(0,1fr)] gap-5 p-5">
        <section className={cn(panel, "min-h-0 overflow-y-auto")}>
          <List label="Gespräche">
            <ListRow title="Strom im Vergleich" detail="Heute" selected onSelect={() => undefined} />
            <ListRow title="Was kostet das Auto im Jahr?" detail="Gestern" onSelect={() => undefined} />
            <ListRow title="Gehalt seit 2023" detail="21.11.2026" onSelect={() => undefined} />
            <ListRow title="Spenden für die Erklärung" detail="14.11.2026" onSelect={() => undefined} />
          </List>
        </section>
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto px-8 py-7">
            <p className="max-w-[60ch] self-end rounded-panel bg-canvas px-4 py-2.5">Wie viel habe ich 2026 für Strom bezahlt, und war das mehr als im Jahr davor?</p>
            <div className="flex max-w-[64ch] flex-col gap-4">
              <p>
                2026 hast du bis heute <span className="font-semibold tabular-nums">950,40 €</span> an die Stadtwerke Beispielstadt gezahlt, in 11 Buchungen. Im selben Zeitraum 2025 waren es <span className="font-semibold tabular-nums">880,00 €</span>, also 70,40 € weniger.
              </p>
              <p>Der Abschlag ist im November von 80,00 € auf 86,40 € gestiegen.</p>
              <div className="flex flex-col gap-1.5">
                <span className="text-small font-medium text-fg-muted">Daraus gelesen</span>
                <div className="flex flex-wrap gap-2">
                  {[
                    ["card", "11 Buchungen in 2026"],
                    ["card", "11 Buchungen in 2025"],
                    ["document", "Rechnung vom 14.11.2026"],
                  ].map(([icon, label]) => (
                    <button key={label} type="button" className="flex h-8 items-center gap-2 rounded-control bg-surface px-2.5 text-small font-medium shadow-[inset_0_0_0_1px_var(--line-strong)]">
                      <Icon name={icon as IconName} size={14} className="text-fg-muted" />
                      {label}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
          <form className="flex items-end gap-2 border-t border-line p-4" onSubmit={(event) => event.preventDefault()}>
            <TextField label="Frage an dein Archiv" placeholder="Zum Beispiel: Welche Versicherungen zahle ich?" value={text} onChange={setText} className="flex-1" />
            <Button kind="primary" type="submit">
              Fragen
            </Button>
          </form>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Einstellungen ----------

const pages: [string, IconName, string][] = [
  ["Ordner", "folder", "3 beobachtet"],
  ["Postfächer", "mail", "1 verbunden"],
  ["Namen", "document", "5 Vorlagen"],
  ["Kategorien", "receipt", "18 Kategorien"],
  ["Modell", "search", "erreichbar"],
  ["Archiv", "inbox", "Sicherung von heute"],
  ["Darstellung", "gear", "wie das System"],
  ["Über", "home", "Version 0.5.0"],
];

export function Einstellungen({ page, children }: { page: string; children?: ReactNode }) {
  return (
    <Frame view="einstellungen" path={["Einstellungen", page]}>
      <main className="grid min-h-0 flex-1 grid-cols-[280px_minmax(0,1fr)] gap-5 p-5">
        <section className={cn(panel, "min-h-0 overflow-y-auto")}>
          <List label="Einstellungen">
            {pages.map(([name, icon, detail]) => (
              <ListRow key={name} title={name} detail={detail} leading={<Icon name={icon} size={16} />} selected={name === page} onSelect={() => undefined} />
            ))}
          </List>
        </section>
        <section className="min-h-0 overflow-y-auto">
          <div className="flex max-w-[640px] flex-col gap-5">{children ?? <Postfaecher />}</div>
        </section>
      </main>
    </Frame>
  );
}

function Postfaecher() {
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  return (
    <>
            <div className={cn(panel, "flex flex-col")}>
              <div className="flex items-center gap-3 px-5 py-4">
                <span className="flex size-9 shrink-0 items-center justify-center rounded-control bg-canvas text-fg-muted">
                  <Icon name="mail" size={17} />
                </span>
                <span className="flex min-w-0 flex-1 flex-col">
                  <span className="truncate font-medium">beispiel@example.org</span>
                  <span className="truncate text-small text-fg-muted">Gmail, zuletzt gelesen heute um 09:12, 3 Rechnungen gefunden</span>
                </span>
                <Capsule tone="ok">verbunden</Capsule>
              </div>
              <div className="flex items-center gap-2 border-t border-line px-5 py-3">
                <Button size="small">Jetzt lesen</Button>
                <Button size="small" kind="plain">
                  Verbindung testen
                </Button>
                <span className="flex-1" />
                <Button size="small" kind="plain" icon="trash">
                  Entfernen …
                </Button>
              </div>
            </div>
            <form className={cn(panel, "flex flex-col gap-4 p-5")} onSubmit={(event) => event.preventDefault()}>
              <Heading>Gmail hinzufügen</Heading>
              <div className="grid grid-cols-2 gap-3">
                <TextField label="Adresse" type="email" value={address} onChange={setAddress} />
                <TextField label="App-Passwort" type="password" value={password} onChange={setPassword} hint="16 Zeichen aus deinem Google-Konto, nicht dein Passwort" />
              </div>
              <div>
                <Button icon="plus" type="submit">
                  Hinzufügen
                </Button>
              </div>
            </form>
            <p className="px-1 text-small text-fg-muted">Finanzarchiv liest deine Postfächer nur. Es verschiebt, markiert und löscht dort nichts, und das App-Passwort liegt im Schlüsselbund deines Systems.</p>
    </>
  );
}

/** Gray rows in the shape of the list while it loads, never a spinner in the middle of the window. */
export function SkeletonRows() {
  return (
    <div aria-hidden className="flex flex-col pt-3">
      <div className="px-2.5 pb-1.5">
        <span className="block h-3 w-28 rounded-full bg-hover" />
      </div>
      {[62, 80, 48, 70, 56, 76, 44, 66, 58, 72, 50].map((width, at) => (
        <div key={at} className="flex h-10 items-center gap-3 px-2.5" style={{ opacity: 1 - at * 0.07 }}>
          <span className="h-3 w-10 rounded-full bg-hover" />
          <span className="h-3 rounded-full bg-hover" style={{ width: `${width * 2}px` }} />
          <span className="flex-1" />
          <span className="h-3 w-16 rounded-full bg-hover" />
        </div>
      ))}
    </div>
  );
}

/** The quick search as it lies over a view, drawn still: the real piece is src/ui/QuickSearch.tsx. */
function SearchOpen() {
  const groups: [string, [IconName, string, string, boolean][]][] = [
    ["Dokumente", [["receipt", "2026-11-14 Stadtwerke Beispielstadt Rechnung", "86,40 €", true], ["receipt", "2026-10-14 Stadtwerke Beispielstadt Rechnung", "80,00 €", false]]],
    ["Absender", [["search", "Stadtwerke Beispielstadt", "11 Buchungen, 12 Dokumente", false]]],
    ["Buchungen", [["card", "18.11.2026 Stadtwerke Beispielstadt", "86,40 €", false]]],
    ["Aktionen", [["upload", "Importieren", "Dokumente", false], ["mail", "Postfächer einrichten", "Einstellungen", false]]],
  ];
  return (
    <div className="fixed inset-0 z-30 flex justify-center bg-backdrop pt-[12vh]">
      <div className="h-fit w-[min(92vw,40rem)] overflow-hidden rounded-dialog bg-raised text-fg shadow-raised">
        <div className="flex h-12 items-center gap-2.5 border-b border-line px-4">
          <Icon name="search" size={16} className="shrink-0 text-fg-muted" />
          <span className="min-w-0 flex-1 text-heading">stadtw</span>
          <Kbd>Esc</Kbd>
        </div>
        <div className="p-1.5">
          {groups.map(([group, rows]) => (
            <div key={group}>
              <p className="px-2.5 pt-2 pb-1 text-small font-medium text-fg-muted">{group}</p>
              {rows.map(([icon, label, detail, selected]) => (
                <div key={label} className={cn("flex h-10 items-center gap-3 rounded-row px-2.5", selected && "bg-selected")}>
                  <Icon name={icon} size={16} className="shrink-0 text-fg-muted" />
                  <span className="min-w-0 flex-1 truncate">{label}</span>
                  <span className="shrink-0 text-small text-fg-muted">{detail}</span>
                </div>
              ))}
            </div>
          ))}
        </div>
        <div className="flex items-center gap-3 border-t border-line px-4 py-2 text-small text-fg-muted">
          <span className="flex items-center gap-1">
            <Kbd>↑</Kbd>
            <Kbd>↓</Kbd> auswählen
          </span>
          <span className="flex items-center gap-1">
            <Kbd>↵</Kbd> öffnen
          </span>
        </div>
      </div>
    </div>
  );
}

export const hauptansichten = {
  start: () => <Start />,
  pruefen: () => <Pruefen />,
  dokumente: () => <Dokumente />,
  konten: () => <Konten />,
  umsaetze: () => <Umsaetze />,
  auswertungen: () => <Auswertungen />,
  steuern: () => <Steuern />,
  chat: () => <Chat />,
  einstellungen: () => <Einstellungen page="Postfächer" />,
  "start-eingang": () => <Start intake />,
} satisfies Record<string, () => ReactNode>;
