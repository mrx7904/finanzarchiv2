import { useEffect, useState } from "react";
import { type HintRow, type RecurringRow_Serialize as RecurringRow, type SalaryYearRow_Serialize as SalaryYearRow, type YearRow_Serialize as YearRow, commands } from "../bindings";
import { recurringSums } from "../money/recurrence";
import type { BookingJump } from "../navigation";
import { BarActions, Button, EmptyState, LoadingRows, Segmented } from "../ui";
import { BalanceLine, Figure, MonthBars, RankBars, euros } from "./charts";
import { balancesOfYear, lastMonth, monthRange, monthValues, nettoByMonth, periodWords, sameMonths, savingsRate } from "./derive";

// "Auswertungen" (#38) in the look picked in #37 (variant A, knowledge/decision-evaluations.md): one column of sections
// for the year the owner picks, the key figures first. Every bar and figure opens the bookings behind it in Umsätze,
// filtered; the recurring payments lead to their page under Konten.

/** The ink for money in, a light gray for money out (one step lighter in dark), gray categories (rule 12). */
const COLORS = { income: "var(--fg)", spending: "light-dark(#d1d1d6, #52525b)", category: "var(--line-strong)", line: "var(--fg)" };

type Loaded = { year: YearRow; before: YearRow | null; salary: SalaryYearRow | null; balances: { day: string; cents: number }[]; recurring: RecurringRow[]; hints: HintRow[] };

