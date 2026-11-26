import { useCallback, useEffect, useState } from "react";
import { type ReviewAnswer, type ReviewGroup, type ReviewQuestion, commands } from "../bindings";
import { PageImage } from "../documents/PageImage";
import { kindLabel } from "../documents/describe";
import { Button, EmptyState, List, ListRow, TextField } from "../ui";
import { reviewKey } from "./keys";

/** Where "Prüfen" stands: the group chosen and the question open, both lifted into the app for the path. */
export type ReviewAt = { group: ReviewGroup | null; question: ReviewQuestion | null };

/** The marked line as CSS: left, top, width and height as fractions of the page. */
function markStyle(mark: (number | null)[]) {
  const share = (at: number) => `${(mark[at] ?? 0) * 100}%`;
  return { left: share(0), top: share(1), width: share(2), height: share(3) };
}

const KINDS = ["invoice", "receipt", "payslip", "statement", "other"];

// "Prüfen" of #17: what waits, grouped by its first reason, as questions with the page beside.
export function Review({ at, onAt, onAnswered }: { at: ReviewAt; onAt: (at: ReviewAt) => void; onAnswered: () => void }) {
  const [groups, setGroups] = useState<ReviewGroup[]>([]);
  const [questions, setQuestions] = useState<ReviewQuestion[]>([]);
  const [index, setIndex] = useState(0);
  const [denied, setDenied] = useState(false);
  const [typed, setTyped] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const loadGroups = useCallback(async () => {
    const result = await commands.reviewGroups();
    if (result.status === "ok") setGroups(result.data);
  }, []);
  const loadQuestions = useCallback(async (key: string) => {
    const result = await commands.reviewQuestions(key);
    if (result.status === "ok") setQuestions(result.data);
    return result.status === "ok" ? result.data : [];
  }, []);

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

  const send = async (answer: ReviewAnswer) => {
    const question = at.question;
    if (!question || !at.group) return;
    setBusy(true);
    const result = await commands.answerQuestion(question.document, question.key, answer);
    setBusy(false);
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
      const action = reviewKey(event.key, { count: list.length, index, open: at.question !== null, readings: denied ? readings.length : 0 });
      if (action.kind === "move") setIndex(action.index);
      else if (action.kind === "open") {
        if (at.group) open(questions[index]);
        else if (groups[index]) onAt({ group: groups[index], question: null });
      } else if (action.kind === "back") onAt(at.question ? { group: at.group, question: null } : { group: null, question: null });
      else if (action.kind === "pick") void send({ answer: "reading", index: action.reading });
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
    const yesNo = !isKind;
    return (
      <section className="grid w-full max-w-5xl grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-start gap-4">
        <div className="relative overflow-hidden rounded-panel shadow-panel">
          <PageImage document={question.document} page={question.page} width={1280} label={`Seite ${question.page} von ${question.name}`} />
          {first?.mark ? (
            <span
              aria-hidden
              className="absolute rounded-[2px] bg-accent/20 shadow-[0_0_0_2px_var(--accent)]"
              style={markStyle(first.mark)}
            />
          ) : null}
        </div>
        <div className="flex flex-col gap-3 rounded-panel bg-surface p-4 shadow-panel">
          <p className="text-body font-medium">{question.text}</p>
          {first ? <p className="text-small text-fg-muted">Gelesen von {question.readings.map((reading) => `${reading.reader}: ${reading.shown}`).join(", ")}</p> : null}
          {isKind ? (
            <div className="flex flex-wrap gap-2">
              {KINDS.map((kind) => (
                <Button key={kind} size="small" busy={busy} onClick={() => void send({ answer: "kind", kind })}>
                  {kindLabel(kind)}
                </Button>
              ))}
            </div>
          ) : null}
          {yesNo ? (
            <div className="flex gap-2">
              <Button kind="primary" busy={busy} onClick={() => void send({ answer: "yes" })}>
                Ja
              </Button>
              <Button onClick={() => (question.field ? setDenied(true) : void send({ answer: "no" }))}>Nein</Button>
            </div>
          ) : null}
          {denied ? (
            <div className="flex flex-col gap-2">
              {question.readings.length > 1 ? (
                <div className="grid grid-cols-2 gap-2">
                  {question.readings.map((reading, number) => (
                    <Button key={`${reading.reader}-${reading.shown}`} onClick={() => void send({ answer: "reading", index: number })}>
                      {`${number + 1} · ${reading.reader}: ${reading.shown}`}
                    </Button>
                  ))}
                </div>
              ) : null}
              <form
                className="flex items-end gap-2"
                onSubmit={(event) => {
                  event.preventDefault();
                  void send({ answer: "typed", text: typed });
                }}
              >
                <TextField label="Richtiger Wert" value={typed} onChange={setTyped} error={error ?? undefined} className="flex-1" />
                <Button type="submit" busy={busy}>
                  Übernehmen
                </Button>
              </form>
            </div>
          ) : error ? (
            <p className="text-small text-danger">{error}</p>
          ) : null}
        </div>
      </section>
    );
  }

  if (at.group) {
    return (
      <section className="flex w-full max-w-3xl flex-col gap-3">
        <List label={at.group.label}>
          {questions.map((question, number) => (
            <ListRow key={question.document} title={question.name} detail={question.text} selected={number === index} onSelect={() => open(question)} />
          ))}
        </List>
      </section>
    );
  }

  return (
    <section className="flex w-full max-w-3xl flex-col gap-3">
      {groups.length === 0 ? (
        <EmptyState icon="check" title="Nichts zu prüfen" hint="Alles, was hereinkam, ist abgelegt." />
      ) : (
        <List label="Gründe">
          {groups.map((group, number) => (
            <ListRow key={group.key} title={group.label} figure={String(group.count)} selected={number === index} onSelect={() => onAt({ group, question: null })} />
          ))}
        </List>
      )}
    </section>
  );
}
