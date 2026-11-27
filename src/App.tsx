import { lazy, Suspense, useCallback, useEffect, useReducer, useState } from "react";
import { type LoadedStatus, describeLoaded } from "./archive-status";
import { type DocumentRow_Serialize as DocumentRow, commands } from "./bindings";
import { Documents } from "./documents/Documents";
import { Evaluations } from "./evaluations/Evaluations";
import { Start } from "./start/Start";
import { Taxes } from "./taxes/Taxes";
import { Chat } from "./chat/Chat";
import { OrdersPage } from "./documents/Orders";
import { IntakeLine } from "./intake/IntakeLine";
import { useIntakeChanged } from "./intake/useIntakeChanged";
import { Accounts } from "./money/Accounts";
import { Bookings } from "./money/Bookings";
import { RecurringPage } from "./money/Recurring";
import { moneyPath } from "./money/describe";
import { initialNav, navFromUrl, navigate, type ViewId } from "./navigation";
import { reviewPath } from "./review/keys";
import { Review } from "./review/Review";
import { Settings, settingsPages, type SettingsPageId } from "./settings/Settings";
import { AppShell, Icon, Kbd, QuickSearch, SidebarRow, applySize, isMac, readFlag, readSize, sizeStep, useQuickSearchKey, writeLayout, type IconName, type SearchEntry } from "./ui";

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
  [
    { id: "start", label: "Start", icon: "home" },
    { id: "review", label: "Prüfen", icon: "check" },
  ],
  [
    { id: "documents", label: "Dokumente", icon: "document" },
    { id: "accounts", label: "Konten", icon: "card" },
  ],
  [
    { id: "evaluations", label: "Auswertungen", icon: "upDown" },
    { id: "taxes", label: "Steuern", icon: "scale" },
    { id: "chat", label: "Chat", icon: "mail" },
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
  // The dev server may name a place in its URL, for the check of the views (#51).
  const [nav, dispatch] = useReducer(navigate, initialNav, (start) => (import.meta.env.DEV ? navFromUrl(start, window.location.search) : start));
  const [waiting, setWaiting] = useState(0);
  const [searching, setSearching] = useState(false);
  // The focus of Prüfen (#54) hides the sidebar.
  const [focusing, setFocusing] = useState(false);
  // The sidebar folded to a rail, and the size of the view (#53), kept on the computer.
  const [rail, setRailHere] = useState(() => readFlag("rail", false));
  const setRail = useCallback((on: boolean) => {
    setRailHere(on);
    writeLayout("rail", on ? true : null);
  }, []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      if (event.key.toLowerCase() === "b") {
        event.preventDefault();
        setRailHere((on) => {
          writeLayout("rail", on ? null : true);
          return !on;
        });
        return;
      }
      const step = event.key === "+" || event.key === "=" ? "+" : event.key === "-" ? "-" : event.key === "0" ? "0" : null;
      if (!step) return;
      event.preventDefault();
      const size = sizeStep(readSize(), step);
      writeLayout("size", size === "mittel" ? null : size);
      void applySize(size);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
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
        rail={rail}
        appName={
          ready ? (
            <button
              type="button"
              aria-label={rail ? "Seitenleiste ausklappen" : "Seitenleiste einklappen"}
              title={`${rail ? "Ausklappen" : "Einklappen"} (${isMac ? "⌘" : "Strg"}+B)`}
              onClick={() => setRail(!rail)}
              className="flex size-7 items-center justify-center rounded-control text-fg-muted transition-quick hover:bg-hover hover:text-fg"
            >
              <Icon name={rail ? "chevronRight" : "back"} size={15} />
            </button>
          ) : undefined
        }
        onBack={nav.back.length > 0 ? back : undefined}
        path={
          !ready
            ? [{ label: "Finanzarchiv" }]
            : nav.view === "review"
              ? reviewPath(review.group?.label ?? null, review.question?.name ?? null, {
                  toStart: () => dispatch({ type: "set", spot: { view: "review", place: { group: null, question: null } } }),
                  toGroup: () => dispatch({ type: "set", spot: { view: "review", place: { group: review.group, question: null } } }),
                })
              : nav.view === "accounts" && accounts.recurring
                ? [{ label: "Konten", onSelect: () => dispatch({ type: "set", spot: { view: "accounts", place: { bookingsOf: undefined, text: "" } } }) }, { label: "Wiederkehrend" }]
                : nav.view === "accounts"
                ? moneyPath(accounts.bookingsOf?.name ?? null, accounts.bookingsOf !== undefined, {
                    toAccounts: () => dispatch({ type: "set", spot: { view: "accounts", place: { bookingsOf: undefined, text: "" } } }),
                  })
                : nav.view === "evaluations"
                  ? [{ label: "Auswertungen" }]
                : nav.view === "start"
                  ? [{ label: "Start" }]
                : nav.view === "taxes"
                  ? [{ label: "Steuern" }]
                : nav.view === "chat"
                  ? [{ label: "Chat" }]
                : nav.view === "settings"
                  ? [{ label: "Einstellungen", onSelect: () => openSettings("ordner") }, { label: settingsLabel }]
                  : documents.orders
                    ? [{ label: "Dokumente", onSelect: () => dispatch({ type: "set", spot: { view: "documents", place: { ...documents, orders: false } } }) }, { label: "Bestellungen" }]
                    : [{ label: "Dokumente" }]
        }
        sidebar={
          ready && !(focusing && nav.view === "review") ? (
            <div className={`flex min-h-0 flex-1 flex-col gap-4 pb-3 ${rail ? "items-center px-2" : "px-3"}`}>
              {rail ? (
                <button type="button" aria-label="Suchen" title={`Suchen (${isMac ? "⌘" : "Strg"}+K)`} onClick={openSearch} className="flex size-8 items-center justify-center rounded-row bg-field text-fg-muted shadow-[inset_0_0_0_1px_var(--line)]">
                  <Icon name="search" size={15} />
                </button>
              ) : (
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
              )}
              <nav aria-label="Ansichten" className="flex flex-1 flex-col gap-4">
                {groups.map((group) => (
                  <ul key={group[0]?.id} className="flex flex-col gap-px">
                    {group.map((entry) => (
                      <li key={entry.id}>
                        <SidebarRow
                          rail={rail}
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
                <IntakeLine rail={rail} />
                <SidebarRow rail={rail} label="Einstellungen" icon="gear" selected={nav.view === "settings"} onSelect={() => dispatch({ type: "go", view: "settings" })} />
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
        ) : nav.view === "chat" ? (
          <Chat onOpenDocument={openDocument} onOpenBookings={() => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: null, text: "" } } })} />
        ) : nav.view === "taxes" ? (
          <Taxes onOpenDocument={openDocument} onOpenBooking={() => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: null, text: "" } } })} />
        ) : nav.view === "start" ? (
          <Start
            onOpenDocument={openDocument}
            onTarget={(target) => {
              if (target.view === "review") dispatch({ type: "go", view: "review" });
              else if (target.view === "evaluations") dispatch({ type: "go", view: "evaluations" });
              else if (target.view === "orders") dispatch({ type: "jump", to: { view: "documents", place: { focus: null, orders: true } } });
              else if (target.view === "accounts") dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: undefined, text: "" } } });
              else openSettings(target.page);
            }}
          />
        ) : nav.view === "evaluations" ? (
          <Evaluations
            year={nav.places.evaluations.year}
            onYear={(year) => dispatch({ type: "set", spot: { view: "evaluations", place: { year } } })}
            onBookings={(filter) => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: null, text: "", filter } } })}
            onRecurring={() => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: undefined, text: "", recurring: true } } })}
          />
        ) : nav.view === "settings" ? (
          <Settings page={nav.places.settings.page} onPage={openSettings} version={loaded.status === "ok" ? loaded.data.version : null} />
        ) : nav.view === "review" ? (
          <main className="flex min-h-0 flex-1 flex-col items-center gap-6 overflow-y-auto px-8 py-8">
            <Review at={review} onAt={(at) => dispatch({ type: "set", spot: { view: "review", place: at } })} onAnswered={countWaiting} focusing={focusing} onFocus={setFocusing} />
          </main>
        ) : (
          <>
            {accounts.recurring ? (
              <RecurringPage onBookings={(text) => dispatch({ type: "jump", to: { view: "accounts", place: { bookingsOf: null, text } } })} />
            ) : accounts.bookingsOf === undefined ? (
              <Accounts
                onRecurring={() => dispatch({ type: "set", spot: { view: "accounts", place: { bookingsOf: undefined, text: "", recurring: true } } })}
                onOpen={(account) =>
                  dispatch({ type: "set", spot: { view: "accounts", place: { bookingsOf: account ? { id: account.id, name: account.name } : null, text: "" } } })
                }
              />
            ) : (
              <Bookings
                account={accounts.bookingsOf ?? null}
                text={accounts.text}
                filter={accounts.filter}
                onOpenDocument={openDocument}
                onClearText={() => dispatch({ type: "set", spot: { view: "accounts", place: { ...accounts, text: "", filter: undefined } } })}
                onRenamed={(name) =>
                  accounts.bookingsOf && dispatch({ type: "set", spot: { view: "accounts", place: { ...accounts, bookingsOf: { ...accounts.bookingsOf, name } } } })
                }
              />
            )}
          </>
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