export function Evaluations({ year: wanted, onYear, onBookings, onRecurring }: { year: number | null; onYear: (year: number) => void; onBookings: (filter: BookingJump) => void; onRecurring: () => void }) {
  const [years, setYears] = useState<number[] | null>(null);
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [month, setMonth] = useState<number | null>(null);
  useEffect(() => {
    void commands.evaluationYears().then((result) => {
      if (result.status === "ok") setYears(result.data);
      else setFailed(result.error.message);
    });
  }, []);
  const year = wanted ?? years?.[0] ?? null;
  useEffect(() => {
    if (year === null) return;
    setLoaded(null);
    void Promise.all([commands.evaluationYear(year), commands.evaluationYear(year - 1), commands.salaryYears(), commands.balancesByMonth(), commands.listRecurring(), commands.listHints()]).then(
      ([this_, before, salary, balances, recurring, hints]) => {
        if (this_.status !== "ok") {
          setFailed(this_.error.message);
          return;
        }
        setLoaded({
          year: this_.data,
          before: before.status === "ok" ? before.data : null,
          salary: salary.status === "ok" ? (salary.data.find((one) => one.year === year) ?? null) : null,
          balances: balances.status === "ok" ? balances.data.total : [],
          recurring: recurring.status === "ok" ? recurring.data : [],
          hints: hints.status === "ok" ? hints.data : [],
        });
      },
    );
  }, [year]);

  const bar = years && years.length > 1 && (
    <BarActions>
      <Segmented label="Jahr" value={String(year)} onChange={(value) => onYear(Number(value))} options={years.slice(0, 5).map((one) => ({ value: String(one), label: String(one) }))} />
    </BarActions>
  );
  if (failed) {
    return (
      <div className="flex flex-col items-start gap-3 px-6 pt-6">
        <p className="text-danger">{failed}</p>
        <Button onClick={() => window.location.reload()}>Erneut versuchen</Button>
      </div>
    );
  }
  if (years !== null && years.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <EmptyState icon="scale" title="Noch keine Buchungen" hint="Die Auswertungen entstehen aus den Kontoauszügen." />
      </div>
    );
  }
  if (!loaded || year === null) {
    return (
      <>
        {bar}
        <div className="mx-auto w-full max-w-[1120px] px-6 pt-6">
          <LoadingRows loading rows={6} />
        </div>
      </>
    );
  }

  const { year: row, before } = loaded;
  const last = lastMonth(row.income, row.spending);
  const income = row.incomeTotal.cents;
  const spending = row.spendingTotal.cents;
  const beforeIncome = before ? sameMonths(before.income, last) : null;
  const beforeSpending = before ? sameMonths(before.spending, last) : null;
  const rate = savingsRate(income, spending);
  const period = periodWords(last);
  const yearRange = { from: `${year}-01-01`, to: `${year}-12-31` };
  const categories = row.categories.filter((part) => !part.income).map((part) => ({ name: part.name ?? "Ohne Kategorie", cents: part.sum.cents, id: part.id }));
  const netto = loaded.salary ? nettoByMonth(loaded.salary.months) : null;
  const balances = balancesOfYear(loaded.balances, year);
  const recurring = recurringSums(loaded.recurring);
  const savings = loaded.hints.slice(0, 3);

  return (
    <>
      {bar}
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        <div className="mx-auto flex w-full max-w-[1120px] flex-col gap-6 px-6 pt-6 pb-10">
          <div className="grid grid-cols-3 gap-6 max-[1100px]:grid-cols-1">
            <FigureButton onClick={() => onBookings({ ...yearRange, label: `${year}` })}>
              <Figure label={`Einnahmen ${year}`} value={euros(income, true)} detail={beforeIncome !== null ? `${year - 1} bis ${period.split(" bis ").at(-1)}: ${euros(beforeIncome, true)}` : undefined} large />
            </FigureButton>
            <FigureButton onClick={() => onBookings({ ...yearRange, label: `${year}` })}>
              <Figure label={`Ausgaben ${year}`} value={euros(spending)} detail={beforeSpending !== null ? `${year - 1} bis ${period.split(" bis ").at(-1)}: ${euros(beforeSpending)}` : undefined} large />
            </FigureButton>
            <Figure label="Saldo" value={euros(income - spending, true)} detail={rate !== null ? `${rate} % der Einnahmen, ${period}` : period} large />
          </div>

          <Section title="Ein- und Ausgaben" figure="nach Monat">
            <MonthBars
              series={[
                { name: "Einnahmen", values: monthValues(row.income, last), color: COLORS.income },
                { name: "Ausgaben", values: monthValues(row.spending, last), color: COLORS.spending },
              ]}
              chosen={month}
              onChoose={(at) => {
                setMonth(at);
                onBookings(monthRange(year, at));
              }}
            />
          </Section>

          <div className="grid grid-cols-2 gap-6 max-[1100px]:grid-cols-1">
            <Section title="Ausgaben nach Kategorie" figure={euros(spending)}>
              <RankBars
                parts={categories}
                color={COLORS.category}
                onChoose={(name) => {
                  const chosen = categories.find((part) => part.name === name);
                  if (chosen?.id !== null && chosen?.id !== undefined) onBookings({ ...yearRange, category: chosen.id, label: `${name} ${year}` });
                }}
              />
            </Section>
            <Section title="Kontostand" figure={balances.values.length > 0 ? euros(balances.values.at(-1) ?? 0) : undefined}>
              {balances.values.length > 1 ? <BalanceLine values={balances.values} labels={balances.labels} color={COLORS.line} /> : <p className="text-fg-muted">Kein Kontostand in {year}</p>}
            </Section>
          </div>

          <div className="grid grid-cols-2 gap-6 max-[1100px]:grid-cols-1">
            <Section title="Gehalt" figure={netto ? `Netto${loaded.salary && loaded.salary.missing > 0 ? `, ${loaded.salary.missing} ${loaded.salary.missing === 1 ? "Monat fehlt" : "Monate fehlen"}` : ""}` : undefined}>
              {netto ? <MonthBars series={[{ name: "Netto", values: netto, color: COLORS.income }]} missing height={120} /> : <p className="text-fg-muted">Keine Lohnabrechnung in {year}</p>}
            </Section>
            <Section title="Wiederkehrend" figure={`${recurring.running} laufend`}>
              <button type="button" onClick={onRecurring} className="flex flex-col items-start gap-0.5 rounded-row px-2 py-1.5 text-left transition-quick hover:bg-hover">
                <span className="text-[22px] leading-snug font-semibold tabular-nums">{euros(recurring.month)}</span>
                <span className="text-small text-fg-muted tabular-nums">im Monat · {euros(recurring.year)} im Jahr</span>
              </button>
              {savings.length > 0 && (
                <ul aria-label="Spartipps" className="flex flex-col gap-1 border-t border-line pt-3">
                  {savings.map((hint) => (
                    <li key={hint.key} className="flex flex-col">
                      <span className="text-small font-medium">{hint.subject}</span>
                      <span className="text-small text-fg-muted">{hint.line}</span>
                    </li>
                  ))}
                </ul>
              )}
            </Section>
          </div>
        </div>
      </div>
    </>
  );
}

function Section({ title, figure, children }: { title: string; figure?: string; children: React.ReactNode }) {
  return (
    <section aria-label={title} className="flex min-w-0 flex-col gap-3 border-t border-line pt-4">
      <div className="flex items-baseline justify-between gap-3">
        <h2 className="text-heading font-semibold">{title}</h2>
        {figure && <span className="text-small text-fg-muted tabular-nums">{figure}</span>}
      </div>
      {children}
    </section>
  );
}

function FigureButton({ onClick, children }: { onClick: () => void; children: React.ReactNode }) {
  return (
    <button type="button" onClick={onClick} className="-mx-2 rounded-row px-2 py-1 text-left transition-quick hover:bg-hover">
      {children}
    </button>
  );
}
