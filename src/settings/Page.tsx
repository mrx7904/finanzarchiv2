// The frame of one page of "Einstellungen" (#49): its actions in the top bar (rule 2), its content in one column, and
// the question before something is removed, with what goes and what stays (rule 18).
import type { ReactNode } from "react";
import { BarActions, Button, Consequences, Dialog } from "../ui";

export function Page({ actions, children }: { actions?: ReactNode; children: ReactNode }) {
  return (
    <>
      {actions && <BarActions>{actions}</BarActions>}
      <div className="flex w-full max-w-2xl flex-col gap-4">{children}</div>
    </>
  );
}

/** A panel of a page: rows of what is set up, with a hairline between them. */
export function Panel({ children }: { children: ReactNode }) {
  return <div className="flex flex-col rounded-panel bg-surface px-4 py-1 shadow-panel">{children}</div>;
}

type ConfirmRemoveProps = {
  open: boolean;
  title: string;
  goes: string[];
  stays: string[];
  onCancel: () => void;
  onRemove: () => void;
};

export function ConfirmRemove({ open, title, goes, stays, onCancel, onRemove }: ConfirmRemoveProps) {
  return (
    <Dialog
      open={open}
      onClose={onCancel}
      size="narrow"
      title={title}
      actions={
        <>
          <Button onClick={onCancel}>Abbrechen</Button>
          <Button kind="danger" icon="trash" onClick={onRemove}>
            Entfernen
          </Button>
        </>
      }
    >
      <Consequences goes={goes} stays={stays} />
    </Dialog>
  );
}
