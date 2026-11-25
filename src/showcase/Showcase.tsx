// The showcase of #07: the pieces of the design system at real size in a window that looks like the app, a sidebar,
// a list, a preview, a form, a menu, a dialog and the quick search. The owner picked the look Graphit from four
// variants here (knowledge/decision-ui-libraries.md). Everything stands in the AppShell, whose top bar is the title
// bar of the window. The foot of the sidebar switches light and dark, a path of one, three or six pages, and the bar
// of Windows or macOS. Only in the dev
// build (App.tsx).
import { useCallback, useState } from "react";
import {
  AppShell,
  Button,
  Dialog,
  EmptyState,
  Icon,
  Kbd,
  List,
  ListRow,
  Menu,
  QuickSearch,
  SelectField,
  TextField,
  cn,
  isMac,
  useQuickSearchKey,
  windowSystem,
  type IconName,
  type SearchEntry,
  type WindowSystem,
} from "../ui";
import { categories, documents, formatCents, type SampleDocument } from "./sample";

type Theme = "system" | "light" | "dark";

// Paths of invented pages, one, three and six deep, so the owner sees the path in the bar fit and fold.
const trails: Record<"1" | "3" | "6", string[]> = {
  "1": ["Dokumente"],
  "3": ["Dokumente", "Rechnungen", "Stadtwerke Beispielstadt"],
  "6": ["Dokumente", "Rechnungen", "2026", "November", "Stadtwerke Beispielstadt", "Rechnung SW-2026-0413"],
};

const views: { id: string; label: string; icon: IconName; count?: number }[][] = [
  [
    { id: "start", label: "Start", icon: "home" },
    { id: "pruefen", label: "Prüfen", icon: "inbox", count: 3 },
  ],
  [
    { id: "dokumente", label: "Dokumente", icon: "document" },
    { id: "rechnungen", label: "Rechnungen", icon: "receipt" },
    { id: "zahlungen", label: "Zahlungen", icon: "card" },
    { id: "steuern", label: "Steuern", icon: "scale" },
  ],
];

