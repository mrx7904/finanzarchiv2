import { useEffect, useState } from "react";
import { type CategoryRow, commands } from "../bindings";
import { Page } from "../settings/Page";
import { Button, SelectField, TextField } from "../ui";

// The categories of #25, a page of Einstellungen since #49: rename, remove and add, each sent to the core at once.
export function CategoriesPage() {
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
    void commands.listCategories().then((result) => {
      if (result.status === "ok") take(result.data);
    });
  }, []);

  const run = async (call: () => ReturnType<typeof commands.listCategories>) => {
    const result = await call();
    if (result.status === "ok") {
      take(result.data);
      setError(null);
    } else setError(result.error.message);
  };

  return (
    <Page>
      <div className="flex flex-col gap-1.5 rounded-panel bg-surface px-4 py-3 shadow-panel">
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
        className="flex items-end gap-2 rounded-panel bg-surface px-4 py-3 shadow-panel"
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
      {error ? <p className="text-small text-danger">{error}</p> : null}
    </Page>
  );
}
