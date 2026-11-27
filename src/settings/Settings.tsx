// "Einstellungen" (#49, knowledge/decision-views.md): what is set up once, as a view with pages. The list of pages at
// the left, the chosen page at the right, the path "Einstellungen › <Seite>". The pages Ordner, Postfächer, Namen and
// Kategorien hold what four dialogs held before, with the same commands.
import { useEffect, useState } from "react";
import { type ArchiveInfo, type ModelStatus, commands } from "../bindings";
import { FoldersPage } from "../documents/Folders";
import { MailboxesPage } from "../documents/Mailboxes";
import { TemplatesPage } from "../documents/Names";
import { CategoriesPage } from "../money/Categories";
import { Button, Icon, Segmented, cn, type IconName } from "../ui";
import { Page, Panel } from "./Page";
import { chooseTheme, storedTheme, type Theme } from "./theme";

export type SettingsPageId = "ordner" | "postfaecher" | "namen" | "kategorien" | "modell" | "archiv" | "darstellung" | "ueber";

/** The pages with the words the owner may search them by (rule 22). */
export const settingsPages: { id: SettingsPageId; label: string; icon: IconName; words: string }[] = [
  { id: "ordner", label: "Ordner", icon: "folder", words: "Ordner beobachten, Eingang, Downloads" },
  { id: "postfaecher", label: "Postfächer", icon: "mail", words: "Gmail, Mail, E-Mail, Postfach, App-Passwort" },
  { id: "namen", label: "Namen", icon: "document", words: "Namensvorlagen, Dateinamen, Umbenennen" },
  { id: "kategorien", label: "Kategorien", icon: "receipt", words: "Kategorie, Ausgaben, Einnahmen" },
  { id: "modell", label: "Modell", icon: "scale", words: "KI, Ollama, lokales Modell, Verbindung" },
  { id: "archiv", label: "Archiv", icon: "inbox", words: "Archivordner, Sicherungen, Speicherort" },
  { id: "darstellung", label: "Darstellung", icon: "gear", words: "Hell, Dunkel, Farbschema, Aussehen" },
  { id: "ueber", label: "Über", icon: "more", words: "Version, Lizenzen" },
];

export function Settings({ page, onPage, version }: { page: SettingsPageId; onPage: (page: SettingsPageId) => void; version: string | null }) {
  // A problem of a folder or a mailbox shows in the list of pages too.
  const [problems, setProblems] = useState<Set<SettingsPageId>>(new Set());
  useEffect(() => {
    void Promise.all([commands.listFolders(), commands.listMailboxes()]).then(([folders, mailboxes]) => {
      const found = new Set<SettingsPageId>();
      if (folders.status === "ok" && folders.data.some((folder) => folder.problem)) found.add("ordner");
      if (mailboxes.status === "ok" && mailboxes.data.some((mailbox) => mailbox.problem)) found.add("postfaecher");
      setProblems(found);
    });
  }, [page]);
  return (
    <div className="grid min-h-0 flex-1 grid-cols-[200px_minmax(0,1fr)] gap-6 px-6 pt-4 pb-6">
      <nav aria-label="Seiten der Einstellungen" className="flex flex-col gap-px">
        {settingsPages.map((entry) => (
          <button
            key={entry.id}
            type="button"
            aria-current={entry.id === page ? "page" : undefined}
            onClick={() => onPage(entry.id)}
            className={cn(
              "flex h-8 items-center gap-2.5 rounded-row px-2 text-left transition-quick",
              entry.id === page ? "bg-selected font-medium text-fg" : "text-fg-muted hover:bg-hover hover:text-fg",
            )}
          >
            <Icon name={entry.icon} size={16} />
            <span className="flex-1">{entry.label}</span>
            {problems.has(entry.id) && <span aria-label="Problem" className="size-2 rounded-full bg-danger" />}
          </button>
        ))}
      </nav>
      <div className="flex min-h-0 flex-col overflow-y-auto">
        {page === "ordner" && <FoldersPage />}
        {page === "postfaecher" && <MailboxesPage />}
        {page === "namen" && <TemplatesPage />}
        {page === "kategorien" && <CategoriesPage />}
        {page === "modell" && <ModelPage />}
        {page === "archiv" && <ArchivePage />}
        {page === "darstellung" && <LookPage />}
        {page === "ueber" && <AboutPage version={version} />}
      </div>
    </div>
  );
}

function Row({ name, children }: { name: string; children: React.ReactNode }) {
  return (
    <div className="flex min-h-11 items-center gap-4 border-b border-line py-2 last:border-b-0">
      <span className="w-36 shrink-0 text-small text-fg-muted">{name}</span>
      <span className="min-w-0 flex-1 truncate">{children}</span>
    </div>
  );
}

