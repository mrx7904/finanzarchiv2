import { lazy, Suspense, useCallback, useEffect, useReducer, useState } from "react";
import { type LoadedStatus, describeLoaded } from "./archive-status";
import { type DocumentRow_Serialize as DocumentRow, commands } from "./bindings";
import { Documents } from "./documents/Documents";
import { OrdersPage } from "./documents/Orders";
import { IntakeLine } from "./intake/IntakeLine";
import { useIntakeChanged } from "./intake/useIntakeChanged";
import { Accounts } from "./money/Accounts";
import { Bookings } from "./money/Bookings";
import { moneyPath } from "./money/describe";
import { initialNav, navigate, type ViewId } from "./navigation";
import { reviewPath } from "./review/keys";
import { Review } from "./review/Review";
import { Settings, settingsPages, type SettingsPageId } from "./settings/Settings";
import { AppShell, Icon, Kbd, QuickSearch, SidebarRow, isMac, useQuickSearchKey, type IconName, type SearchEntry } from "./ui";

// The showcase of the design system (#07), only in the dev build: the branch is removed from the release.
const Showcase = import.meta.env.DEV ? lazy(() => import("./showcase/Showcase").then((module) => ({ default: module.Showcase }))) : null;

function useHash() {
  const [hash, setHash] = useState(window.location.hash);
  useEffect(() => {
    const onChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return hash;
}

/** The views in the sidebar, in their groups without a title. A view not built yet is not listed (#44). */
const groups: { id: ViewId; label: string; icon: IconName }[][] = [
  [{ id: "review", label: "Prüfen", icon: "check" }],
  [
    { id: "documents", label: "Dokumente", icon: "document" },
    { id: "accounts", label: "Konten", icon: "card" },
  ],
];

/** What the quick search finds beside the views: documents, accounts, parties and bookings, loaded when it opens. */
type Findings = { documents: DocumentRow[]; accounts: { id: number; name: string }[]; parties: string[]; bookings: { id: number; party: string; detail: string }[] };

// The frame of the views (#44, knowledge/decision-views.md): the sidebar of the views, one navigation state with the
// way back of jumps, the intake at the foot, the quick search from everywhere. Its top bar is the title bar of the
// window (decision-app-shell.md).
export function App() {
  const hash = useHash();
  const [loaded, setLoaded] = useState<LoadedStatus | null>(null);
  const [nav, dispatch] = useReducer(navigate, initialNav);
  const [waiting, setWaiting] = useState(0);
  const [searching, setSearching] = useState(false);
  const [findings, setFindings] = useState<Findings | null>(null);
  const countWaiting = () =>
    void commands.reviewGroups().then((result) => {
      if (result.status === "ok") setWaiting(result.data.reduce((sum, group) => sum + group.count, 0));
    });
  useEffect(countWaiting, []);
  useIntakeChanged(countWaiting);

  useEffect(() => {
    commands.appStatus().then(setLoaded, () =>
      setLoaded({ status: "error", error: { kind: "internal", message: "Der Kern antwortet nicht. Bitte starte Finanzarchiv neu." } }),
    );
  }, []);

  const back = useCallback(() => dispatch({ type: "back" }), []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.altKey && event.key === "ArrowLeft") {
        event.preventDefault();
        back();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [back]);

  const openSearch = useCallback(() => {
    setSearching(true);
    void loadFindings().then(setFindings);
  }, []);
  useQuickSearchKey(openSearch);

  if (Showcase && hash === "#gestaltung") {
    return (
      <Suspense fallback={null}>
        <Showcase />
      </Suspense>
    );
  }

  const ready = loaded?.status === "ok" && loaded.data.archive.state === "ready";
  if (loaded && !ready) return <ArchiveProblem loaded={loaded} />;

  const { review, documents, accounts } = nav.places;
  const openDocument = (document: number) => dispatch({ type: "jump", to: { view: "documents", place: { focus: document, orders: false } } });
  const openSettings = (page: SettingsPageId) => {
    dispatch({ type: "set", spot: { view: "settings", place: { page } } });
    dispatch({ type: "go", view: "settings" });
  };
  const entries: SearchEntry[] = searchEntries({ findings, go: (view) => dispatch({ type: "go", view }), openDocument, openBookings: (bookingsOf, text) => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf, text } } }), openSettings, back: nav.back.length > 0 ? back : null });
  const settingsLabel = settingsPages.find((entry) => entry.id === nav.places.settings.page)?.label ?? "";

  return (
    <>
      <AppShell
        onBack={nav.back.length > 0 ? back : undefined}
        path={
          !ready
            ? [{ label: "Finanzarchiv" }]
            : nav.view === "review"
              ? reviewPath(review.group?.label ?? null, review.question?.name ?? null, {
                  toStart: () => dispatch({ type: "set", spot: { view: "review", place: { group: null, question: null } } }),
                  toGroup: () => dispatch({ type: "set", spot: { view: "review", place: { group: review.group, question: null } } }),
                })
              : nav.view === "accounts"
                ? moneyPath(accounts.bookingsOf?.name ?? null, accounts.bookingsOf !== undefined, {
                    toAccounts: () => dispatch({ type: "set", spot: { view: "accounts", place: { bookingsOf: undefined, text: "" } } }),
                  })
                : nav.view === "settings"
                  ? [{ label: "Einstellungen", onSelect: () => openSettings("ordner") }, { label: settingsLabel }]
                  : documents.orders
                    ? [{ label: "Dokumente", onSelect: () => dispatch({ type: "set", spot: { view: "documents", place: { ...documents, orders: false } } }) }, { label: "Bestellungen" }]
                    : [{ label: "Dokumente" }]
        }
        sidebar={
          ready ? (
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
                {groups.map((group) => (
                  <ul key={group[0]?.id} className="flex flex-col gap-px">
                    {group.map((entry) => (
                      <li key={entry.id}>
                        <SidebarRow
                          label={entry.label}
                          icon={entry.icon}
                          count={entry.id === "review" ? waiting : undefined}
                          selected={nav.view === entry.id}
                          onSelect={() => dispatch({ type: "go", view: entry.id })}
                        />
                      </li>
                    ))}
                  </ul>
                ))}
              </nav>
              <div className="flex flex-col gap-px">
                <IntakeLine />
                <SidebarRow label="Einstellungen" icon="gear" selected={nav.view === "settings"} onSelect={() => dispatch({ type: "go", view: "settings" })} />
              </div>
            </div>
          ) : undefined
        }
      >
        {!loaded ? null : nav.view === "documents" && documents.orders ? (
          <OrdersPage onOpenDocument={openDocument} />
        ) : nav.view === "documents" ? (
          <Documents
            focus={documents.focus}
            onChoose={(focus) => dispatch({ type: "set", spot: { view: "documents", place: { focus, orders: false } } })}
            onJump={openDocument}
            onOpenBooking={(account) => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: account, text: "" } } })}
            onOrders={() => dispatch({ type: "set", spot: { view: "documents", place: { ...documents, orders: true } } })}
          />
        ) : nav.view === "settings" ? (
          <Settings page={nav.places.settings.page} onPage={openSettings} version={loaded.status === "ok" ? loaded.data.version : null} />
        ) : nav.view === "review" ? (
          <main className="flex min-h-0 flex-1 flex-col items-center gap-6 overflow-y-auto px-8 py-8">
            <Review at={review} onAt={(at) => dispatch({ type: "set", spot: { view: "review", place: at } })} onAnswered={countWaiting} />
          </main>
        ) : (
          <main className="flex min-h-0 flex-1 flex-col items-center gap-6 overflow-y-auto px-8 py-8">
            {accounts.bookingsOf === undefined ? (
              <Accounts
                onOpen={(account) =>
                  dispatch({ type: "set", spot: { view: "accounts", place: { bookingsOf: account ? { id: account.id, name: account.name } : null, text: "" } } })
                }
              />
            ) : (
              <Bookings account={accounts.bookingsOf?.id ?? null} text={accounts.text} onOpenDocument={openDocument} />
            )}
          </main>
        )}
      </AppShell>
      <QuickSearch open={searching} onClose={() => setSearching(false)} entries={entries} />
    </>
  );
}

