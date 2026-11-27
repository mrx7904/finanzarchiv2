// The root of the draft: picks the view by ?v= and the scheme by ?t=. Without ?v= it lists every view as a link.
import type { ReactNode } from "react";
import { hauptansichten } from "./Entwurf";
import { anpassen } from "./Anpassen";
import { dialoge } from "./Dialoge";
import { unterseiten } from "./Unterseiten";

const views: Record<string, () => ReactNode> = { ...hauptansichten, ...unterseiten, ...dialoge, ...anpassen };

export function Seite() {
  const query = new URLSearchParams(window.location.search);
  const theme = query.get("t");
  const view = views[query.get("v") ?? ""];
  return (
    <div data-theme={theme === "light" || theme === "dark" ? theme : undefined} className="h-full bg-canvas text-fg">
      {view ? (
        view()
      ) : (
        <ul className="columns-3 p-8 font-sans text-body">
          {Object.keys(views).map((name) => (
            <li key={name} className="py-0.5">
              <a href={`?v=${name}`} className="underline-offset-2 hover:underline">
                {name}
              </a>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
