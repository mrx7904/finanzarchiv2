import { useEffect, useState } from "react";
import { type CategoryRow, commands } from "../bindings";
import { Button, Dialog, SelectField, TextField } from "../ui";

// The categories of #25 in a dialog: rename, remove and add, each sent to the core at once.
export function Categories({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [categories, setCategories] = useState<CategoryRow[]>([]);
  const [names, setNames] = useState<Record<number, string>>({});
  const [added, setAdded] = useState("");
  const [direction, setDirection] = useState("out");
  const [error, setError] = useState<string | null>(null);

  const take = (rows: CategoryRow[]) => {
    setCategories(rows);
    setNames(Object.fromEntries(rows.map((row) => [row.id, row.name])));
  };
  useEffect(() => {
    if (!open) return;
    setError(null);
    void commands.listCategories().then((result) => {
      if (result.status === "ok") take(result.data);
    });
  }, [open]);

  const run = async (call: () => ReturnType<typeof commands.listCategories>) => {
    const result = await call();
    if (result.status === "ok") {
      take(result.data);
      setError(null);
    } else setError(result.error.message);
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Kategorien"
      description="Eine Buchung bekommt die Kategorie ihrer Partei, solange du ihr keine eigene gibst."
      actions={
        <Button kind="primary" onClick={onClose}>
          Fertig
        </Button>
      }
    >
      <div className="flex max-h-[50vh] flex-col gap-1.5 overflow-y-auto">
        {categories.map((category) => (
          <div key={category.id} className="flex items-end gap-2">
            <TextField
              label={category.direction === "in" ? "Einnahme" : "Ausgabe"}
              value={names[category.id] ?? ""}
              onChange={(value) => setNames({ ...names, [category.id]: value })}
              onBlur={() => {
                if (names[category.id] !== category.name) void run(() => commands.renameCategory(category.id, names[category.id] ?? ""));
              }}
              className="flex-1"
            />
            <Button size="small" kind="plain" icon="trash" label={`${category.name} entfernen`} onClick={() => void run(() => commands.removeCategory(category.id))} />
          </div>
        ))}
      </div>
      <form
        className="mt-3 flex items-end gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          void run(() => commands.addCategory(added, direction)).then(() => setAdded(""));
        }}
      >
        <TextField label="Neue Kategorie" value={added} onChange={setAdded} className="flex-1" />
        <SelectField
          label="Art"
          value={direction}
          onChange={setDirection}
          options={[
            { value: "out", label: "Ausgabe" },
            { value: "in", label: "Einnahme" },
          ]}
        />
        <Button type="submit" icon="plus">
          Hinzufügen
        </Button>
      </form>
      {error ? <p className="mt-2 text-small text-danger">{error}</p> : null}
    </Dialog>
  );
}