async function loadFindings(): Promise<Findings> {
  const [documents, accounts, bookings] = await Promise.all([
    commands.listDocuments(),
    commands.listAccounts(),
    commands.listBookings({ account: null, offset: 0, limit: 2000 }),
  ]);
  const rows = bookings.status === "ok" ? bookings.data.rows : [];
  return {
    documents: documents.status === "ok" ? documents.data : [],
    accounts: accounts.status === "ok" ? accounts.data.map((account) => ({ id: account.id, name: account.name })) : [],
    parties: [...new Set(rows.map((row) => row.party).filter((party): party is string => Boolean(party)))].sort((a, b) => a.localeCompare(b, "de")),
    bookings: rows.slice(0, 500).map((row) => ({
      id: row.id,
      party: row.party ?? row.otherSide ?? row.purpose,
      detail: `${row.day.split("-").reverse().join(".")} · ${row.incoming ? "+" : ""}${row.amount}`,
    })),
  };
}

type SearchContext = {
  findings: Findings | null;
  go: (view: ViewId) => void;
  openDocument: (document: number) => void;
  openBookings: (bookingsOf: { id: number; name: string } | null, text: string) => void;
  openSettings: (page: SettingsPageId) => void;
  back: (() => void) | null;
};

/** What the quick search finds and runs (rule 22): the views, the actions, and what the archive holds. */
export function searchEntries({ findings, go, openDocument, openBookings, openSettings, back }: SearchContext): SearchEntry[] {
  const entries: SearchEntry[] = groups.flat().map((view) => ({ id: `view:${view.id}`, group: "Ansichten", label: view.label, icon: view.icon, onSelect: () => go(view.id) }));
  entries.push({ id: "view:settings", group: "Ansichten", label: "Einstellungen", icon: "gear", onSelect: () => go("settings") });
  for (const page of settingsPages) {
    entries.push({ id: `settings:${page.id}`, group: "Einstellungen", label: page.label, keywords: [page.words], icon: page.icon, onSelect: () => openSettings(page.id) });
  }
  if (back) entries.push({ id: "action:back", group: "Aktionen", label: "Zurück", icon: "back", onSelect: back });
  entries.push({ id: "action:pause", group: "Aktionen", label: "Eingang pausieren oder fortsetzen", icon: "inbox", onSelect: () => void togglePause() });
  if (Showcase) entries.push({ id: "action:showcase", group: "Aktionen", label: "Gestaltung ansehen", icon: "more", onSelect: () => (window.location.hash = "gestaltung") });
  if (!findings) return entries;
  for (const document of findings.documents) {
    entries.push({ id: `document:${document.id}`, group: "Dokumente", label: document.name, detail: document.originalName, icon: "document", onSelect: () => openDocument(document.id) });
  }
  for (const account of findings.accounts) {
    entries.push({ id: `account:${account.id}`, group: "Konten", label: account.name, icon: "card", onSelect: () => openBookings(account, "") });
  }
  for (const party of findings.parties) {
    entries.push({ id: `party:${party}`, group: "Partner", label: party, icon: "receipt", onSelect: () => openBookings(null, party) });
  }
  for (const booking of findings.bookings) {
    entries.push({ id: `booking:${booking.id}`, group: "Buchungen", label: booking.party, detail: booking.detail, icon: "card", onSelect: () => openBookings(null, booking.party) });
  }
  return entries;
}

