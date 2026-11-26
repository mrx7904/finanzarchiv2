import { useEffect, useState } from "react";
import { commands } from "../bindings";
import { Icon, cn } from "../ui";
import { mailParts } from "./describe";

// The preview of a mail kept as its text (#31): its head as a short table, then the text as code and
// the model read it. Nothing of the mail is loaded, it is plain text.
export function MailText({ document, className }: { document: number; className?: string }) {
  const [text, setText] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    setText(null);
    setFailed(false);
    void commands.mailText(document).then((result) => {
      if (result.status === "ok") setText(result.data);
      else setFailed(true);
    });
  }, [document]);

  if (failed) return <p className="px-2 py-8 text-center text-small text-fg-muted">Der Text der Mail ließ sich nicht lesen.</p>;
  if (text === null) return <div className={cn("h-64 rounded-control bg-field", className)} aria-busy="true" />;
  const { head, body } = mailParts(text);
  return (
    <div className={cn("flex max-h-[32rem] flex-col gap-2 overflow-auto rounded-control bg-field p-3", className)}>
      <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-0.5 text-small">
        {head.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-fg-muted">{label}</dt>
            <dd className="truncate text-fg">{value}</dd>
          </div>
        ))}
      </dl>
      <p className="whitespace-pre-wrap border-t border-line pt-2 text-small text-fg">{body}</p>
    </div>
  );
}

/** The small mark of a mail in the list, in place of a first page. */
export function MailMark({ className }: { className?: string }) {
  return (
    <span className={cn("flex items-center justify-center bg-field text-fg-muted", className)} aria-label="Mail">
      <Icon name="mail" size={14} />
    </span>
  );
}
