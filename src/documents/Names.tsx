import { useEffect, useState } from "react";
import { type NameTemplates, commands } from "../bindings";
import { Page } from "../settings/Page";
import { Button, TextField } from "../ui";
import { kindLabel } from "./describe";

// Names by template of #18: the template of each kind, a page of Einstellungen since #49. A document is renamed by
// "Umbenennen …" at the document (#46).

/** The page "Namen" of Einstellungen (#49): one template per kind and the placeholders it may use. */
export function TemplatesPage() {
  const [saved, setSaved] = useState(false);
  const [templates, setTemplates] = useState<NameTemplates | null>(null);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    void commands.nameTemplates().then((result) => {
      if (result.status !== "ok") return;
      setTemplates(result.data);
      setDrafts(Object.fromEntries(result.data.templates.map((entry) => [entry.kind, entry.template])));
      setErrors({});
    });
  }, []);

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
    setSaved(Object.keys(failed).length === 0);
  };

  return (
    <Page
      actions={
        <>
          {saved && <span className="text-small text-fg-muted">Gesichert</span>}
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
              onChange={(value) => {
                setDrafts({ ...drafts, [entry.kind]: value });
                setSaved(false);
              }}
              hint={`Leer: ${entry.defaultTemplate}`}
              error={errors[entry.kind]}
            />
          ))}
          <p className="text-small text-fg-muted">{templates.placeholders.map((placeholder) => `{${placeholder.name}} ${placeholder.meaning}`).join(" · ")}</p>
        </div>
      ) : null}
    </Page>
  );
}
