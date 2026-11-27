// The sub pages and the states of the draft (docs/entwurf-ansichten.md): what lies below a view of the sidebar, and
// how a view looks when it is empty, loads, fails or stands in a narrow window. Invented data, no call to the core.
import { useState, type ReactNode } from "react";
import { AppShell, Button, Capsule, EmptyState, Icon, List, ListRow, SelectField, TextField, cn, type IconName, type Tone } from "../ui";
import { Dokumente, Einstellungen, Frame, Heading, Page, Segments, Umsaetze, panel } from "./Entwurf";

// ---------- Prüfen: reasons, then the documents of one reason ----------

function PruefenGruende() {
  return (
    <Frame view="pruefen" path={["Prüfen"]} actions={<Button kind="primary">Der Reihe nach prüfen</Button>}>
      <main className="min-h-0 flex-1 overflow-y-auto px-10 py-9">
        <div className="flex max-w-[720px] flex-col gap-6">
          <header className="flex flex-col gap-1">
            <h1 className="text-title font-semibold tracking-[-0.025em]">4 Dokumente warten auf deine Antwort.</h1>
            <p className="text-fg-muted">Alles andere seit Montag ist abgelegt, 19 Dokumente.</p>
          </header>
          <div className={panel}>
            <List label="Gründe">
              <ListRow title="Betrag unklar" detail="Die Positionen ergeben eine andere Summe als der Gesamtbetrag" figure="2" leading={<Icon name="receipt" size={16} />} selected onSelect={() => undefined} />
              <ListRow title="Art unklar" detail="Rechnung, Zahlungsbeleg oder etwas anderes" figure="1" leading={<Icon name="document" size={16} />} onSelect={() => undefined} />
              <ListRow title="Vielleicht doppelt" detail="Gleicher Absender, gleiche Nummer, andere Datei" figure="1" leading={<Icon name="inbox" size={16} />} onSelect={() => undefined} />
            </List>
          </div>
        </div>
      </main>
    </Frame>
  );
}

