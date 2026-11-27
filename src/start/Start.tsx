import { useEffect, useState } from "react";
import { commands } from "../bindings";
import { Figure, euros } from "../evaluations/charts";
import { lastMonth, periodWords } from "../evaluations/derive";
import { useIntakeChanged } from "../intake/useIntakeChanged";
import { Button, Icon, LoadingRows } from "../ui";
import { type Step, type StepTarget, nextSteps, startSentence } from "./steps";

// "Start" (#40, FA-43): one sentence on what waits, then "Als Nächstes", the most pressing first, each with one button
// that does it, and beside them the key figures of the running year, each opening Auswertungen. The steps are derived
// from the archive every time and never stored.
export function Start({ onTarget, onOpenDocument }: { onTarget: (target: StepTarget) => void; onOpenDocument: (document: number) => void }) {
  const [filed, setFiled] = useState<{ id: number; name: string }[]>([]);
  const [steps, setSteps] = useState<Step[] | null>(null);
  const [figures, setFigures] = useState<{ year: number; income: number; spending: number; period: string } | null>(null);

  const load = async () => {
    const [groups, model, folders, mailboxes, accounts, orders, hints] = await Promise.all([
      commands.reviewGroups(),
      commands.modelStatus(),
      commands.listFolders(),
      commands.listMailboxes(),
      commands.listAccounts(),
      commands.listOrders(),
      commands.listHints(),
    ]);
    setSteps(
      nextSteps({
        waiting: groups.status === "ok" ? groups.data.reduce((sum, group) => sum + group.count, 0) : 0,
        modelProblem: model.status === "ok" && !model.data.installed ? (model.data.problem ?? "nicht erreichbar") : null,
        folderProblems: folders.status === "ok" ? folders.data.filter((folder) => folder.problem).length : 0,
        mailboxProblems: mailboxes.status === "ok" ? mailboxes.data.filter((mailbox) => mailbox.problem).length : 0,
        gaps: accounts.status === "ok" ? accounts.data.filter((account) => account.findings.some((finding) => finding.includes("fehlt"))).map((account) => ({ id: account.id, name: account.name })) : [],
        ordersMissing: orders.status === "ok" ? orders.data.filter((order) => order.invoice === null && !order.noInvoice).length : 0,
        hints: hints.status === "ok" ? hints.data : [],
      }),
    );
    const documents = await commands.listDocuments();
    if (documents.status === "ok") {
      const week = Date.now() - 7 * 24 * 60 * 60 * 1000;
      setFiled(documents.data.filter((row) => row.state === "filed" && Date.parse(row.importedAt) >= week).map((row) => ({ id: row.id, name: row.name })));
    }
    const years = await commands.evaluationYears();
    const year = years.status === "ok" ? years.data[0] : undefined;
    if (year === undefined) return;
    const row = await commands.evaluationYear(year);
    if (row.status === "ok") setFigures({ year, income: row.data.incomeTotal.cents, spending: row.data.spendingTotal.cents, period: periodWords(lastMonth(row.data.income, row.data.spending)) });
  };
  useEffect(() => {
    void load();
  }, []);
  useIntakeChanged(() => void load());

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <div className="mx-auto grid w-full max-w-[1120px] grid-cols-[minmax(0,1fr)_300px] gap-8 px-6 pt-6 pb-10 max-[1300px]:grid-cols-1">
        <section aria-label="Als Nächstes" className="flex min-w-0 flex-col gap-4">
          <p className="text-[24px] leading-tight font-semibold tracking-tight">{steps ? startSentence(steps) : " "}</p>
          {steps === null ? (
            <LoadingRows loading rows={4} />
          ) : (
            steps.length > 0 && (
              <div className="flex flex-col gap-2">
                <span className="text-small font-medium text-fg-muted">Als Nächstes</span>
                <ul className="flex flex-col rounded-panel bg-surface shadow-panel">
                  {steps.map((step) => (
                    <li key={step.key} className="flex min-h-14 items-center gap-3 border-b border-line px-4 py-2.5 last:border-b-0">
                      <span className="flex min-w-0 flex-1 flex-col">
                        <span className="font-medium">{step.title}</span>
                        {step.detail && <span className="truncate text-small text-fg-muted">{step.detail}</span>}
                      </span>
                      <Button size="small" onClick={() => onTarget(step.target)}>
                        {step.action}
                        <Icon name="chevronRight" size={14} />
                      </Button>
                    </li>
                  ))}
                </ul>
              </div>
            )
          )}
          {filed.length > 0 && (
            <div className="flex flex-col gap-2">
              <span className="text-small font-medium text-fg-muted">In den letzten 7 Tagen abgelegt · {filed.length}</span>
              <ul className="flex flex-col rounded-panel bg-surface shadow-panel">
                {filed.slice(0, 5).map((document) => (
                  <li key={document.id} className="border-b border-line last:border-b-0">
                    <button type="button" onClick={() => onOpenDocument(document.id)} className="flex h-10 w-full items-center gap-2.5 px-4 text-left transition-quick hover:bg-hover">
                      <Icon name="document" size={16} className="text-fg-muted" />
                      <span className="truncate">{document.name}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </section>
        {figures && (
          <aside aria-label={`Das Jahr ${figures.year}`} className="flex flex-col gap-4">
            <span className="text-small font-medium text-fg-muted">
              {figures.year}
              {figures.period ? `, ${figures.period}` : ""}
            </span>
            {[
              ["Einnahmen", euros(figures.income, true)],
              ["Ausgaben", euros(figures.spending)],
              ["Saldo", euros(figures.income - figures.spending, true)],
            ].map(([label, value]) => (
              <button key={label} type="button" onClick={() => onTarget({ view: "evaluations" })} className="-mx-2 rounded-row px-2 py-1 text-left transition-quick hover:bg-hover">
                <Figure label={label ?? ""} value={value ?? ""} />
              </button>
            ))}
          </aside>
        )}
      </div>
    </div>
  );
}
