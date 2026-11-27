// The pieces of #45 in the showcase, each at real size, before a view uses it: the dense rows of Umsätze with the head
// of a month, a row's actions on hover, the filter set by a jump, the switch and the segments, the gray rows of a
// load, the rows of read values, a menu with a long entry opened by a Button, and a dialog that lists what goes and
// what stays. Invented data only.
import { useState } from "react";
import {
  Button,
  Consequences,
  DenseRow,
  Dialog,
  FilterMark,
  GroupHead,
  List,
  LoadingRows,
  Menu,
  Segmented,
  Switch,
  ValueRow,
} from "../ui";

const bookings = [
  { id: "b1", day: "28.11.", title: "Beispiel Software GmbH", detail: "Gehalt November", figure: "+2.594,40 €" },
  { id: "b2", day: "25.11.", title: "Beispiel Markt", detail: "Lebensmittel", figure: "34,12 €" },
  { id: "b3", day: "21.11.", title: "Beispiel Musik", detail: "Abos und Medien", figure: "11,99 €" },
  { id: "b4", day: "03.11.", title: "Wohnbau Musterstadt eG", detail: "Miete und Wohnen", figure: "780,00 €" },
];

const older = [
  { id: "b5", day: "30.10.", title: "Beispiel Software GmbH", detail: "Gehalt Oktober", figure: "+2.594,40 €" },
  { id: "b6", day: "12.10.", title: "Stadtwerke Musterstadt", detail: "Strom und Heizung", figure: "85,00 €" },
];

export function Pieces() {
  const [chosen, setChosen] = useState("b2");
  const [jump, setJump] = useState<string | null>("Beispiel Markt");
  const [thinking, setThinking] = useState(true);
  const [size, setSize] = useState<"s" | "m" | "l">("m");
  const [loading, setLoading] = useState(false);
  const [removing, setRemoving] = useState(false);

  return (
    <div className="grid min-h-0 flex-1 grid-cols-[minmax(360px,520px)_minmax(320px,1fr)] gap-4 overflow-y-auto px-6 pt-4 pb-6">
      <section aria-label="Umsätze" className="flex min-h-0 flex-col rounded-panel bg-surface shadow-panel">
        <div className="flex flex-col gap-2 border-b border-line p-3">
          <div className="flex items-center gap-2">
            {jump ? <FilterMark label={jump} onClear={() => setJump(null)} /> : <Button size="small" onClick={() => setJump("Beispiel Markt")}>Sprung nachstellen</Button>}
          </div>
          <span className="text-small text-fg-muted tabular-nums">6 Buchungen · +5.188,80 € · 1.711,23 €</span>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto">
          {loading ? (
            <LoadingRows loading density="dense" rows={8} />
          ) : (
            <List label="Umsätze" className="gap-0">
              <GroupHead title="November 2026" figures={<><span>+2.594,40 €</span><span>826,11 €</span></>} />
              {bookings.map((b) => (
                <DenseRow
                  key={b.id}
                  lead={b.day}
                  title={b.title}
                  detail={b.detail}
                  figure={b.figure}
                  selected={b.id === chosen}
                  onSelect={() => setChosen(b.id)}
                  actions={<Button kind="plain" size="small" icon="more" label="Mehr" />}
                />
              ))}
              <GroupHead title="Oktober 2026" figures={<><span>+2.594,40 €</span><span>85,00 €</span></>} />
              {older.map((b) => (
                <DenseRow key={b.id} lead={b.day} title={b.title} detail={b.detail} figure={b.figure} selected={b.id === chosen} onSelect={() => setChosen(b.id)} />
              ))}
            </List>
          )}
        </div>
      </section>

      <div className="flex flex-col gap-4">
        <section aria-label="Auswahl" className="flex flex-col gap-3 rounded-panel bg-surface p-4 shadow-panel">
          <Switch label="Nachdenken" checked={thinking} onChange={setThinking} />
          <Segmented label="Größe" value={size} onChange={setSize} options={[{ value: "s", label: "Klein" }, { value: "m", label: "Mittel" }, { value: "l", label: "Groß" }]} />
          <div className="flex gap-2">
            <Button
              onClick={() => {
                setLoading(true);
                window.setTimeout(() => setLoading(false), 1600);
              }}
            >
              Laden nachstellen
            </Button>
            <Menu
              align="start"
              entries={[
                { label: "Neu lesen lassen", icon: "upload", onSelect: () => undefined },
                { label: "Als Tabelle exportieren und in den Ordner Downloads legen …", icon: "folder", onSelect: () => undefined },
                { label: "Löschen …", icon: "trash", danger: true, onSelect: () => setRemoving(true) },
              ]}
              trigger={(props) => (
                <Button {...props} icon="more">
                  Mehr
                </Button>
              )}
            />
          </div>
        </section>

        <section aria-label="Angaben" className="flex flex-col rounded-panel bg-surface px-4 py-2 shadow-panel">
          <ValueRow name="Absender" value="Stadtwerke Musterstadt" source="read" />
          <ValueRow name="Datum" value="12.10.2026" source="read" />
          <ValueRow name="Netto" value="71,43 €" source="checked" amount />
          <ValueRow name="Brutto" value="85,00 €" source="sender" amount />
          <ValueRow name="Kategorie" value="Strom und Heizung" source="owner" />
        </section>
      </div>

      <Dialog
        open={removing}
        onClose={() => setRemoving(false)}
        size="narrow"
        title="Dokument löschen?"
        actions={
          <>
            <Button onClick={() => setRemoving(false)}>Abbrechen</Button>
            <Button kind="danger" icon="trash" onClick={() => setRemoving(false)}>
              Löschen
            </Button>
          </>
        }
      >
        <Consequences goes={["Die Rechnung und ihre Seiten", "Die Verknüpfung mit der Buchung vom 12.10."]} stays={["Die Buchung im Konto", "Die Datei im Papierkorb des Archivs"]} />
      </Dialog>
    </div>
  );
}
