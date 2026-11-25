import { useCallback, useEffect, useState } from "react";
import { type IntakeOverview_Serialize as IntakeOverview, commands } from "../bindings";
import { Button } from "../ui";
import { intakeLine, stageLabel } from "./describe";
import { useIntakeChanged } from "./useIntakeChanged";

// "Eingang" of #05: what is in the queue, what was refused and why, and the pause.
export function Intake() {
  const [overview, setOverview] = useState<IntakeOverview | null>(null);

  const reload = useCallback(async () => {
    const result = await commands.intakeOverview();
    if (result.status === "ok") setOverview(result.data);
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);
  useIntakeChanged(() => void reload());

  if (!overview) return null;
  const quiet = overview.open === 0 && overview.failed.length === 0;
  return (
    <section aria-label="Eingang" className="flex w-full max-w-3xl flex-col gap-2 rounded-panel bg-surface p-4 shadow-panel">
      <div className="flex items-center justify-between gap-3">
        <p className="text-body">
          <span className="font-medium">Eingang</span> <span className="text-fg-muted">{intakeLine(overview)}</span>
        </p>
        {quiet && !overview.paused ? null : (
          <Button size="small" onClick={() => void commands.setIntakePaused(!overview.paused)}>
            {overview.paused ? "Fortsetzen" : "Pausieren"}
          </Button>
        )}
      </div>
      {overview.running.length > 0 ? (
        <ul className="flex flex-col gap-0.5 text-small">
          {overview.running.map((job) => (
            <li key={job.id} className="flex flex-col">
              <span className="flex justify-between gap-3">
                <span className="truncate">{job.name}</span>
                <span className="shrink-0 text-fg-muted">{overview.paused ? "angehalten" : stageLabel(job.stage)}</span>
              </span>
              {job.error ? <span className="text-fg-muted">{job.error}</span> : null}
            </li>
          ))}
        </ul>
      ) : null}
      {overview.failed.length > 0 ? (
        <div className="flex flex-col gap-1">
          <ul className="flex flex-col gap-0.5 text-small">
            {overview.failed.map((job) => (
              <li key={job.id}>
                <span className="font-medium">{job.name}</span> <span className="text-danger">{job.error}</span>
              </li>
            ))}
          </ul>
          <div>
            <Button size="small" kind="plain" onClick={() => void commands.dismissFailed()}>
              Ausblenden
            </Button>
          </div>
        </div>
      ) : null}
    </section>
  );
}
