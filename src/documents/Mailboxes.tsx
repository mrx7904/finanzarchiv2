import { useEffect, useState, type FormEvent } from "react";
import { type MailboxRow_Serialize as MailboxRow, commands } from "../bindings";
import { Button, Dialog, TextField } from "../ui";
import { mailboxLine } from "./describe";

// "Postfächer" of #29: the owner's Gmail, read only. A mailbox is added once its login worked, the
// app password goes to the keychain of the system and is never shown again.
export function Mailboxes({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [mailboxes, setMailboxes] = useState<MailboxRow[]>([]);
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<"add" | number | null>(null);

  useEffect(() => {
    if (!open) return;
    setError(null);
    setPassword("");
    void commands.listMailboxes().then((result) => {
      if (result.status === "ok") setMailboxes(result.data);
    });
  }, [open]);

  const run = async (which: "add" | number, call: () => ReturnType<typeof commands.listMailboxes>) => {
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
    <Dialog
      open={open}
      onClose={onClose}
      title="Postfächer"
      description="Das Archiv liest dein Gmail nur: Es verschiebt, löscht, sendet und markiert nichts. Das App-Passwort liegt im Schlüsselbund des Systems."
      actions={
        <Button kind="primary" onClick={onClose}>
          Fertig
        </Button>
      }
    >
      {mailboxes.length > 0 ? (
        <ul className="mb-4 flex flex-col gap-2">
          {mailboxes.map((mailbox) => (
            <li key={mailbox.id} className="flex items-center justify-between gap-3">
              <span className="min-w-0">
                <span className="block truncate text-body">{mailbox.address}</span>
                <span className={mailbox.problem ? "text-small text-danger" : "text-small text-fg-muted"}>{mailboxLine(mailbox)}</span>
              </span>
              <span className="flex shrink-0 gap-1">
                <Button size="small" kind="plain" busy={busy === mailbox.id} disabled={busy !== null} onClick={() => void run(mailbox.id, () => commands.testMailbox(mailbox.id))}>
                  Anmeldung prüfen
                </Button>
                <Button size="small" kind="plain" icon="trash" disabled={busy !== null} onClick={() => void run(mailbox.id, () => commands.removeMailbox(mailbox.id))}>
                  Entfernen
                </Button>
              </span>
            </li>
          ))}
        </ul>
      ) : null}
      <form className="flex flex-col gap-2.5 border-t border-line pt-3" onSubmit={(event) => void add(event)}>
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
      {error ? <p className="mt-2 text-small text-danger">{error}</p> : null}
    </Dialog>
  );
}