export function Showcase() {
  const [theme, setTheme] = useState<Theme>("system");
  const [depth, setDepth] = useState<"1" | "3" | "6">("3");
  const [trail, setTrail] = useState<string[]>(trails["3"]);
  const [system, setSystem] = useState<WindowSystem>(windowSystem() === "mac" ? "mac" : "windows");
  const [view, setView] = useState("dokumente");
  const [filter, setFilter] = useState<"offen" | "erledigt">("offen");
  const [chosenId, setChosenId] = useState("d1");
  const [query, setQuery] = useState("");
  const [deleting, setDeleting] = useState(false);
  const [searching, setSearching] = useState(false);
  const [saving, setSaving] = useState(false);

  const chosen = documents.find((d) => d.id === chosenId) ?? documents[0];
  const openSearch = useCallback(() => setSearching(true), []);
  useQuickSearchKey(openSearch);

  const shown = filter === "erledigt" ? [] : documents.filter((d) => d.sender.toLowerCase().includes(query.toLowerCase()));
  const entries: SearchEntry[] = [
    ...views.flat().map((v) => ({ id: v.id, group: "Ansichten", label: v.label, icon: v.icon, onSelect: () => setView(v.id) })),
    ...documents.map((d) => ({ id: d.id, group: "Dokumente", label: d.sender, detail: `${d.kind} vom ${d.date}`, icon: "document" as const, onSelect: () => setChosenId(d.id) })),
  ];

  return (
    <div data-theme={theme === "system" ? undefined : theme} className="relative h-full">
      {system === "mac" && windowSystem() !== "mac" && <DrawnTrafficLights />}
      <AppShell
        system={system}
        path={trail.map((label, index) => ({ label, onSelect: () => setTrail(trail.slice(0, index + 1)) }))}
        appName={
          <>
            <span data-tauri-drag-region className="flex size-6 items-center justify-center rounded-[6px] bg-accent text-on-accent">
              <Icon name="folder" size={14} />
            </span>
            <span data-tauri-drag-region className="font-semibold">
              Finanzarchiv
            </span>
          </>
        }
        actions={
          <>
            <Menu
              align="end"
              entries={[
                { label: "Dateien auswählen", icon: "upload", onSelect: () => undefined },
                { label: "Ordner beobachten", icon: "folder", onSelect: () => undefined },
                { label: "Aus dem Postfach", icon: "mail", onSelect: () => undefined },
              ]}
              trigger={({ ref, ...props }) => (
                <button
                  ref={ref}
                  type="button"
                  {...props}
                  className="inline-flex h-(--control-height) items-center gap-1.5 rounded-control bg-surface px-3 font-medium text-fg shadow-panel transition-quick hover:bg-hover active:scale-[0.98]"
                >
                  <Icon name="plus" size={15} />
                  Importieren
                  <Icon name="chevronDown" size={14} className="text-fg-muted" />
                </button>
              )}
            />
            <Button kind="primary" icon="inbox">
              Prüfen starten
            </Button>
          </>
        }
        sidebar={
          <div className="flex min-h-0 flex-1 flex-col gap-4 px-3 pb-3">
            <button
              type="button"
              onClick={openSearch}
              className="flex h-(--control-height) items-center gap-2 rounded-control bg-field px-2.5 text-fg-muted shadow-[inset_0_0_0_1px_var(--line)] transition-quick hover:text-fg"
            >
              <Icon name="search" size={15} />
              <span className="flex-1 text-left">Suchen</span>
              <Kbd>{isMac ? "⌘" : "Strg"}</Kbd>
              <Kbd>K</Kbd>
            </button>
            <nav aria-label="Ansichten" className="flex flex-1 flex-col gap-4">
              {views.map((group, i) => (
                <ul key={i} className="flex flex-col gap-px">
                  {group.map((v) => (
                    <li key={v.id}>
                      <SidebarRow {...v} selected={view === v.id} onSelect={() => setView(v.id)} />
                    </li>
                  ))}
                </ul>
              ))}
            </nav>
            <SidebarRow id="einstellungen" label="Einstellungen" icon="gear" selected={view === "einstellungen"} onSelect={() => setView("einstellungen")} />
            <div className="flex flex-col gap-2 border-t border-line pt-3 text-small text-fg-muted">
              <span>Vorschau</span>
              <Segments value={theme} onChange={setTheme} options={[{ value: "system", label: "System" }, { value: "light", label: "Hell" }, { value: "dark", label: "Dunkel" }]} />
              <Segments
                value={depth}
                onChange={(value) => {
                  setDepth(value);
                  setTrail(trails[value]);
                }}
                options={[{ value: "1", label: "Pfad 1" }, { value: "3", label: "3" }, { value: "6", label: "6" }]}
              />
              <Segments value={system} onChange={setSystem} options={[{ value: "windows", label: "Windows" }, { value: "mac", label: "macOS" }]} />
            </div>
          </div>
        }
      >
          <p className="px-6 pt-4 pb-4 text-fg-muted">Alles, was im Archiv liegt. Wähle ein Dokument, um seine Angaben zu prüfen.</p>
          <div className="grid min-h-0 flex-1 grid-cols-[minmax(320px,400px)_minmax(0,1fr)_minmax(280px,340px)] gap-4 px-6 pb-6">
            <section aria-label="Liste" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
              <div className="flex flex-col gap-3 border-b border-line p-3">
                <TextField label="Suchen" value={query} onChange={setQuery} icon="search" clearable placeholder="Absender" />
                <div role="radiogroup" aria-label="Filter" className="inline-flex self-start rounded-control bg-canvas p-0.5 shadow-[inset_0_0_0_1px_var(--line)]">
                  {(["offen", "erledigt"] as const).map((f) => (
                    <button
                      key={f}
                      type="button"
                      role="radio"
                      aria-checked={filter === f}
                      onClick={() => setFilter(f)}
                      className={cn(
                        "h-6 rounded-[calc(var(--r-control)-2px)] px-3 text-small font-medium transition-quick",
                        filter === f ? "bg-surface text-fg shadow-panel" : "text-fg-muted hover:text-fg",
                      )}
                    >
                      {f === "offen" ? "Offen" : "Erledigt"}
                    </button>
                  ))}
                </div>
              </div>
              <div className="min-h-0 flex-1 overflow-y-auto">
                {shown.length > 0 ? (
                  <List label="Dokumente">
                    {shown.map((d) => (
                      <ListRow
                        key={d.id}
                        title={d.sender}
                        detail={`${d.kind} vom ${d.date}`}
                        leading={<Icon name={d.kind === "Kontoauszug" ? "card" : "receipt"} size={16} />}
                        figure={d.cents > 0 ? formatCents(d.cents) : undefined}
                        state={d.state}
                        selected={d.id === chosenId}
                        onSelect={() => setChosenId(d.id)}
                      />
                    ))}
                  </List>
                ) : filter === "erledigt" ? (
                  <EmptyState
                    icon="check"
                    title="Noch nichts erledigt"
                    hint="Bestätigte Dokumente erscheinen hier, sobald du das erste geprüft hast."
                    actions={<Button onClick={() => setFilter("offen")}>Zu den offenen</Button>}
                  />
                ) : (
                  <EmptyState
                    icon="search"
                    title="Kein Absender passt"
                    hint={`Kein Dokument stammt von „${query}“.`}
                    actions={<Button onClick={() => setQuery("")}>Suche leeren</Button>}
                  />
                )}
              </div>
            </section>

            <section aria-label="Vorschau" className="flex min-h-0 items-center justify-center rounded-panel bg-surface p-4 shadow-panel">
              <div className="flex size-full items-center justify-center [container-type:size]">
                {chosen && <PaperPreview document={chosen} />}
              </div>
            </section>

            <section aria-label="Angaben" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
              {chosen && (
                <DocumentForm
                  key={chosen.id}
                  document={chosen}
                  saving={saving}
                  onConfirm={() => {
                    setSaving(true);
                    window.setTimeout(() => setSaving(false), 900);
                  }}
                  onDelete={() => setDeleting(true)}
                />
              )}
            </section>
          </div>
      </AppShell>

        <Dialog
          open={deleting}
          onClose={() => setDeleting(false)}
          size="narrow"
          title="Dokument löschen?"
          description={`Die Rechnung von ${chosen?.sender ?? ""} kommt in den Papierkorb des Archivs. Dort bleibt sie, bis du ihn leerst.`}
          actions={
            <>
              <Button onClick={() => setDeleting(false)}>Abbrechen</Button>
              <Button kind="danger" icon="trash" onClick={() => setDeleting(false)}>
                Löschen
              </Button>
            </>
          }
        />
        <QuickSearch open={searching} onClose={() => setSearching(false)} entries={entries} />
    </div>
  );
}

