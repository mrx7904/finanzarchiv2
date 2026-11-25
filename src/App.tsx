import { useEffect, useState } from "react";
import { type LoadedStatus, describeLoaded } from "./archive-status";
import { commands } from "./bindings";

// The empty shell of #01. Its look is a placeholder until the owner picks the design system in #07.
export function App() {
  const [loaded, setLoaded] = useState<LoadedStatus | null>(null);

  useEffect(() => {
    commands.appStatus().then(setLoaded, () =>
      setLoaded({ status: "error", error: { kind: "internal", message: "Der Kern antwortet nicht. Bitte starte Finanzarchiv neu." } }),
    );
  }, []);

  const line = loaded ? describeLoaded(loaded) : null;
  return (
    <main className="flex h-screen flex-col items-center justify-center gap-2 bg-neutral-50 text-neutral-900 dark:bg-neutral-900 dark:text-neutral-100">
      <h1 className="text-2xl font-semibold">Finanzarchiv</h1>
      <p className={line?.tone === "problem" ? "text-sm text-red-600 dark:text-red-400" : "text-sm text-neutral-500"}>
        {line ? line.text : "Das Archiv wird geöffnet …"}
      </p>
      {line?.version ? <p className="text-xs text-neutral-400">Version {line.version}</p> : null}
    </main>
  );
}
