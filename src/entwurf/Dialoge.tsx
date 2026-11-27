// The dialogs of the draft (docs/entwurf-ansichten.md, rule 18): a dialog asks before something is removed or moved,
// or for one short input, or lets the owner pick one thing. Each lies over the view it is opened from. The dialog is
// the real piece of the design system (src/ui/Dialog.tsx), the data are invented.
import type { ReactNode } from "react";
import { Button, Dialog, Icon, List, ListRow, TextField } from "../ui";
import { Dokumente, Einstellungen, Page } from "./Entwurf";
import { Archiv, Bestellungen, Ordner } from "./Unterseiten";

const nothing = () => undefined;

function Cancel({ children = "Abbrechen" }: { children?: ReactNode }) {
  return <Button kind="plain">{children}</Button>;
}

/** What a removal takes with it and what it leaves, as two short lines, so the button's weight is clear. */
function Consequences({ rows }: { rows: [boolean, string][] }) {
  return (
    <ul className="flex flex-col gap-1.5">
      {rows.map(([stays, text]) => (
        <li key={text} className="flex items-start gap-2.5">
          <Icon name={stays ? "check" : "close"} size={15} className={stays ? "mt-0.5 text-fg-muted" : "mt-0.5 text-danger"} />
          <span>{text}</span>
        </li>
      ))}
    </ul>
  );
}

function DokumentLoeschen() {
  return (
    <>
      <Dokumente />
      <Dialog
        open
        onClose={nothing}
        size="narrow"
        title="Dieses Dokument löschen?"
        description="2026-11-14 Stadtwerke Beispielstadt Rechnung"
        actions={
          <>
            <Cancel />
            <Button kind="danger">Löschen</Button>
          </>
        }
      >
        <Consequences
          rows={[
            [false, "Das Dokument und seine gelesenen Werte verlassen das Archiv."],
            [true, "Die Datei kommt in den Papierkorb neben dem Archiv, bis du ihn leerst."],
            [true, "Die Buchung vom 18.11. bleibt und steht dann ohne Beleg da."],
          ]}
        />
      </Dialog>
    </>
  );
}

function Umbenennen() {
  return (
    <>
      <Dokumente />
      <Dialog
        open
        onClose={nothing}
        size="narrow"
        title="Umbenennen"
        actions={
          <>
            <Button kind="plain" className="mr-auto">
              Name nach Vorlage
            </Button>
            <Cancel />
            <Button kind="primary">Umbenennen</Button>
          </>
        }
      >
        <TextField label="Name" value="2026-11-14 Stadtwerke Abschlag November" onChange={nothing} hint="Ein Name von dir bleibt, auch wenn sich die Vorlage ändert." />
      </Dialog>
    </>
  );
}

function RechnungZuordnen() {
  return (
    <>
      <Bestellungen />
      <Dialog
        open
        onClose={nothing}
        title="Welche Rechnung gehört zur Bestellung vom 09.11.?"
        description="Schreibtischlampe, Ersatzbirne, 64,90 €"
        actions={
          <>
            <Button kind="plain" icon="upload" className="mr-auto">
              Datei auswählen …
            </Button>
            <Cancel />
            <Button kind="primary">Zuordnen</Button>
          </>
        }
      >
        <div className="-mx-1 flex flex-col gap-2">
          <p className="px-1 text-small font-medium text-fg-muted">Rechnungen ohne Bestellung, die passende zuerst</p>
          <List label="Rechnungen" className="p-0">
            <ListRow title="Amazon" detail="Rechnung vom 10.11.2026, gleicher Betrag" figure="64,90 €" leading={<Page className="w-6 !p-[2px] !shadow-panel" />} selected onSelect={nothing} />
            <ListRow title="Amazon" detail="Rechnung vom 03.11.2026" figure="23,49 €" leading={<Page className="w-6 !p-[2px] !shadow-panel" />} onSelect={nothing} />
            <ListRow title="Lampenhaus Beispiel" detail="Rechnung vom 28.10.2026" figure="61,00 €" leading={<Page className="w-6 !p-[2px] !shadow-panel" />} onSelect={nothing} />
          </List>
        </div>
      </Dialog>
    </>
  );
}

