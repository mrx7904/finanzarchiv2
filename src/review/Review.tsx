import { useCallback, useEffect, useState } from "react";
import { type ReviewAnswer, type ReviewGroup, type ReviewQuestion, commands } from "../bindings";
import { PageImage } from "../documents/PageImage";
import { PageView } from "../documents/PageView";
import { kindLabel } from "../documents/describe";
import { BarActions, Button, Dialog, EmptyState, List, ListRow, TextField } from "../ui";
import { SEQUENCE, inSequence, questionLine, reviewKey } from "./keys";

/** Where "Prüfen" stands: the group chosen and the question open, both lifted into the app for the path. */
export type ReviewAt = { group: ReviewGroup | null; question: ReviewQuestion | null };

/** The marked line as CSS: left, top, width and height as fractions of the page. */
function markStyle(mark: (number | null)[]) {
  const share = (at: number) => `${(mark[at] ?? 0) * 100}%`;
  return { left: share(0), top: share(1), width: share(2), height: share(3) };
}

const KINDS = ["invoice", "receipt", "payslip", "statement", "other"];

// "Prüfen" of #17, as designed in #47 (knowledge/decision-views.md): the reasons as an overview with "Der Reihe nach
// prüfen" in the top bar, the documents of a reason with the page of the chosen one, and the question with its page at
// the left and the question as a large sentence at the right. Every answer of #17 is kept, and so are its keys.
type ReviewProps = {
  at: ReviewAt;
  onAt: (at: ReviewAt) => void;
  onAnswered: () => void;
  /** The focus of #54: the sidebar hidden, the page, the question and the progress left. */
  focusing?: boolean;
  onFocus?: (on: boolean) => void;
};