async function togglePause() {
  const overview = await commands.intakeOverview();
  if (overview.status === "ok") await commands.setIntakePaused(!overview.data.paused);
}

/** A damaged or missing archive replaces every view, sidebar included, by one page: the sentence and the snapshots. */
function ArchiveProblem({ loaded }: { loaded: LoadedStatus }) {
  const line = describeLoaded(loaded);
  const archive = loaded.status === "ok" ? loaded.data.archive : null;
  const snapshots = archive && archive.state !== "ready" ? archive.snapshots : [];
  return (
    <AppShell path={[{ label: "Finanzarchiv" }]}>
      <main className="flex min-h-0 flex-1 flex-col items-center justify-center gap-5 overflow-y-auto px-8 py-8">
        <p className="max-w-xl text-center text-danger">{line.text}</p>
        {snapshots.length > 0 && (
          <ul aria-label="Sicherungen" className="flex w-full max-w-md flex-col gap-px rounded-panel bg-surface p-1 shadow-panel">
            {snapshots.map((snapshot) => (
              <li key={snapshot.name} className="flex h-10 items-center gap-3 rounded-row px-3">
                <Icon name="folder" size={16} className="text-fg-muted" />
                <span className="flex-1 truncate">{new Date(snapshot.takenAt).toLocaleString("de-DE", { dateStyle: "medium", timeStyle: "short" })}</span>
              </li>
            ))}
          </ul>
        )}
      </main>
    </AppShell>
  );
}
