import { useEffect, useState } from "react";
import { type DocumentChecks_Serialize as DocumentChecks, commands } from "../bindings";
import { Button } from "../ui";
import { filingLine } from "./describe";

// "Warum?" and "Rückgängig" of #16: why a document was filed by itself, or why it waits.
export function Filing({ document, onChanged }: { document: number; onChanged: () => void }) {
  const [checks, setChecks] = useState<DocumentChecks | null>(null);
  const [why, setWhy] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let current = true;
    setWhy(false);
    void commands.documentChecks(document).then((result) => {
      if (current && result.status === "ok") setChecks(result.data);
    });
    return () => {
      current = false;
    };
  }, [document]);

  if (checks === null) return null;
  const failed = checks.checks.filter((check) => !check.passed);
  const passed = checks.checks.filter((check) => check.passed);

  const undo = async () => {
    setBusy(true);
    const result = await commands.undoFiling(document);
    setBusy(false);
    if (result.status === "ok") {
      setChecks(result.data);
      onChanged();
    }
  };

  return (
    <div className="flex flex-col gap-1.5 text-small">
      <p className={checks.state === "waiting" ? "font-medium" : "text-fg-muted"}>{filingLine(checks.state, checks.rule, checks.decidedAt)}</p>
      {checks.state === "waiting" && failed.length > 0 ? (
        <ul className="flex list-disc flex-col gap-0.5 pl-4">
          {failed.map((check, index) => (
            <li key={`${check.check}-${check.field ?? ""}-${index}`}>{check.detail}</li>
          ))}
        </ul>
      ) : null}
      {checks.state === "filed" ? (
        <div className="flex gap-2">
          <Button size="small" kind="plain" onClick={() => setWhy(!why)} aria-expanded={why}>
            Warum?
          </Button>
          {checks.rule !== null ? (
            <Button size="small" kind="plain" busy={busy} onClick={() => void undo()}>
              Rückgängig
            </Button>
          ) : null}
        </div>
      ) : null}
      {why ? (
        <ul className="flex list-disc flex-col gap-0.5 pl-4 text-fg-muted">
          {passed.map((check, index) => (
            <li key={`${check.check}-${check.field ?? ""}-${index}`}>{check.detail}</li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
