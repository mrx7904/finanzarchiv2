import { lazy, Suspense, useEffect, useState } from "react";
import { type LoadedStatus, describeLoaded } from "./archive-status";
import { commands } from "./bindings";
import { Documents } from "./documents/Documents";
import { Intake } from "./intake/Intake";
import { useIntakeChanged } from "./intake/useIntakeChanged";
import { reviewPath } from "./review/keys";
import { Review, type ReviewAt } from "./review/Review";
import { AppShell, Button, SidebarRow } from "./ui";

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

// The shell of #01 in the look of #07, with the state of the archive from the core (#03) the intake of #05 and the documents of #04.
// Its top bar is the title bar of the window from the start (decision-app-shell.md).
export function App() {
  const hash = useHash();
  const [loaded, setLoaded] = useState<LoadedStatus | null>(null);
  const [view, setView] = useState<"documents" | "review">("documents");
  const [at, setAt] = useState<ReviewAt>({ group: null, question: null });
  const [waiting, setWaiting] = useState(0);
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

  if (Showcase && hash === "#gestaltung") {
    return (
      <Suspense fallback={null}>
        <Showcase />
      </Suspense>
    );
  }

  const line = loaded ? describeLoaded(loaded) : null;
  const ready = loaded?.status === "ok" && loaded.data.archive.state === "ready";
  return (
    <AppShell
      path={
        !ready
          ? [{ label: "Finanzarchiv" }]
          : view === "review"
            ? reviewPath(at.group?.label ?? null, at.question?.name ?? null, {
                toStart: () => setAt({ group: null, question: null }),
                toGroup: () => setAt({ group: at.group, question: null }),
              })
            : [{ label: "Dokumente" }]
      }
      sidebar={
        ready ? (
          <nav aria-label="Ansichten" className="flex flex-col gap-px px-3 pb-3">
            <SidebarRow label="Dokumente" icon="document" selected={view === "documents"} onSelect={() => setView("documents")} />
            <SidebarRow
              label="Prüfen"
              icon="check"
              count={waiting}
              selected={view === "review"}
              onSelect={() => {
                setView("review");
                setAt({ group: null, question: null });
              }}
            />
          </nav>
        ) : undefined
      }
      actions={
        Showcase && (
          <Button kind="plain" onClick={() => (window.location.hash = "gestaltung")}>
            Gestaltung ansehen
          </Button>
        )
      }
    >
      <main className="flex min-h-0 flex-1 flex-col items-center gap-6 overflow-y-auto px-8 py-8">
        <header className="flex flex-col items-center gap-1">
          <p className={line?.tone === "problem" ? "text-danger" : "text-fg-muted"}>{line ? line.text : "Das Archiv wird geöffnet …"}</p>
          {line?.version ? <p className="text-small text-fg-muted">Version {line.version}</p> : null}
        </header>
        {ready && view === "documents" ? <Intake /> : null}
        {ready && view === "documents" ? <Documents /> : null}
        {ready && view === "review" ? <Review at={at} onAt={setAt} onAnswered={countWaiting} /> : null}
      </main>
    </AppShell>
  );
}