/** The traffic lights macOS draws over the bar, drawn here only to preview the bar of the Mac on another system. */
function DrawnTrafficLights() {
  return (
    <div aria-hidden className="pointer-events-none absolute top-[calc(var(--bar-height)/2-6px)] left-5 z-10 flex gap-2">
      {["#ff5f57", "#febc2e", "#28c840"].map((color) => (
        <span key={color} className="size-3 rounded-full" style={{ backgroundColor: color }} />
      ))}
    </div>
  );
}

function SidebarRow({ label, icon, count, selected, onSelect }: { id: string; label: string; icon: IconName; count?: number; selected: boolean; onSelect: () => void }) {
  return (
    <button
      type="button"
      aria-current={selected ? "page" : undefined}
      onClick={onSelect}
      className={cn(
        "relative flex h-8 w-full items-center gap-2.5 rounded-row px-2 text-left transition-quick",
        selected ? "bg-selected font-medium text-fg" : "text-fg hover:bg-hover",
      )}
    >
      {selected && <span aria-hidden className="absolute top-1/2 left-0 h-4 w-[3px] -translate-y-1/2 rounded-full bg-mark" />}
      <Icon name={icon} size={16} className={selected ? "text-fg-accent" : "text-fg-muted"} />
      <span className="flex-1">{label}</span>
      {count !== undefined && <span className="rounded-full bg-accent px-1.5 text-small font-semibold text-on-accent tabular-nums">{count}</span>}
    </button>
  );
}

