import { useEffect, useState } from "react";
import { type NameTemplates, commands } from "../bindings";
import { Button, Dialog, TextField } from "../ui";
import { kindLabel } from "./describe";

// Names by template of #18: the owner renames a document, or changes the template of a kind.

/** The name of a document in the preview, with the file name it came as and "Umbenennen". */
export function DocumentName({ document, name, originalName, byOwner, onChanged }: { document: number; name: string; originalName: string; byOwner: boolean; onChanged: () => void }) {
  const [editing, setEditing] = useState(false);
  const [text, setText] = useState(name);
  useEffect(() => {
    setEditing(false);
    setText(name);
  }, [document, name]);

  const save = async (value: string) => {
    const result = await commands.renameDocument(document, value);
    if (result.status === "ok") {
      setEditing(false);
      onChanged();
    }
  };

  if (editing) {
    return (
      <form
        className="flex items-end gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          void save(text);
        }}
      >
        <TextField label="Name" value={text} onChange={setText} className="flex-1" hint="Leer lassen, um den Namen nach der Vorlage zu bilden." />
        <Button type="submit" size="small" kind="primary">
          Sichern
        </Button>
        <Button size="small" kind="plain" onClick={() => setEditing(false)}>
          Abbrechen
        </Button>
      </form>
    );
  }
  return (
    <div className="flex flex-col gap-0.5">
      <div className="flex items-center justify-between gap-2">
        <p className="text-body font-medium">{name}</p>
        <Button size="small" kind="plain" onClick={() => setEditing(true)}>
          Umbenennen
        </Button>
      </div>
      {name !== originalName ? <p className="text-small text-fg-muted">Datei: {originalName}</p> : null}
      {byOwner ? <p className="text-small text-fg-muted">Von dir benannt, keine Vorlage ändert diesen Namen.</p> : null}
    </div>
  );
}

/** The dialog with one template per kind and the placeholders it may use. */
export function TemplatesDialog({ open, onClose, onChanged }: { open: boolean; onClose: () => void; onChanged: () => void }) {
  const [templates, setTemplates] = useState<NameTemplates | null>(null);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!open) return;
    void commands.nameTemplates().then((result) => {
      if (result.status !== "ok") return;
      setTemplates(result.data);
      setDrafts(Object.fromEntries(result.data.templates.map((entry) => [entry.kind, entry.template])));
      setErrors({});
    });
  }, [open]);

  const save = async () => {
    if (!templates) return;
    const failed: Record<string, string> = {};
    for (const entry of templates.templates) {
      const draft = drafts[entry.kind] ?? "";
      if (draft === entry.template) continue;
      const result = await commands.setNameTemplate(entry.kind, draft);
      if (result.status === "error") failed[entry.kind] = result.error.message;
    }
    setErrors(failed);
    if (Object.keys(failed).length === 0) {
      onChanged();
      onClose();
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Namensvorlagen"
      description="Abgelegte Dokumente heißen nach der Vorlage ihrer Art. Ein Name, den du selbst gibst, bleibt."
      actions={
        <>
          <Button kind="plain" onClick={onClose}>
            Abbrechen
          </Button>
          <Button kind="primary" onClick={() => void save()}>
            Sichern
          </Button>
        </>
      }
    >
      {templates ? (
        <div className="flex flex-col gap-3">
          {templates.templates.map((entry) => (
            <TextField
              key={entry.kind}
              label={kindLabel(entry.kind)}
              value={drafts[entry.kind] ?? ""}
              onChange={(value) => setDrafts({ ...drafts, [entry.kind]: value })}
              hint={`Leer: ${entry.defaultTemplate}`}
              error={errors[entry.kind]}
            />
          ))}
          <p className="text-small text-fg-muted">{templates.placeholders.map((placeholder) => `{${placeholder.name}} ${placeholder.meaning}`).join(" · ")}</p>
        </div>
      ) : null}
    </Dialog>
  );
}