function Wiederherstellen() {
  return (
    <>
      <Archiv />
      <Dialog
        open
        onClose={nothing}
        size="narrow"
        title="Die Sicherung von gestern, 19:41, wiederherstellen?"
        actions={
          <>
            <Cancel />
            <Button kind="primary">Wiederherstellen</Button>
          </>
        }
      >
        <Consequences
          rows={[
            [false, "Was seit gestern, 19:41, dazukam, ist danach nicht mehr im Archiv: 23 Dokumente."],
            [true, "Der heutige Stand wird vorher selbst gesichert, du kannst zurück."],
            [true, "Deine Originale in den Ordnern bleiben, wie sie sind."],
          ]}
        />
      </Dialog>
    </>
  );
}

function Umziehen() {
  return (
    <>
      <Archiv />
      <Dialog
        open
        onClose={nothing}
        title="Archiv umziehen"
        description="Das Archiv wird an den neuen Ort kopiert, dort geprüft und erst dann am alten Ort entfernt."
        actions={
          <>
            <Cancel />
            <Button kind="primary">Umziehen</Button>
          </>
        }
      >
        <div className="flex flex-col gap-4">
          <div className="flex items-end gap-2">
            <TextField label="Neuer Ort" value="E:\Archiv\Finanzen" onChange={nothing} className="flex-1" hint="412 MB werden gebraucht, 118 GB sind frei" />
            <Button icon="folder" className="mb-[22px]">
              Auswählen …
            </Button>
          </div>
          <Consequences
            rows={[
              [true, "1.284 Dokumente und 1.730 Buchungen ziehen mit, dazu alle Sicherungen."],
              [true, "Währenddessen ruht der Eingang. Danach liest er weiter, wo er stand."],
            ]}
          />
        </div>
      </Dialog>
    </>
  );
}

function PostfachEntfernen() {
  return (
    <>
      <Einstellungen page="Postfächer" />
      <Dialog
        open
        onClose={nothing}
        size="narrow"
        title="Dieses Postfach entfernen?"
        description="beispiel@example.org"
        actions={
          <>
            <Cancel />
            <Button kind="danger">Entfernen</Button>
          </>
        }
      >
        <Consequences
          rows={[
            [false, "Finanzarchiv liest dieses Postfach nicht mehr, das App-Passwort wird aus dem Schlüsselbund gelöscht."],
            [true, "Die 41 Rechnungen, die daraus kamen, bleiben im Archiv."],
            [true, "Im Postfach selbst ändert sich nichts."],
          ]}
        />
      </Dialog>
    </>
  );
}

function OrdnerEntfernen() {
  return (
    <>
      <Ordner />
      <Dialog
        open
        onClose={nothing}
        size="narrow"
        title="Den Ordner „Steuer 2026“ nicht mehr beobachten?"
        actions={
          <>
            <Cancel />
            <Button kind="primary">Nicht mehr beobachten</Button>
          </>
        }
      >
        <Consequences
          rows={[
            [true, "Die 58 Dokumente, die daraus kamen, bleiben im Archiv."],
            [true, "Der Ordner und seine Dateien bleiben, wie sie sind."],
          ]}
        />
      </Dialog>
    </>
  );
}

export const dialoge = {
  "dialog-dokument-loeschen": () => <DokumentLoeschen />,
  "dialog-umbenennen": () => <Umbenennen />,
  "dialog-rechnung-zuordnen": () => <RechnungZuordnen />,
  "dialog-wiederherstellen": () => <Wiederherstellen />,
  "dialog-umziehen": () => <Umziehen />,
  "dialog-postfach-entfernen": () => <PostfachEntfernen />,
  "dialog-ordner-entfernen": () => <OrdnerEntfernen />,
} satisfies Record<string, () => ReactNode>;
