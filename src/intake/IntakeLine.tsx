// The intake seen from every view (#44): one quiet line at the foot of the sidebar, shown only while something is
// read, paused or refused. A click opens the jobs above it with their stage, the pause, and what was refused with its
// reason and "Ausblenden". It replaces the box "Eingang" above Dokumente. A native popover, as the menu is: it lies
// in the top layer and closes on Escape and on a click beside it.
import { useCallback, useEffect, useId, useState } from "react";
import { type IntakeOverview_Serialize as IntakeOverview, commands } from "../bindings";
import { Button, Spinner, cn } from "../ui";
import { footLine, stageLabel } from "./describe";
import { useIntakeChanged } from "./useIntakeChanged";

export function IntakeLine({ rail = false }: { rail?: boolean }) {
  const id = useId();
  const [overview, setOverview] = useState<IntakeOverview | null>(null);
  const reload = useCallback(async () => {
    const result = await commands.intakeOverview();
    if (result.status === "ok") setOverview(result.data);
  }, []);
  useEffect(() => {
    void reload();
  }, [reload]);
  useIntakeChanged(() => void reload());

  const line = overview ? footLine(overview) : null;
  if (!overview || !line) return null;
  const reading = overview.open > 0 && !overview.paused;
  return (
    <>
      <button
        type="button"
        popoverTarget={id}
        aria-label={rail ? `Eingang, ${line}` : undefined}
        title={rail ? `Eingang, ${line}` : undefined}
        className={cn("flex h-8 items-center gap-2.5 rounded-row text-left text-fg-muted transition-quick hover:bg-hover hover:text-fg", rail ? "w-8 justify-center" : "w-full px-2")}
      >
        {reading ? (
          <Spinner size={16} />
        ) : (
          <span className="flex w-4 justify-center">
            <span className={cn("size-2 rounded-full", overview.failed.length > 0 && !overview.paused ? "bg-danger" : "bg-fg-muted")} />
          </span>
        )}
        {!rail && <span className="flex-1 truncate">{line}</span>}
      </button>
      <div id={id} popover="auto" className="fa-menu fixed inset-auto bottom-14 left-3 m-0 w-[380px] rounded-dialog bg-raised text-fg shadow-raised">
        <div className="flex flex-col">
        <div className="flex items-center justify-between px-4 pt-3.5 pb-2">
          <span className="text-heading font-semibold">Eingang</span>
          <Button size="small" onClick={() => void commands.setIntakePaused(!overview.paused).then(reload)}>
            {overview.paused ? "Fortsetzen" : "Pausieren"}
          </Button>
        </div>
        {overview.running.length > 0 && (
          <ul className="flex max-h-72 flex-col overflow-y-auto px-4 pb-2">
            {overview.running.map((job) => (
              <li key={job.id} className="flex h-9 items-center gap-2.5">
                {job.stage === "queued" || overview.paused ? (
                  <span className="flex w-3.5 shrink-0 justify-center">
                    <span className="size-1.5 rounded-full bg-fg-muted" />
                  </span>
                ) : (
                  <Spinner />
                )}
                <span className="min-w-0 flex-1 truncate">{job.name}</span>
                <span className="shrink-0 text-small text-fg-muted">{overview.paused ? "angehalten" : stageLabel(job.stage)}</span>
              </li>
            ))}
          </ul>
        )}
        {overview.failed.length > 0 && (
          <div className="flex flex-col gap-1 border-t border-line px-4 py-3">
            <span className="text-small font-medium text-fg-muted">Nicht angenommen</span>
            <ul className="flex max-h-48 flex-col gap-1 overflow-y-auto">
              {overview.failed.map((job) => (
                <li key={job.id} className="flex items-center gap-2.5">
                  <span className="flex w-3.5 shrink-0 justify-center">
                    <span className="size-2 rounded-full bg-danger" />
                  </span>
                  <span className="min-w-0 flex-1 truncate">{job.name}</span>
                  <span className="max-w-48 shrink-0 truncate text-small text-fg-muted" title={job.error ?? undefined}>
                    {job.error}
                  </span>
                </li>
              ))}
            </ul>
            <div>
              <Button size="small" kind="plain" className="-ml-2" onClick={() => void commands.dismissFailed().then(reload)}>
                Ausblenden
              </Button>
            </div>
          </div>
        )}
        </div>
      </div>
    </>
  );
}
