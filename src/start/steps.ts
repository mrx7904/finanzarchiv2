// The next steps of "Start" (#40, FA-43), derived from the archive each time and never stored, the most pressing first:
// the local model out of reach (nothing is read without it), a folder or a mailbox with a problem (nothing comes in),
// documents that wait in Prüfen, an account with a statement missing, orders without an invoice, a savings hint.

export type StepTarget =
  | { view: "review" }
  | { view: "settings"; page: "modell" | "ordner" | "postfaecher" }
  | { view: "accounts"; account: { id: number; name: string } }
  | { view: "orders" }
  | { view: "evaluations" };

export type Step = { key: string; title: string; detail?: string; action: string; target: StepTarget };

export type StartFacts = {
  waiting: number;
  modelProblem: string | null;
  folderProblems: number;
  mailboxProblems: number;
  /** Accounts whose findings name a missing statement. */
  gaps: { id: number; name: string }[];
  ordersMissing: number;
  hints: { key: string; subject: string; line: string }[];
};

export function nextSteps(facts: StartFacts): Step[] {
  const steps: Step[] = [];
  if (facts.modelProblem) steps.push({ key: "model", title: "Das lokale Modell ist nicht erreichbar", detail: facts.modelProblem, action: "Modell ansehen", target: { view: "settings", page: "modell" } });
  if (facts.folderProblems > 0) steps.push({ key: "folders", title: facts.folderProblems === 1 ? "Ein Ordner hat ein Problem" : `${facts.folderProblems} Ordner haben ein Problem`, action: "Ordner ansehen", target: { view: "settings", page: "ordner" } });
  if (facts.mailboxProblems > 0) steps.push({ key: "mailboxes", title: facts.mailboxProblems === 1 ? "Ein Postfach hat ein Problem" : `${facts.mailboxProblems} Postfächer haben ein Problem`, action: "Postfächer ansehen", target: { view: "settings", page: "postfaecher" } });
  if (facts.waiting > 0) steps.push({ key: "review", title: facts.waiting === 1 ? "1 Frage wartet in Prüfen" : `${facts.waiting} Fragen warten in Prüfen`, action: "Prüfen", target: { view: "review" } });
  for (const account of facts.gaps) steps.push({ key: `gap:${account.id}`, title: `Im ${account.name} fehlt ein Kontoauszug`, action: "Konto ansehen", target: { view: "accounts", account } });
  if (facts.ordersMissing > 0) steps.push({ key: "orders", title: facts.ordersMissing === 1 ? "Einer Bestellung fehlt die Rechnung" : `${facts.ordersMissing} Bestellungen fehlt die Rechnung`, action: "Bestellungen", target: { view: "orders" } });
  for (const hint of facts.hints.slice(0, 2)) steps.push({ key: `hint:${hint.key}`, title: hint.subject, detail: hint.line, action: "Auswertungen", target: { view: "evaluations" } });
  return steps;
}

/** The one sentence at the top of Start. */
export function startSentence(steps: Step[]): string {
  if (steps.length === 0) return "Nichts wartet.";
  return steps.length === 1 ? "Eine Sache wartet." : `${steps.length} Dinge warten.`;
}
