import { lazy, Suspense, useEffect, useState } from "react";
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

// The empty shell of #01, in the look of #07 until the first view arrives.
export function App() {
  const hash = useHash();
  if (Showcase && hash === "#gestaltung") {
    return (
      <Suspense fallback={null}>
        <Showcase />
      </Suspense>
    );
  }
  return (
    <main className="flex h-screen flex-col items-center justify-center gap-2 bg-canvas font-sans text-fg">
      <h1 className="font-title text-title">Finanzarchiv</h1>
      <p className="text-fg-muted">Das Fundament steht. Hier entsteht das Archiv.</p>
      {Showcase && (
        <Button className="mt-4" onClick={() => (window.location.hash = "gestaltung")}>
          Gestaltung ansehen
        </Button>
      )}
    </main>
  );
}