function DocumentForm({ document, saving, onConfirm, onDelete }: { document: SampleDocument; saving: boolean; onConfirm: () => void; onDelete: () => void }) {
  const [sender, setSender] = useState(document.sender);
  const [number, setNumber] = useState(document.number);
  const [date, setDate] = useState(document.date);
  const [amount, setAmount] = useState(document.cents > 0 ? formatCents(document.cents).replace(" €", "") : "");
  const [category, setCategory] = useState(document.category);
  const [note, setNote] = useState("");
  return (
    <form
      className="flex min-h-0 flex-1 flex-col"
      onSubmit={(event) => {
        event.preventDefault();
        onConfirm();
      }}
    >
      <div className="flex items-center justify-between px-4 pt-4 pb-2">
        <h2 className="text-heading font-semibold">Angaben</h2>
        <Menu
          align="end"
          entries={[
            { label: "Datei im Ordner zeigen", icon: "folder", onSelect: () => undefined },
            { label: "Löschen", icon: "trash", danger: true, onSelect: onDelete },
          ]}
          trigger={({ ref, ...props }) => (
            <button ref={ref} type="button" aria-label="Weitere Aktionen" {...props} className="flex size-7 items-center justify-center rounded-control text-fg-muted transition-quick hover:bg-hover hover:text-fg">
              <Icon name="more" size={16} />
            </button>
          )}
        />
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-4 pb-4">
        <TextField label="Absender" value={sender} onChange={setSender} />
        <div className="grid grid-cols-2 gap-3">
          <TextField label="Rechnungsdatum" value={date} onChange={setDate} />
          <TextField label="Nummer" value={number} onChange={setNumber} />
        </div>
        <TextField label="Betrag" value={amount} onChange={setAmount} suffix="€" inputMode="decimal" hint="Brutto, wie auf der Rechnung" error={amount.trim() === "" ? "Ohne Betrag kann die Rechnung nicht bestätigt werden" : undefined} />
        <SelectField label="Kategorie" value={category} onChange={setCategory} options={categories} />
        <TextField label="Notiz" value={note} onChange={setNote} placeholder="Optional" />
      </div>
      <div className="flex items-center gap-2 border-t border-line px-4 py-3">
        <Button kind="danger" icon="trash" label="Löschen" onClick={onDelete} />
        <span className="flex-1" />
        <Button kind="plain">Später</Button>
        <Button kind="primary" type="submit" busy={saving}>
          Bestätigen
        </Button>
      </div>
    </form>
  );
}

/**
 * A drawn first page, standing in for the PDF preview until #06 renders real pages. It fits the panel whole, by width
 * or by height, whichever is smaller, and its type scales with it (container units). Paper stays white in dark mode.
 */
function PaperPreview({ document }: { document: SampleDocument }) {
  return (
    <div
      className="flex aspect-[1/1.414] w-[min(100cqw,calc(100cqh/1.414))] shrink-0 flex-col gap-[4cqw] rounded-[2px] bg-white p-[7cqw] text-[#1d1d1f] shadow-[0_0_0_1px_rgb(0_0_0/0.08),0_4px_16px_rgb(0_0_0/0.12)] [container-type:inline-size]"
      style={{ fontFamily: "Georgia, serif" }}
    >
      <div className="flex items-start justify-between">
        <div className="flex flex-col gap-[1cqw]">
          <span className="text-[4.2cqw] font-semibold">{document.sender}</span>
          <span className="text-[2.6cqw] text-[#555]">Musterweg 1, 12345 Beispielstadt</span>
        </div>
        <span className="size-[10cqw] rounded-full bg-[#e8e8ea]" />
      </div>
      <div className="mt-[6cqw] flex flex-col gap-[1cqw]">
        <span className="text-[5cqw] font-semibold">{document.kind}</span>
        <span className="text-[3cqw] text-[#555]">Nr. {document.number} vom {document.date}</span>
      </div>
      <div className="flex flex-col gap-[2.4cqw]">
        {[86, 72, 90, 64].map((w, i) => (
          <div key={i} className="flex items-center justify-between gap-[4cqw]">
            <span className="h-[1.8cqw] rounded-full bg-[#e3e3e6]" style={{ width: `${w - 30}%` }} />
            <span className="h-[1.8cqw] w-[14%] rounded-full bg-[#e3e3e6]" />
          </div>
        ))}
      </div>
      {document.cents > 0 && (
        <div className="flex justify-between border-t border-[#ddd] pt-[3cqw] text-[3.4cqw] font-semibold">
          <span>Gesamtbetrag</span>
          <span className="tabular-nums">{formatCents(document.cents)}</span>
        </div>
      )}
    </div>
  );
}

function Segments<T extends string>({ value, onChange, options }: { value: T; onChange: (value: T) => void; options: { value: T; label: string }[] }) {
  return (
    <div role="radiogroup" className="flex rounded-control bg-canvas p-0.5 shadow-[inset_0_0_0_1px_var(--line)]">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={value === o.value}
          onClick={() => onChange(o.value)}
          className={cn(
            "h-6 flex-1 rounded-[calc(var(--r-control)-2px)] px-2 text-small font-medium transition-quick",
            value === o.value ? "bg-surface text-fg shadow-panel" : "text-fg-muted hover:text-fg",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