function ModelPage() {
  const [status, setStatus] = useState<ModelStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const test = async () => {
    setBusy(true);
    const result = await commands.modelStatus();
    setBusy(false);
    if (result.status === "ok") setStatus(result.data);
  };
  useEffect(() => {
    void test();
  }, []);
  return (
    <Page
      actions={
        <Button kind="primary" busy={busy} onClick={() => void test()}>
          Verbindung testen
        </Button>
      }
    >
      <Panel>
        <Row name="Modell">{status?.name ?? "…"}</Row>
        <Row name="Adresse">{status?.address || "…"}</Row>
        <Row name="Zustand">
          {status ? (
            <span className="inline-flex items-center gap-2">
              <span className={cn("size-2 rounded-full", status.installed ? "bg-ok" : "bg-danger")} />
              {status.installed ? "bereit" : (status.problem ?? "nicht erreichbar")}
            </span>
          ) : (
            "…"
          )}
        </Row>
      </Panel>
    </Page>
  );
}

function ArchivePage() {
  const [info, setInfo] = useState<ArchiveInfo | null>(null);
  useEffect(() => {
    void commands.archiveInfo().then((result) => {
      if (result.status === "ok") setInfo(result.data);
    });
  }, []);
  const number = (count: number) => count.toLocaleString("de-DE");
  return (
    <Page>
      <Panel>
        <Row name="Ort">{info?.folder ?? "…"}</Row>
        <Row name="Dokumente">{info ? number(info.documents) : "…"}</Row>
        <Row name="Buchungen">{info ? number(info.bookings) : "…"}</Row>
        <Row name="Konten">{info ? number(info.accounts) : "…"}</Row>
      </Panel>
      {info && info.snapshots.length > 0 && (
        <section aria-label="Sicherungen" className="flex flex-col gap-2">
          <span className="text-small font-medium text-fg-muted">Sicherungen</span>
          <Panel>
            {info.snapshots.map((snapshot) => (
              <Row key={snapshot.name} name={new Date(snapshot.takenAt).toLocaleString("de-DE", { dateStyle: "medium", timeStyle: "short" })}>
                <span className="text-fg-muted">{snapshot.reason}</span>
              </Row>
            ))}
          </Panel>
        </section>
      )}
    </Page>
  );
}

function LookPage() {
  const [theme, setTheme] = useState<Theme>(storedTheme());
  return (
    <Page>
      <Panel>
        <Row name="Farbschema">
          <Segmented
            label="Farbschema"
            value={theme}
            onChange={(value) => {
              setTheme(value);
              chooseTheme(value);
            }}
            options={[
              { value: "system", label: "Wie das System" },
              { value: "light", label: "Hell" },
              { value: "dark", label: "Dunkel" },
            ]}
          />
        </Row>
      </Panel>
    </Page>
  );
}

/** The pieces and packages of the window with their sources and licences (knowledge/decision-ui-libraries.md). */
export const licences: { name: string; what: string; licence: string }[] = [
  { name: "React 19.3.1", what: "Oberfläche", licence: "MIT" },
  { name: "Tauri 2", what: "Fenster und Kern", licence: "MIT oder Apache-2.0" },
  { name: "motion 14.14.0", what: "Bewegung", licence: "MIT" },
  { name: "cmdk 1.1.1", what: "Schnellsuche", licence: "MIT" },
  { name: "clsx 2.1.1", what: "Klassen", licence: "MIT" },
  { name: "Beautiful UI", what: "Liste, Menü, Felder, leerer Zustand", licence: "MIT, Shane Levine" },
  { name: "Magic UI", what: "Einblenden des leeren Zustands", licence: "MIT" },
  { name: "Aceternity UI", what: "Ideen für Dialog und Knopf", licence: "nur die Idee" },
  { name: "RareUI", what: "Ideen für Spinner, Schalter, Segmente", licence: "nur die Idee" },
  { name: "21st.dev", what: "Ideen für Pfad, Tasten, Ladezeilen, Filter", licence: "nur die Idee" },
  { name: "PDFium", what: "Seiten der Dokumente", licence: "BSD-3-Clause und Apache-2.0" },
];

function AboutPage({ version }: { version: string | null }) {
  return (
    <Page>
      <Panel>
        <Row name="Version">{version ?? "…"}</Row>
      </Panel>
      <section aria-label="Lizenzen" className="flex flex-col gap-2">
        <span className="text-small font-medium text-fg-muted">Lizenzen</span>
        <Panel>
          {licences.map((entry) => (
            <Row key={entry.name} name={entry.name}>
              <span className="flex justify-between gap-3">
                <span className="truncate">{entry.what}</span>
                <span className="shrink-0 text-small text-fg-muted">{entry.licence}</span>
              </span>
            </Row>
          ))}
        </Panel>
      </section>
    </Page>
  );
}