function PruefenListe() {
  return (
    <Frame view="pruefen" path={["Prüfen", "Betrag unklar"]} actions={<Button kind="primary">Frage öffnen</Button>}>
      <main className="grid min-h-0 flex-1 grid-cols-[440px_minmax(0,1fr)] gap-5 p-5">
        <section className={cn(panel, "min-h-0 overflow-y-auto")}>
          <List label="Betrag unklar">
            <ListRow title="Stadtwerke Beispielstadt" detail="Ist 86,40 € der Betrag dieser Rechnung?" leading={<Icon name="receipt" size={16} />} selected onSelect={() => undefined} />
            <ListRow title="Telefon Nord GmbH" detail="Ist 39,99 € der Betrag dieser Rechnung?" leading={<Icon name="receipt" size={16} />} onSelect={() => undefined} />
          </List>
        </section>
        <section aria-label="Seite" className={cn(panel, "flex min-h-0 items-center justify-center overflow-hidden p-5")}>
          <div className="flex size-full items-center justify-center [container-type:size]">
            <Page marked className="h-[min(100cqh,141.4cqw)]" />
          </div>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Dokumente › Bestellungen ----------

const orders: { title: string; detail: string; figure: string; state: { tone: Tone; label: string } }[] = [
  { title: "Schreibtischlampe, Ersatzbirne", detail: "Amazon, bestellt am 09.11.2026", figure: "64,90 €", state: { tone: "danger", label: "fehlt" } },
  { title: "Druckerpapier 2500 Blatt", detail: "Amazon, bestellt am 02.11.2026", figure: "23,49 €", state: { tone: "danger", label: "fehlt" } },
  { title: "Fachbuch Datenbanken", detail: "Amazon, bestellt am 24.10.2026", figure: "49,99 €", state: { tone: "ok", label: "vollständig" } },
  { title: "USB-Kabel, Netzteil", detail: "Amazon, bestellt am 11.10.2026", figure: "31,98 €", state: { tone: "ok", label: "vollständig" } },
  { title: "Fahrradschloss", detail: "Amazon, bestellt am 29.09.2026", figure: "39,95 €", state: { tone: "ok", label: "vollständig" } },
];

export function Bestellungen() {
  return (
    <Frame view="dokumente" path={["Dokumente", "Bestellungen"]}>
      <main className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_400px] gap-5 p-5">
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <p className="border-b border-line px-4 py-3 text-small text-fg-muted">38 Bestellungen in 2026, bei 2 fehlt die Rechnung</p>
          <List label="Bestellungen" className="min-h-0 flex-1 overflow-y-auto">
            {orders.map((row, at) => (
              <ListRow key={row.title} title={row.title} detail={row.detail} figure={row.figure} state={row.state} leading={<Icon name="receipt" size={16} />} selected={at === 0} onSelect={() => undefined} />
            ))}
          </List>
        </section>
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex flex-col gap-1 p-5">
            <span className="text-small text-fg-muted">Amazon, bestellt am 09.11.2026</span>
            <h2 className="text-heading font-semibold">Schreibtischlampe, Ersatzbirne</h2>
            <span className="text-title font-semibold tracking-[-0.025em] tabular-nums">64,90 €</span>
          </div>
          <div className="flex flex-col divide-y divide-line border-t border-line px-5 py-1">
            {[
              ["Schreibtischlampe LED", "54,95 €"],
              ["Ersatzbirne E14", "9,95 €"],
            ].map(([name, sum]) => (
              <div key={name} className="flex items-center justify-between py-2.5">
                <span>{name}</span>
                <span className="font-medium tabular-nums">{sum}</span>
              </div>
            ))}
          </div>
          <div className="flex flex-col gap-2 border-t border-line px-5 py-4">
            <p className="text-small font-medium text-fg-muted">Bezahlt</p>
            <button type="button" className="flex items-center gap-3 rounded-row bg-canvas px-3 py-2.5 text-left">
              <Icon name="card" size={16} className="text-fg-muted" />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="font-medium">Kreditkarte, 10.11.2026</span>
                <span className="truncate text-small text-fg-muted">Amazon, Bestellung vom 09.11.</span>
              </span>
              <span className="font-medium tabular-nums">64,90 €</span>
              <Icon name="chevronRight" size={14} className="text-fg-muted" />
            </button>
          </div>
          <div className="flex flex-col gap-2 border-t border-line px-5 py-4">
            <p className="text-small font-medium text-fg-muted">Rechnung</p>
            <div className="flex items-center gap-3 rounded-row px-3 py-3 shadow-[inset_0_0_0_1px_var(--line-strong)]">
              <span aria-hidden className="size-2 shrink-0 rounded-full bg-danger" />
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="font-medium">Noch keine Rechnung</span>
                <span className="text-small text-fg-muted">Kommt sie per Mail oder in einen Ordner, findet sie von selbst hierher.</span>
              </span>
            </div>
          </div>
          <div className="mt-auto flex items-center justify-end gap-2 border-t border-line px-5 py-3">
            <Button size="small" kind="plain">
              Braucht keine Rechnung
            </Button>
            <Button size="small">Rechnung zuordnen …</Button>
          </div>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Konten › Wiederkehrend ----------

const recurring: { title: string; detail: string; figure: string; state?: { tone: Tone; label: string } }[] = [
  { title: "Hausverwaltung Birke", detail: "monatlich, nächste am 01.12.", figure: "780,00 €" },
  { title: "Stadtwerke Beispielstadt", detail: "monatlich, nächste am 18.12.", figure: "86,40 €", state: { tone: "warn", label: "gestiegen" } },
  { title: "Telefon Nord GmbH", detail: "monatlich, nächste am 12.12.", figure: "39,99 €" },
  { title: "Tonspur", detail: "monatlich, nächste am 15.12.", figure: "10,99 €", state: { tone: "warn", label: "doppelt?" } },
  { title: "Klangwelt", detail: "monatlich, nächste am 03.12.", figure: "10,99 €", state: { tone: "warn", label: "doppelt?" } },
  { title: "Versicherung Muster AG", detail: "jährlich, nächste im November 2027", figure: "214,50 €" },
  { title: "Sportverein Eiche", detail: "vierteljährlich, seit Juli keine Zahlung", figure: "45,00 €", state: { tone: "neutral", label: "beendet" } },
];

function Wiederkehrend() {
  return (
    <Frame view="konten" path={["Konten", "Wiederkehrend"]}>
      <main className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_400px] gap-5 p-5">
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex items-end justify-between gap-4 border-b border-line px-4 py-3.5">
            <div className="flex flex-col">
              <span className="text-small text-fg-muted">14 Zahlungen, im Monat</span>
              <span className="text-title font-semibold tracking-[-0.025em] tabular-nums">1.186,40 €</span>
            </div>
            <span className="pb-1 text-small text-fg-muted tabular-nums">im Jahr 14.236,80 €</span>
          </div>
          <List label="Wiederkehrende Zahlungen" className="min-h-0 flex-1 overflow-y-auto">
            {recurring.map((row, at) => (
              <ListRow key={row.title} title={row.title} detail={row.detail} figure={row.figure} state={row.state ?? { tone: "neutral", label: "läuft" }} selected={at === 1} onSelect={() => undefined} />
            ))}
          </List>
        </section>
        <section className={cn(panel, "flex min-h-0 flex-col")}>
          <div className="flex flex-col gap-1 p-5">
            <span className="text-small text-fg-muted">monatlich, vom Girokonto</span>
            <h2 className="text-heading font-semibold">Stadtwerke Beispielstadt</h2>
            <span className="text-title font-semibold tracking-[-0.025em] tabular-nums">86,40 €</span>
            <span className="text-small text-fg-muted">Seit November 6,40 € mehr, vorher 80,00 €</span>
          </div>
          <div className="flex flex-col gap-2 border-t border-line px-5 py-4">
            <p className="text-small font-medium text-fg-muted">Die letzten zwölf Monate</p>
            <div className="flex items-end gap-1.5" style={{ height: 64 }}>
              {[80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 80, 88].map((height, at) => (
                <span key={at} className={cn("flex-1 rounded-t-[3px]", at === 11 ? "bg-accent" : "bg-line-strong")} style={{ height: `${height}%` }} />
              ))}
            </div>
            <div className="flex justify-between text-small text-fg-muted">
              <span>Dez 2025</span>
              <span>Nov 2026</span>
            </div>
          </div>
          <div className="flex flex-col divide-y divide-line border-t border-line px-5 py-1">
            {[
              ["Nächste erwartet", "18.12.2026"],
              ["Im Jahr 2026", "950,40 €"],
              ["Kategorie", "Wohnen und Energie"],
            ].map(([label, value]) => (
              <div key={label} className="flex items-center justify-between py-2.5">
                <span className="text-fg-muted">{label}</span>
                <span className="font-medium tabular-nums">{value}</span>
              </div>
            ))}
          </div>
          <div className="mt-auto flex items-center justify-between gap-2 border-t border-line px-5 py-3">
            <Button size="small" kind="plain">
              Ist keine wiederkehrende Zahlung
            </Button>
            <Button size="small">Buchungen zeigen</Button>
          </div>
        </section>
      </main>
    </Frame>
  );
}

// ---------- Einstellungen: the seven other pages ----------

function Row({ icon, title, detail, children }: { icon?: IconName; title: string; detail?: string; children?: ReactNode }) {
  return (
    <div className="flex min-h-(--row-height) items-center gap-3 px-5 py-2.5">
      {icon && (
        <span className="flex size-8 shrink-0 items-center justify-center rounded-control bg-canvas text-fg-muted">
          <Icon name={icon} size={16} />
        </span>
      )}
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{title}</span>
        {detail && <span className="truncate text-small text-fg-muted">{detail}</span>}
      </span>
      {children}
    </div>
  );
}

function Switch({ on }: { on: boolean }) {
  return (
    <span role="switch" aria-checked={on} className={cn("flex h-5.5 w-9.5 shrink-0 items-center rounded-full p-0.5", on ? "justify-end bg-accent" : "justify-start bg-line-strong")}>
      <span className={cn("size-4.5 rounded-full", on ? "bg-on-accent" : "bg-surface")} />
    </span>
  );
}

const rows = "flex flex-col divide-y divide-line";

export function Ordner() {
  return (
    <Einstellungen page="Ordner">
      <div className={cn(panel, rows)}>
        <Row icon="inbox" title="Eingangsordner" detail="D:\Finanzen\Eingang, was hier landet, wird gelesen und ins Archiv gelegt">
          <Button size="small" kind="plain">
            Verlegen …
          </Button>
        </Row>
      </div>
      <div className={cn(panel, rows)}>
        <div className="flex items-center justify-between px-5 py-3.5">
          <Heading>Beobachtete Ordner</Heading>
          <Button size="small" icon="plus">
            Ordner hinzufügen …
          </Button>
        </div>
        <Row icon="folder" title="Scans" detail="D:\Scans, zuletzt gelesen heute um 08:40">
          <Capsule tone="ok">beobachtet</Capsule>
          <Button size="small" kind="plain" icon="trash" label="Scans entfernen" />
        </Row>
        <Row icon="folder" title="Steuer 2026" detail="E:\Steuer 2026, das Laufwerk ist nicht verbunden">
          <Capsule tone="danger">fehlt</Capsule>
          <Button size="small" kind="plain" icon="trash" label="Steuer 2026 entfernen" />
        </Row>
      </div>
      <div className={cn(panel, rows)}>
        <Row icon="upload" title="Downloads beobachten" detail="Nur was wie eine Rechnung, ein Kontoauszug oder eine Abrechnung aussieht">
          <Switch on />
        </Row>
      </div>
      <p className="px-1 text-small text-fg-muted">Die Originale bleiben in ihrem Ordner. Das Archiv legt sich eine eigene Kopie an.</p>
    </Einstellungen>
  );
}

function Namen() {
  const templates: [string, string, string][] = [
    ["Rechnung", "{Datum} {Absender} Rechnung", "2026-11-14 Stadtwerke Beispielstadt Rechnung"],
    ["Gehaltsabrechnung", "{Jahr}-{Monat} {Arbeitgeber} Gehalt", "2026-10 Beispielwerk GmbH Gehalt"],
    ["Kontoauszug", "{Jahr}-{Monat} {Bank} {Konto}", "2026-10 Musterbank Girokonto"],
    ["Zahlungsbeleg", "{Datum} {Absender} Beleg", "2026-10-29 Praxis am Markt Beleg"],
    ["Sonstiges", "{Datum} {Absender}", "2026-09-03 Hausverwaltung Birke"],
  ];
  return (
    <Einstellungen page="Namen">
      <div className={cn(panel, "flex flex-col gap-4 p-5")}>
        {templates.map(([kind, template, example]) => (
          <TextField key={kind} label={kind} value={template} onChange={() => undefined} hint={`Ergibt: ${example}`} />
        ))}
        <div className="flex items-center justify-between gap-3 border-t border-line pt-4">
          <span className="text-small text-fg-muted">1.284 Dokumente bekommen ihren Namen neu. Von dir benannte bleiben.</span>
          <Button kind="primary">Übernehmen</Button>
        </div>
      </div>
    </Einstellungen>
  );
}

function Kategorien() {
  const [added, setAdded] = useState("");
  const spending: [string, string][] = [
    ["Wohnen und Energie", "6 Absender"],
    ["Lebensmittel", "9 Absender"],
    ["Auto", "4 Absender"],
    ["Versicherungen", "3 Absender"],
    ["Unterhaltung", "5 Absender"],
    ["Telefon und Internet", "2 Absender"],
  ];
  return (
    <Einstellungen page="Kategorien">
      <div className={cn(panel, rows)}>
        <div className="flex items-center justify-between px-5 py-3.5">
          <Heading aside="13 Kategorien">Ausgaben</Heading>
        </div>
        {spending.map(([name, count]) => (
          <div key={name} className="group flex h-11 items-center gap-3 px-5">
            <span className="flex-1 font-medium">{name}</span>
            <span className="text-small text-fg-muted">{count}</span>
            <Button size="small" kind="plain" icon="trash" label={`${name} entfernen`} className={name === "Auto" ? "" : "invisible"} />
          </div>
        ))}
        <div className="px-5 py-2.5 text-small text-fg-muted">7 weitere</div>
      </div>
      <div className={cn(panel, rows)}>
        <div className="flex items-center justify-between px-5 py-3.5">
          <Heading aside="5 Kategorien">Einnahmen</Heading>
        </div>
        {["Gehalt", "Erstattungen", "Zinsen"].map((name) => (
          <div key={name} className="flex h-11 items-center gap-3 px-5">
            <span className="flex-1 font-medium">{name}</span>
          </div>
        ))}
      </div>
      <form className={cn(panel, "grid grid-cols-[minmax(0,1fr)_160px_auto] items-end gap-3 p-5")} onSubmit={(event) => event.preventDefault()}>
        <TextField label="Neue Kategorie" value={added} onChange={setAdded} />
        <SelectField label="Art" value="out" onChange={() => undefined} options={[{ value: "out", label: "Ausgabe" }, { value: "in", label: "Einnahme" }]} />
        <Button type="submit" icon="plus">
          Hinzufügen
        </Button>
      </form>
    </Einstellungen>
  );
}

function Modell() {
  return (
    <Einstellungen page="Modell">
      <div className={cn(panel, rows)}>
        <Row icon="search" title="Das lokale Modell ist erreichbar" detail="Ollama auf diesem Computer, zuletzt geprüft vor 2 Minuten">
          <Capsule tone="ok">bereit</Capsule>
        </Row>
        <div className="flex items-center gap-2 px-5 py-3">
          <Button size="small">Verbindung testen</Button>
        </div>
      </div>
      <div className={cn(panel, "flex flex-col gap-4 p-5")}>
        <div className="grid grid-cols-2 gap-3">
          <SelectField label="Modell" value="m" onChange={() => undefined} options={[{ value: "m", label: "beispiel-modell 12b" }]} hint="Liest Text und Seitenbilder" />
          <TextField label="Adresse" value="127.0.0.1:11434" onChange={() => undefined} hint="Nur ein Computer von dir" />
        </div>
      </div>
      <p className="px-1 text-small text-fg-muted">Das Modell liest nur, was die festen Leser nicht lesen konnten. Kein Dokument und kein Betrag verlässt deine Computer.</p>
    </Einstellungen>
  );
}

export function Archiv() {
  return (
    <Einstellungen page="Archiv">
      <div className={cn(panel, rows)}>
        <Row icon="folder" title="D:\Finanzen\Archiv" detail="1.284 Dokumente, 1.730 Buchungen, 412 MB">
          <Button size="small" kind="plain">
            Im Ordner zeigen
          </Button>
          <Button size="small" kind="plain">
            Umziehen …
          </Button>
        </Row>
      </div>
      <div className={cn(panel, rows)}>
        <div className="flex items-center justify-between px-5 py-3.5">
          <Heading aside="täglich und nach jedem großen Import">Sicherungen</Heading>
          <Button size="small">Jetzt sichern</Button>
        </div>
        {(
          [
            ["Heute, 08:02", "beim Start"],
            ["Gestern, 19:41", "nach dem Import von 135 Dateien"],
            ["25.11.2026, 07:55", "beim Start"],
          ] satisfies [string, string][]
        ).map(([when, why]) => (
          <Row key={when} title={when} detail={why}>
            <Button size="small" kind="plain">
              Wiederherstellen …
            </Button>
          </Row>
        ))}
      </div>
    </Einstellungen>
  );
}

function Darstellung() {
  return (
    <Einstellungen page="Darstellung">
      <div className={cn(panel, rows)}>
        <Row title="Aussehen" detail="Hell, dunkel oder wie dein System">
          <Segments options={["Hell", "Dunkel", "System"]} value="System" />
        </Row>
        <Row title="Größe" detail="Schrift, Felder und Diagramme wachsen zusammen">
          <Segments options={["Klein", "Mittel", "Groß"]} value="Mittel" />
        </Row>
        <Row title="Beim Start öffnen" detail="Die Ansicht, mit der Finanzarchiv beginnt">
          <Segments options={["Start", "Zuletzt geöffnet"]} value="Start" />
        </Row>
      </div>
    </Einstellungen>
  );
}

function Ueber() {
  return (
    <Einstellungen page="Über">
      <div className={cn(panel, rows)}>
        <Row icon="folder" title="Finanzarchiv 0.5.0" detail="Auf diesem Computer gebaut am 27.11.2026">
          <Button size="small" kind="plain">
            Was ist neu?
          </Button>
        </Row>
      </div>
      <div className={cn(panel, rows)}>
        <div className="px-5 py-3.5">
          <Heading aside="Stücke und ihre Lizenzen">Gebaut mit</Heading>
        </div>
        {[
          ["Tauri", "Fenster und Kern", "MIT, Apache 2.0"],
          ["PDFium", "Text und Seiten einer PDF", "Apache 2.0"],
          ["Motion", "Bewegung", "MIT"],
          ["cmdk", "Schnellsuche", "MIT"],
          ["Beautiful UI", "Liste, Menü, Feld, Leerzustand", "MIT"],
          ["Magic UI", "Einblenden", "MIT"],
        ].map(([name, what, licence]) => (
          <div key={name} className="flex h-11 items-center gap-3 px-5">
            <span className="w-36 font-medium">{name}</span>
            <span className="flex-1 text-small text-fg-muted">{what}</span>
            <span className="text-small text-fg-muted">{licence}</span>
          </div>
        ))}
      </div>
    </Einstellungen>
  );
}

// ---------- States ----------

function DokumenteLeer() {
  return (
    <Frame
      view="dokumente"
      path={["Dokumente"]}
      actions={
        <Button kind="primary" icon="plus">
          Importieren
        </Button>
      }
    >
      <main className="flex min-h-0 flex-1 p-5">
        <section className={cn(panel, "flex flex-1 items-center justify-center")}>
          <EmptyState
            icon="document"
            title="Noch keine Dokumente im Archiv"
            hint="Zieh Dateien oder einen Ordner ins Fenster. Was hereinkommt, wird gelesen und abgelegt."
            actions={
              <>
                <Button kind="primary" icon="upload">
                  Dateien auswählen …
                </Button>
                <Button icon="folder">Ordner beobachten …</Button>
              </>
            }
          />
        </section>
      </main>
    </Frame>
  );
}

function PruefenLeer() {
  return (
    <Frame view="pruefen" path={["Prüfen"]}>
      <main className="flex min-h-0 flex-1 p-5">
        <section className={cn(panel, "flex flex-1 items-center justify-center")}>
          <EmptyState icon="check" title="Nichts zu prüfen" hint="Alles, was hereinkam, ist abgelegt. Heute waren es 23 Dokumente." actions={<Button>Abgelegtes ansehen</Button>} />
        </section>
      </main>
    </Frame>
  );
}

function AuswertungenFehler() {
  return (
    <Frame view="auswertungen" path={["Auswertungen"]} actions={<Segments options={["2024", "2025", "2026"]} value="2026" />}>
      <main className="min-h-0 flex-1 overflow-y-auto px-10 py-9">
        <div className="flex max-w-[560px] flex-col items-start gap-4">
          <div className="flex flex-col gap-1">
            <h1 className="flex items-center gap-2.5 text-heading font-semibold">
              <span aria-hidden className="size-2 rounded-full bg-danger" />
              Die Zahlen für 2026 ließen sich nicht berechnen.
            </h1>
            <p className="text-fg-muted">Das Archiv hat nicht geantwortet. Deine Daten sind unverändert.</p>
          </div>
          <Button>Erneut versuchen</Button>
        </div>
      </main>
    </Frame>
  );
}

/** A damaged or missing archive replaces every view: no sidebar, one sentence, the snapshots. */
function ArchivBeschaedigt() {
  return (
    <AppShell system="windows" path={[{ label: "Finanzarchiv" }]}>
      <main className="flex min-h-0 flex-1 items-center justify-center p-8">
        <div className="flex w-full max-w-[560px] flex-col gap-6">
          <div className="flex flex-col gap-2">
            <h1 className="flex items-start gap-3 text-title font-semibold tracking-[-0.025em]">
              <span aria-hidden className="mt-3 size-2.5 shrink-0 rounded-full bg-danger" />
              Die Archivdatei ist beschädigt.
            </h1>
            <p className="pl-[22px] text-fg-muted">Sie bleibt unverändert liegen. Deine Originale sind nicht betroffen. Du kannst eine Sicherung wiederherstellen.</p>
          </div>
          <div className={cn(panel, rows)}>
            {[
              ["Heute, 08:02", "1.284 Dokumente, 1.730 Buchungen", true],
              ["Gestern, 19:41", "1.284 Dokumente, 1.730 Buchungen", false],
              ["25.11.2026, 07:55", "1.149 Dokumente, 1.730 Buchungen", false],
            ].map(([when, what, newest]) => (
              <Row key={String(when)} title={String(when)} detail={String(what)}>
                <Button size="small" kind={newest ? "primary" : "secondary"}>
                  Wiederherstellen …
                </Button>
              </Row>
            ))}
          </div>
          <div className="flex gap-2 pl-1">
            <Button kind="plain" icon="folder">
              Ordner des Archivs zeigen
            </Button>
          </div>
        </div>
      </main>
    </AppShell>
  );
}

export const unterseiten = {
  "pruefen-gruende": () => <PruefenGruende />,
  "pruefen-liste": () => <PruefenListe />,
  bestellungen: () => <Bestellungen />,
  wiederkehrend: () => <Wiederkehrend />,
  "einstellungen-ordner": () => <Ordner />,
  "einstellungen-namen": () => <Namen />,
  "einstellungen-kategorien": () => <Kategorien />,
  "einstellungen-modell": () => <Modell />,
  "einstellungen-archiv": () => <Archiv />,
  "einstellungen-darstellung": () => <Darstellung />,
  "einstellungen-ueber": () => <Ueber />,
  "dokumente-leer": () => <DokumenteLeer />,
  "pruefen-leer": () => <PruefenLeer />,
  "umsaetze-laden": () => <Umsaetze loading />,
  "auswertungen-fehler": () => <AuswertungenFehler />,
  "archiv-beschaedigt": () => <ArchivBeschaedigt />,
  "dokumente-sprung": () => <Dokumente back="Girokonto" />,
  "dokumente-suche": () => <Dokumente search />,
} satisfies Record<string, () => ReactNode>;
