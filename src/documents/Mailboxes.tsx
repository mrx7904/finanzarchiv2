import { useEffect, useState, type FormEvent } from "react";
import { type MailboxRow_Serialize as MailboxRow, commands } from "../bindings";
import { ConfirmRemove, Page, Panel } from "../settings/Page";
import { Button, TextField } from "../ui";
import { mailboxLine } from "./describe";

// "Postfächer" of #29: the owner's Gmail, read only. A mailbox is added once its login worked, the
// app password goes to the keychain of the system and is never shown again. Its mails are read every
// 15 minutes and at "Jetzt lesen" (#31).
export function MailboxesPage() {
  const [removing, setRemoving] = useState<MailboxRow | null>(null);
  const [mailboxes, setMailboxes] = useState<MailboxRow[]>([]);
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<"add" | "read" | number | null>(null);

  useEffect(() => {
    void commands.listMailboxes().then((result) => {
      if (result.status === "ok") setMailboxes(result.data);
    });
  }, []);

  const run = async (which: "add" | "read" | number, call: () => ReturnType<typeof commands.listMailboxes>) => {
    setBusy(which);
    const result = await call();
    setBusy(null);
    if (result.status === "ok") {
      setMailboxes(result.data);
      setError(null);
      return true;
    }
    setError(result.error.message);
    return false;
  };

  const add = async (event: FormEvent) => {
    event.preventDefault();
    if (await run("add", () => commands.addMailbox(address, password))) {
      setAddress("");
    }
    // The password leaves the window's state whatever the login said.
    setPassword("");
  };

  return (
    <Page
      actions={
        mailboxes.length > 0 ? (
          <Button kind="primary" busy={busy === "read"} disabled={busy !== null} onClick={() => void run("read", () => commands.readMailNow())}>
            Jetzt lesen
          </Button>
        ) : undefined
      }
    >
      {mailboxes.length > 0 ? (
        <Panel>
        <ul className="flex flex-col">
          {mailboxes.map((mailbox) => (
            <li key={mailbox.id} className="flex min-h-14 items-center justify-between gap-3 border-b border-line py-2 last:border-b-0">
              <span className="min-w-0">
                <span className="block truncate text-body">{mailbox.address}</span>
                <span className={mailbox.problem ? "text-small text-danger" : "text-small text-fg-muted"}>{mailboxLine(mailbox)}</span>
              </span>
              <span className="flex shrink-0 gap-1">
                <Button size="small" kind="plain" busy={busy === mailbox.id} disabled={busy !== null} onClick={() => void run(mailbox.id, () => commands.testMailbox(mailbox.id))}>
                  Anmeldung prüfen
                </Button>
                <Button size="small" kind="plain" icon="trash" disabled={busy !== null} onClick={() => setRemoving(mailbox)}>
                  Entfernen
                </Button>
              </span>
            </li>
          ))}
        </ul>
        </Panel>
      ) : null}
      <form className="flex flex-col gap-2.5 rounded-panel bg-surface px-4 py-3 shadow-panel" onSubmit={(event) => void add(event)}>
        <span className="text-body font-medium">Gmail hinzufügen</span>
        <TextField label="Adresse" type="email" autoComplete="off" value={address} onChange={setAddress} />
        <TextField
          label="App-Passwort"
          type="password"
          autoComplete="off"
          value={password}
          onChange={setPassword}
          hint="Nicht das Passwort des Google-Kontos: ein App-Passwort, angelegt im Google-Konto unter Sicherheit, App-Passwörter."
        />
        <div>
          <Button type="submit" kind="secondary" icon="mail" busy={busy === "add"} disabled={busy !== null || !address || !password}>
            Anmelden und hinzufügen
          </Button>
        </div>
      </form>
      {error ? <p className="text-small text-danger">{error}</p> : null}
      <ConfirmRemove
        open={removing !== null}
        title="Postfach entfernen?"
        goes={["Die Anmeldung und das App-Passwort im Schlüsselbund", "Das Lesen neuer Mails"]}
        stays={["Die Mails im Postfach", "Die Dokumente im Archiv"]}
        onCancel={() => setRemoving(null)}
        onRemove={() => {
          const mailbox = removing;
          setRemoving(null);
          if (mailbox) void run(mailbox.id, () => commands.removeMailbox(mailbox.id));
        }}
      />
    </Page>
  );
}
