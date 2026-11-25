import { lazy, Suspense, useEffect, useState } from "react";
import { type LoadedStatus, describeLoaded } from "./archive-status";
import { commands } from "./bindings";
import { Button } from "./ui";

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

// The empty shell of #01 in the look of #07, with the state of the archive from the core (#03).
export function App() {
  const hash = useHash();
  const [loaded, setLoaded] = useState<LoadedStatus | null>(null);

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
  return (
    <main className="flex h-screen flex-col items-center justify-center gap-2 bg-canvas font-sans text-fg">
      <h1 className="font-title text-title">Finanzarchiv</h1>
      <p className={line?.tone === "problem" ? "text-danger" : "text-fg-muted"}>{line ? line.text : "Das Archiv wird geöffnet …"}</p>
      {line?.version ? <p className="text-small text-fg-muted">Version {line.version}</p> : null}
      {Showcase && (
        <Button className="mt-4" onClick={() => (window.location.hash = "gestaltung")}>
          Gestaltung ansehen
        </Button>
      )}
    </main>
  );
}
