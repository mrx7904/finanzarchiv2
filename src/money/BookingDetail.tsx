import { useEffect, useState } from "react";
import { type BookingDetail as Detail, type CategoryRow, type DocumentCard, commands } from "../bindings";
import { MailMark } from "../documents/MailText";
import { PageImage } from "../documents/PageImage";
import { kindLabel } from "../documents/describe";
import { Button, SelectField, cn } from "../ui";
import { cardLine, formatIsoDay, partyYearLine } from "./describe";

// An opened booking of "Umsätze" (#26, the owner's word of 2026-11-26): the booking itself, the
// document that paid it as a card, the party over the year, the other half of a move, and the
// statement only as a reference that opens on demand.
export function BookingDetail({
  booking,
  categories,
  category,
  categoryByParty,
  onCategory,
  onChanged,
  onOpenDocument,
}: {
  booking: number;
  categories: CategoryRow[];
  category: number | null;
  categoryByParty: boolean;
  onCategory: (value: string, forParty: boolean) => void;
  onChanged: () => void;
  onOpenDocument: (document: number) => void;
}) {
  const [detail, setDetail] = useState<Detail | null>(null);
  const [page, setPage] = useState(false);

  useEffect(() => {
    void commands.bookingDetail(booking).then((result) => {
      if (result.status === "ok") setDetail(result.data);
    });
  }, [booking]);

  if (!detail) return <div className="h-32 rounded-control bg-field" aria-busy="true" />;

  const reload = (next: Detail) => {
    setDetail(next);
    onChanged();
  };
  const undoLink = async (link: number) => {
    await commands.undoLink(link);
    const result = await commands.bookingDetail(booking);
    if (result.status === "ok") reload(result.data);
  };
  const link = async (document: number) => {
    const result = await commands.linkToBooking(booking, document);
    if (result.status === "ok") reload(result.data);
  };
  const undoMove = async (pair: number) => {
    await commands.undoMove(pair);
    const result = await commands.bookingDetail(booking);
    if (result.status === "ok") reload(result.data);
  };

  return (
    <div className="flex flex-col gap-4 rounded-control bg-field p-4 text-small">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="flex min-w-0 flex-col gap-0.5">
          <span className="text-body font-medium">{detail.party ?? detail.otherSide ?? detail.kind}</span>
          {detail.otherSide && detail.otherSide !== detail.party ? <span className="text-fg-muted">{detail.otherSide}</span> : null}
          <span className="text-fg-muted">
            {detail.kind} · {formatIsoDay(detail.day)}
            {detail.valueDay && detail.valueDay !== detail.day ? `, Wert ${formatIsoDay(detail.valueDay)}` : ""} · {detail.account}
          </span>
        </div>
        <span className={cn("text-title font-semibold tabular-nums", detail.incoming ? "text-ok" : "text-fg")}>{detail.amount}</span>
      </div>

      {detail.purpose ? <p className="whitespace-pre-wrap break-words text-fg">{detail.purpose}</p> : null}

      <div className="flex flex-wrap items-end gap-2">
        <SelectField
          label={categoryByParty ? "Kategorie (von der Partei)" : "Kategorie"}
          value={category === null ? "" : String(category)}
          onChange={(value) => onCategory(value, false)}
          options={[{ value: "", label: "Keine" }, ...categories.map((row) => ({ value: String(row.id), label: row.name }))]}
        />
        {detail.party && category !== null ? (
          <Button size="small" kind="plain" onClick={() => onCategory(String(category), true)}>
            Für alle von {detail.party}
          </Button>
        ) : null}
      </div>

      <section className="flex flex-col gap-2" aria-label="Beleg">
        <h3 className="font-medium text-fg-muted">Beleg</h3>
        {detail.documents.length === 0 && detail.candidates.length === 0 ? <p className="text-fg-muted">Kein Beleg verknüpft.</p> : null}
        {detail.documents.map((card) => (
          <Card key={card.link} card={card} onOpen={onOpenDocument}>
            <Button size="small" kind="plain" onClick={() => void undoLink(card.link)}>
              Rückgängig
            </Button>
          </Card>
        ))}
        {detail.candidates.length > 0 ? <p className="text-fg-muted">Vielleicht gehört dazu:</p> : null}
        {detail.candidates.map((card) => (
          <Card key={card.document} card={card} onOpen={onOpenDocument}>
            <Button size="small" kind="secondary" onClick={() => void link(card.document)}>
              Das ist der Beleg
            </Button>
          </Card>
        ))}
      </section>

      {detail.movement ? (
        <section className="flex flex-col gap-1" aria-label="Umbuchung">
          <h3 className="font-medium text-fg-muted">Umbuchung</h3>
          <p>
            {detail.movement.why}
            {detail.movement.made ? " Die Gegenbuchung hat das Archiv angelegt." : ""}
          </p>
          {detail.counterpart ? (
            <p className="text-fg-muted tabular-nums">
              Gegenbuchung: {formatIsoDay(detail.counterpart.day)} auf {detail.counterpart.account}, {detail.counterpart.amount}
            </p>
          ) : null}
          <div>
            <Button size="small" kind="plain" onClick={() => void undoMove(detail.movement?.pair ?? 0)}>
              Keine Umbuchung
            </Button>
          </div>
        </section>
      ) : null}

      {detail.partyYear ? (
        <section className="flex flex-col gap-1" aria-label="Verlauf">
          <h3 className="font-medium text-fg-muted">{partyYearLine(detail.partyYear)}</h3>
          <ul className="flex flex-col">
            {detail.partyYear.latest.map((brief) => (
              <li key={brief.id} className="flex justify-between gap-3 tabular-nums">
                <span className="text-fg-muted">
                  {formatIsoDay(brief.day)} · {brief.account}
                </span>
                <span className={brief.incoming ? "text-ok" : "text-fg"}>{brief.amount}</span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      {detail.statement ? (
        <section className="flex flex-col gap-2" aria-label="Kontoauszug">
          <div className="flex flex-wrap items-center justify-between gap-2 text-fg-muted">
            <span>
              Aus {detail.statement.name}
              {detail.statement.page ? `, Seite ${detail.statement.page}` : ""}
            </span>
            <span className="flex gap-1">
              <Button size="small" kind="plain" onClick={() => setPage(!page)}>
                {page ? "Seite ausblenden" : "Seite zeigen"}
              </Button>
              <Button size="small" kind="plain" onClick={() => onOpenDocument(detail.statement?.document ?? 0)}>
                In Dokumente öffnen
              </Button>
            </span>
          </div>
          {page ? <PageImage document={detail.statement.document} page={detail.statement.page ?? 1} width={640} label="Seite des Kontoauszugs" className="max-w-md rounded-control" /> : null}
        </section>
      ) : null}
    </div>
  );
}

function Card({ card, onOpen, children }: { card: DocumentCard; onOpen: (document: number) => void; children: React.ReactNode }) {
  return (
    <div className="flex items-center gap-3 rounded-control bg-surface p-2 shadow-panel">
      <button type="button" className="shrink-0" onClick={() => onOpen(card.document)} aria-label={`${card.name} öffnen`}>
        {card.fileType === "mail" ? (
          <MailMark className="h-14 w-11 rounded-[3px]" />
        ) : (
          <PageImage document={card.document} width={160} label={`Erste Seite von ${card.name}`} className="h-14 w-11 rounded-[3px]" />
        )}
      </button>
      <button type="button" className="flex min-w-0 flex-1 flex-col text-left" onClick={() => onOpen(card.document)}>
        <span className="truncate text-body">{card.sender ?? card.name}</span>
        <span className="truncate text-fg-muted">{cardLine(kindLabel(card.kind), card.number, card.day, card.amount)}</span>
        {card.why ? <span className="truncate text-fg-muted">Warum? {card.why}</span> : null}
      </button>
      <span className="shrink-0">{children}</span>
    </div>
  );
}
