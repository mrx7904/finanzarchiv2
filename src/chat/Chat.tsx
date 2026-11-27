import { type FormEvent, useCallback, useEffect, useRef, useState } from "react";
import { type ChatMessageRow, type ChatRow, commands } from "../bindings";
import { BarActions, Button, Dialog, Icon, List, ListRow, Spinner, cn, columnStyle, useColumn } from "../ui";

// "Chat" (#42, knowledge/decision-chat.md) as a list with detail: the chats at the left, the chosen one at the right
// with the question field at its foot. The model looks up through fixed lookups, code checks every amount, and the
// sources under an answer are the documents and bookings the lookups returned; each opens in its own view.
export function Chat({ onOpenDocument, onOpenBookings }: { onOpenDocument: (document: number) => void; onOpenBookings: () => void }) {
  const [chats, setChats] = useState<ChatRow[]>([]);
  const [chosen, setChosen] = useState<number | null>(null);
  const [messages, setMessages] = useState<ChatMessageRow[]>([]);
  const [question, setQuestion] = useState("");
  const [asking, setAsking] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const end = useRef<HTMLDivElement>(null);
  const [width, divider] = useColumn({ key: "chat.list", initial: 280, least: 220, most: 440, side: "left", label: "Breite der Chats" });


  const loadChats = useCallback(async () => {
    const result = await commands.listChats();
    if (result.status === "ok") setChats(result.data);
  }, []);
  useEffect(() => {
    void loadChats();
  }, [loadChats]);
  useEffect(() => {
    if (chosen === null) {
      setMessages([]);
      return;
    }
    void commands.chatMessages(chosen).then((result) => {
      if (result.status === "ok") setMessages(result.data);
    });
  }, [chosen]);
  useEffect(() => {
    end.current?.scrollIntoView({ block: "end" });
  }, [messages, asking]);

  const ask = async (event: FormEvent) => {
    event.preventDefault();
    const text = question.trim();
    if (!text || asking) return;
    setAsking(text);
    setQuestion("");
    setError(null);
    const result = await commands.askChat(chosen, text);
    setAsking(null);
    if (result.status !== "ok") {
      setError(result.error.message);
      setQuestion(text);
      return;
    }
    if (chosen !== result.data.chat) setChosen(result.data.chat);
    else {
      const listed = await commands.chatMessages(result.data.chat);
      if (listed.status === "ok") setMessages(listed.data);
    }
    void loadChats();
  };

  return (
    <>
      <BarActions>
        {chosen !== null && <Button kind="plain" icon="trash" label="Chat löschen …" onClick={() => setDeleting(true)} />}
        <Button kind="primary" icon="plus" onClick={() => setChosen(null)}>
          Neuer Chat
        </Button>
      </BarActions>
      <div style={columnStyle("--list", width)} className="grid min-h-0 flex-1 grid-cols-[var(--list)_minmax(0,1fr)] gap-4 px-6 pt-4 pb-6">
        <section aria-label="Chats" className="relative min-h-0 overflow-y-auto rounded-panel bg-surface shadow-panel">
          {divider}
          {chats.length === 0 ? (
            <p className="p-4 text-small text-fg-muted">Noch kein Chat</p>
          ) : (
            <List label="Chats">
              {chats.map((chat) => (
                <ListRow key={chat.id} title={chat.title} detail={new Date(chat.createdAt).toLocaleDateString("de-DE")} selected={chat.id === chosen} onSelect={() => setChosen(chat.id)} />
              ))}
            </List>
          )}
        </section>
        <section aria-label="Unterhaltung" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
          <div className="min-h-0 flex-1 overflow-y-auto">
            <div className="mx-auto flex max-w-3xl flex-col gap-5 px-6 py-6">
              {messages.length === 0 && !asking && <p className="text-[22px] leading-snug font-semibold">Was möchtest du über dein Archiv wissen?</p>}
              {messages.map((message) => (message.role === "owner" ? <Question key={message.id} text={message.text} /> : <Answer key={message.id} message={message} onOpenDocument={onOpenDocument} onOpenBookings={onOpenBookings} />))}
              {asking && (
                <>
                  <Question text={asking} />
                  <p role="status" className="inline-flex items-center gap-2 text-fg-muted">
                    <Spinner />
                    Sieht in deinem Archiv nach …
                  </p>
                </>
              )}
              {error && <p className="text-small text-danger">{error}</p>}
              <div ref={end} />
            </div>
          </div>
          <form onSubmit={(event) => void ask(event)} className="flex items-end gap-2 border-t border-line p-3">
            <textarea
              aria-label="Frage"
              rows={1}
              value={question}
              onChange={(event) => setQuestion(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  event.currentTarget.form?.requestSubmit();
                }
              }}
              placeholder="Frage an dein Archiv"
              className="max-h-40 min-h-(--control-height) flex-1 resize-none rounded-control bg-field px-3 py-2 text-body text-fg shadow-[inset_0_0_0_1px_var(--line)] outline-none field-sizing-content focus:shadow-[inset_0_0_0_2px_var(--fg)]"
            />
            <Button type="submit" kind="primary" icon="upload" label="Fragen" busy={asking !== null} disabled={!question.trim()} />
          </form>
        </section>
      </div>
      <Dialog
        open={deleting}
        onClose={() => setDeleting(false)}
        size="narrow"
        title="Chat löschen?"
        actions={
          <>
            <Button onClick={() => setDeleting(false)}>Abbrechen</Button>
            <Button
              kind="danger"
              icon="trash"
              onClick={() => {
                if (chosen === null) return;
                void commands.deleteChat(chosen).then(() => {
                  setDeleting(false);
                  setChosen(null);
                  void loadChats();
                });
              }}
            >
              Löschen
            </Button>
          </>
        }
      />
    </>
  );
}

function Question({ text }: { text: string }) {
  return <p className="max-w-[80%] self-end rounded-[14px] bg-selected px-3.5 py-2 whitespace-pre-wrap">{text}</p>;
}

function Answer({ message, onOpenDocument, onOpenBookings }: { message: ChatMessageRow; onOpenDocument: (document: number) => void; onOpenBookings: () => void }) {
  return (
    <div className="flex flex-col gap-2">
      <p className="whitespace-pre-wrap">{message.text}</p>
      {message.sources.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {message.sources.slice(0, 8).map((source, at) => (
            <button
              key={`${source.document ?? ""}-${source.booking ?? ""}-${at}`}
              type="button"
              onClick={() => (typeof source.document === "number" ? onOpenDocument(source.document) : onOpenBookings())}
              className={cn("inline-flex h-6 items-center gap-1 rounded-full bg-canvas px-2 text-small text-fg-muted shadow-[inset_0_0_0_1px_var(--line)] transition-quick hover:text-fg")}
            >
              <Icon name={typeof source.document === "number" ? "document" : "card"} size={12} />
              {typeof source.document === "number" ? "Dokument" : "Buchung"}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
