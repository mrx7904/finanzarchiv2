import { useEffect, useState } from "react";
import { type LoadedStatus, describeLoaded } from "./archive-status";
import { commands } from "./bindings";
import { Documents } from "./documents/Documents";

// The shell of #01 in its placeholder look, until the owner picks the design system in #07.
export function App() {
  const [loaded, setLoaded] = useState<LoadedStatus | null>(null);

  useEffect(() => {
    commands.appStatus().then(setLoaded, () =>
      setLoaded({ status: "error", error: { kind: "internal", message: "Der Kern antwortet nicht. Bitte starte Finanzarchiv neu." } }),
    );
  }, []);

  const line = loaded ? describeLoaded(loaded) : null;
  const ready = loaded?.status === "ok" && loaded.data.archive.state === "ready";
  return (
    <main className="flex min-h-screen flex-col items-center gap-6 bg-neutral-50 px-8 py-10 text-neutral-900 dark:bg-neutral-900 dark:text-neutral-100">
      <header className="flex flex-col items-center gap-1">
        <h1 className="text-2xl font-semibold">Finanzarchiv</h1>
        <p className={line?.tone === "problem" ? "text-sm text-red-600 dark:text-red-400" : "text-sm text-neutral-500"}>
          {line ? line.text : "Das Archiv wird geöffnet …"}
        </p>
        {line?.version ? <p className="text-xs text-neutral-400">Version {line.version}</p> : null}
      </header>
      {ready ? <Documents /> : null}
    </main>
  );
}
