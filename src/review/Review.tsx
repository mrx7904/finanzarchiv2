import { useCallback, useEffect, useState } from "react";
import { type ReviewAnswer, type ReviewGroup, type ReviewQuestion, commands } from "../bindings";
import { PageImage } from "../documents/PageImage";
import { kindLabel } from "../documents/describe";
import { Button, Dialog, EmptyState, List, ListRow, TextField } from "../ui";
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
  const [deleting, setDeleting] = useState(false);

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
      const action = reviewKey(event.key, { count: list.length, index, open: at.question !== null, readings: denied ? readings.length : 0 });
      if (action.kind === "move") setIndex(action.index);
      else if (action.kind === "open") {
        if (at.group) open(questions[index]);
        else if (groups[index]) onAt({ group: groups[index], question: null });
      } else if (action.kind === "back") onAt(at.question ? { group: at.group, question: null } : { group: null, question: null });
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
                <Button key={kind} size="small" busy={busy} onClick={() => send({ answer: "kind", kind })}>
                  {kindLabel(kind)}
                </Button>
              ))}
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