export function Review({ at, onAt, onAnswered, focusing = false, onFocus }: ReviewProps) {
  // The focus ends by itself when no question is open any more.
  useEffect(() => {
    if (!at.question && focusing) onFocus?.(false);
  }, [at.question, focusing, onFocus]);
  const [groups, setGroups] = useState<ReviewGroup[]>([]);
  const [questions, setQuestions] = useState<ReviewQuestion[]>([]);
  const [index, setIndex] = useState(0);
  const [denied, setDenied] = useState(false);
  const [typed, setTyped] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const loadGroups = useCallback(async () => {
    const result = await commands.reviewGroups();
    if (result.status === "ok") setGroups(result.data);
  }, []);
  const loadQuestions = useCallback(async (key: string) => {
    if (key === SEQUENCE) {
      // "Der Reihe nach prüfen": every question of every reason, each document once.
      const listed = await commands.reviewGroups();
      if (listed.status !== "ok") return [];
      const byKey: Record<string, ReviewQuestion[]> = {};
      for (const group of listed.data) {
        const result = await commands.reviewQuestions(group.key);
        if (result.status === "ok") byKey[group.key] = result.data;
      }
      const all = inSequence(listed.data, byKey);
      setQuestions(all);
      return all;
    }
    const result = await commands.reviewQuestions(key);
    if (result.status === "ok") setQuestions(result.data);
    return result.status === "ok" ? result.data : [];
  }, []);
  const [filed, setFiled] = useState<number | null>(null);
  useEffect(() => {
    if (groups.length > 0) return;
    void commands.listDocuments().then((result) => {
      if (result.status === "ok") setFiled(result.data.filter((row) => row.state === "filed").length);
    });
  }, [groups.length]);
  const startSequence = async () => {
    const all = await loadQuestions(SEQUENCE);
    const sequence: ReviewGroup = { key: SEQUENCE, label: "Der Reihe nach", count: all.length };
    onAt({ group: sequence, question: all[0] ?? null });
  };

  useEffect(() => {
    void loadGroups();
  }, [loadGroups]);
  useEffect(() => {
    setIndex(0);
    if (at.group) void loadQuestions(at.group.key);
  }, [at.group, loadQuestions]);
  useEffect(() => {
    setDenied(false);
    setTyped("");
    setError(null);
  }, [at.question]);

  const open = (question: ReviewQuestion | undefined) => {
    if (question) onAt({ group: at.group, question });
  };

  const send = (answer: ReviewAnswer) => {
    const question = at.question;
    if (question) void settle(() => commands.answerQuestion(question.document, question.key, answer));
  };

  /** Runs an answer and moves on to the next document of the group, or back to the groups. */
  const settle = async (run: () => Promise<{ status: "ok" } | { status: "error"; error: { message: string } }>) => {
    const question = at.question;
    if (!question || !at.group) return;
    setBusy(true);
    const result = await run();
    setBusy(false);
    setDeleting(false);
    if (result.status === "error") {
      setError(result.error.message);
      return;
    }
    onAnswered();
    void loadGroups();
    const left = await loadQuestions(at.group.key);
    const next = left.find((other) => other.document !== question.document) ?? null;
    onAt({ group: left.length > 0 ? at.group : null, question: next });
  };

  const readings = at.question?.readings ?? [];
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement) return;
      const list = at.group ? questions : groups;
      const action = reviewKey(event.key, { count: list.length, index, open: at.question !== null, readings: denied || at.question?.key === "link:choose" ? readings.length : 0 });
      if (action.kind === "move") setIndex(action.index);
      else if (action.kind === "open") {
        if (at.group) open(questions[index]);
        else if (groups[index]) onAt({ group: groups[index], question: null });
      } else if (action.kind === "back") {
        // Esc ends the focus first, then goes one level back.
        if (focusing) onFocus?.(false);
        else onAt(at.question ? { group: at.group, question: null } : { group: null, question: null });
      } else if (event.key === "f" && at.question && !(event.target instanceof HTMLTextAreaElement)) onFocus?.(!focusing);
      else if (action.kind === "pick") send({ answer: "reading", index: action.reading });
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (at.question) {
    const question = at.question;
    const first = question.readings[0];
    const isKind = question.key === "check:kind";
    const isDuplicate = question.key === "check:duplicate";
    const isUndone = question.key === "check:undone";
    const isLink = question.key === "link:choose";
    const at_ = questions.findIndex((other) => other.document === question.document && other.key === question.key);
    const nextQuestion = questions.find((other, number) => number > at_ && other.document !== question.document) ?? null;
    return (
      <>
      <BarActions>
        {at_ >= 0 && <span className="text-small text-fg-muted tabular-nums">{questionLine(at_, questions.length)}</span>}
        <Button kind="plain" disabled={!nextQuestion} onClick={() => open(nextQuestion ?? undefined)}>
          Überspringen
        </Button>
        <Button kind="plain" aria-pressed={focusing} title="Fokus (F)" onClick={() => onFocus?.(!focusing)}>
          {focusing ? "Fokus beenden" : "Fokus"}
        </Button>
      </BarActions>
      <section className="grid min-h-0 w-full flex-1 grid-cols-[minmax(0,1fr)_minmax(320px,440px)] gap-4 px-6 pt-4 pb-6">
        <div className="flex min-h-0 flex-col rounded-panel bg-surface p-3 shadow-panel">
          <PageView
            key={`${question.document}-${question.key}`}
            document={question.document}
            name={question.name}
            pages={Math.max(question.page, 1)}
            page={question.page}
            startZoom={100}
            overlay={first?.mark ? <span aria-hidden className="absolute rounded-[2px] bg-accent/20 shadow-[0_0_0_2px_var(--accent)]" style={markStyle(first.mark)} /> : undefined}
          />
        </div>
        <div className="flex min-h-0 flex-col gap-4 overflow-y-auto rounded-panel bg-surface p-5 shadow-panel">
          <span className="truncate text-small text-fg-muted">{question.name}</span>
          <p className="text-[22px] leading-snug font-semibold">{question.text}</p>
          {first && !isLink ? <p className="text-small text-fg-muted">Gelesen von {question.readings.map((reading) => `${reading.reader}: ${reading.shown}`).join(", ")}</p> : null}
          {isKind ? (
            <div className="flex flex-wrap gap-2">
              {KINDS.map((kind) => (
                <Button key={kind} size="small" busy={busy} onClick={() => send({ answer: "kind", kind })}>
                  {kindLabel(kind)}
                </Button>
              ))}
            </div>
          ) : isLink ? (
            <div className="flex flex-col gap-2">
              {question.readings.map((reading, number) => (
                <Button key={reading.shown} busy={busy} onClick={() => send({ answer: "reading", index: number })}>
                  {`${number + 1} · ${reading.shown}`}
                </Button>
              ))}
              <div>
                <Button size="small" kind="plain" busy={busy} onClick={() => send({ answer: "no" })}>
                  Keine davon
                </Button>
              </div>
            </div>
          ) : isDuplicate ? (
            <div className="flex gap-2">
              <Button kind="primary" busy={busy} onClick={() => send({ answer: "yes" })}>
                Ja, eigenes Dokument
              </Button>
              <Button kind="danger" onClick={() => setDeleting(true)}>
                Nein, Doppel löschen …
              </Button>
            </div>
          ) : isUndone ? (
            <div className="flex gap-2">
              <Button kind="primary" busy={busy} onClick={() => send({ answer: "yes" })}>
                Ja, ablegen
              </Button>
              <Button busy={busy} onClick={() => send({ answer: "no" })}>
                Nein, zurückhalten
              </Button>
            </div>
          ) : !denied ? (
            <div className="flex gap-2">
              <Button kind="primary" busy={busy} onClick={() => send({ answer: "yes" })}>
                Ja
              </Button>
              <Button onClick={() => setDenied(true)}>Nein</Button>
            </div>
          ) : (
            <div className="flex flex-col gap-2">
              <p className="text-body font-medium">Welcher Wert stimmt?</p>
              {question.readings.length > 1 ? (
                <div className="grid grid-cols-2 gap-2">
                  {question.readings.map((reading, number) => (
                    <Button key={`${reading.reader}-${reading.shown}`} busy={busy} onClick={() => send({ answer: "reading", index: number })}>
                      {`${number + 1} · ${reading.shown} (${reading.reader})`}
                    </Button>
                  ))}
                </div>
              ) : null}
              <form
                className="flex items-end gap-2"
                onSubmit={(event) => {
                  event.preventDefault();
                  send({ answer: "typed", text: typed });
                }}
              >
                <TextField label={question.readings.length > 1 ? "Oder ein anderer Wert" : "Richtiger Wert"} value={typed} onChange={setTyped} error={error ?? undefined} className="flex-1" />
                <Button type="submit" busy={busy}>
                  Übernehmen
                </Button>
              </form>
              <div>
                <Button size="small" kind="plain" onClick={() => setDenied(false)}>
                  Zurück zur Frage
                </Button>
              </div>
            </div>
          )}
          {error && !denied ? <p className="text-small text-danger">{error}</p> : null}
          <Dialog
            open={deleting}
            onClose={() => setDeleting(false)}
            title="Doppel löschen?"
            description={`„${question.name}“ wird aus dem Archiv entfernt. Die Datei kommt in den Ordner „Papierkorb“ neben dem Archiv, bis du ihn leerst.`}
            actions={
              <>
                <Button kind="plain" onClick={() => setDeleting(false)}>
                  Abbrechen
                </Button>
                <Button kind="danger" busy={busy} onClick={() => void settle(() => commands.deleteDuplicate(question.document))}>
                  Löschen
                </Button>
              </>
            }
          />
          {nextQuestion && <span className="mt-auto border-t border-line pt-3 text-small text-fg-muted">Als Nächstes: {nextQuestion.name}</span>}
        </div>
      </section>
      </>
    );
  }

  if (at.group) {
    const chosen = questions[index] ?? null;
    return (
      <section className="grid min-h-0 w-full flex-1 grid-cols-[minmax(320px,440px)_minmax(0,1fr)] gap-4 px-6 pt-4 pb-6">
        <div className="min-h-0 overflow-y-auto rounded-panel bg-surface shadow-panel">
          <List label={at.group.label}>
            {questions.map((question, number) => (
              <ListRow key={`${question.document}-${question.key}`} title={question.name} detail={question.text} selected={number === index} onSelect={() => (number === index ? open(question) : setIndex(number))} />
            ))}
          </List>
        </div>
        <div className="flex min-h-0 items-center justify-center overflow-hidden rounded-panel bg-surface p-4 shadow-panel">
          {chosen ? <PageImage key={chosen.document} document={chosen.document} page={chosen.page} width={900} label={`Seite ${chosen.page} von ${chosen.name}`} fit /> : null}
        </div>
      </section>
    );
  }

  const waiting = groups.reduce((sum, group) => sum + group.count, 0);
  return (
    <>
      <BarActions>
        <Button kind="primary" disabled={groups.length === 0} onClick={() => void startSequence()}>
          Der Reihe nach prüfen
        </Button>
      </BarActions>
      <section className="mx-auto flex w-full max-w-[1120px] flex-col gap-4 overflow-y-auto px-6 pt-6 pb-8">
        {groups.length === 0 ? (
          <div className="flex flex-1 items-center justify-center py-16">
            <EmptyState icon="check" title="Nichts zu prüfen" hint={filed !== null ? `${filed.toLocaleString("de-DE")} Dokumente sind abgelegt.` : ""} />
          </div>
        ) : (
          <>
            <p className="text-heading font-semibold">{waiting === 1 ? "1 Frage wartet" : `${waiting} Fragen warten`}</p>
            <div className="rounded-panel bg-surface shadow-panel">
              <List label="Gründe">
                {groups.map((group, number) => (
                  <ListRow key={group.key} title={group.label} figure={String(group.count)} selected={number === index} onSelect={() => onAt({ group, question: null })} />
                ))}
              </List>
            </div>
          </>
        )}
      </section>
    </>
  );
}
