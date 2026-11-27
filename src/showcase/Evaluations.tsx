// The variants of "Auswertungen" (#37) at real size, for the owner's pick (decided for the owner by Fable 5.2 while
// the owner is away): A keeps rule 12 to the letter (gray, the chosen mark in the accent), B gives the two series the
// first two hues of the validated palette of the dataviz skill and the categories one hue, C is the layout of 1.0
// (decision-dashboard-look.md there) in Graphit. Invented numbers; in the dev app with the core the same pieces show
// the archive's own (#38).
import { useState } from "react";
import { BalanceLine, Figure, MonthBars, RankBars, euros } from "../evaluations/charts";
import { Segmented, cn } from "../ui";

const income = [259440, 259440, 259440, 259440, 278900, 259440, 259440, 259440, 259440, 259440, null, null];
const spending = [182310, 141220, 168900, 152040, 201330, 139870, 233110, 147650, 158220, 149990, null, null];
const salary = [259440, 259440, null, 259440, 259440, 259440, 259440, 259440, 259440, 259440, null, null];
const balances = [412300, 489400, 580000, 687400, 765000, 884600, 910900, 1022700, 1123900, 1233300];
const months = ["31.01.", "28.02.", "31.03.", "30.04.", "31.05.", "30.06.", "31.07.", "31.08.", "30.09.", "31.10."];
const categories = [
  { name: "Miete und Wohnen", cents: 780000 },
  { name: "Lebensmittel", cents: 341220 },
  { name: "Auto und Tanken", cents: 182040 },
  { name: "Versicherungen", cents: 121880 },
  { name: "Restaurant und Café", cents: 98760 },
  { name: "Abos und Medien", cents: 43180 },
  { name: "Strom und Heizung", cents: 85000 },
  { name: "Ohne Kategorie", cents: 22950 },
].sort((a, b) => b.cents - a.cents);
const total = (values: (number | null)[]) => values.reduce<number>((sum, value) => sum + (value ?? 0), 0);
const incomeTotal = total(income);
const spendingTotal = total(spending);
const before = { income: 3004000, spending: 1922400 };

type Colors = { income: string; spending: string; category: string; line: string };
// The pick of 2026-11-27 (Fable 5.2 for the owner): A, the money out one step lighter in dark, the line in the ink.
export const GRAPHIT: Colors = { income: "var(--fg)", spending: "light-dark(#d1d1d6, #52525b)", category: "var(--line-strong)", line: "var(--fg)" };
const PALETTE: Colors = { income: "light-dark(#2a78d6, #3987e5)", spending: "light-dark(#eb6834, #d95926)", category: "light-dark(#2a78d6, #3987e5)", line: "light-dark(#2a78d6, #3987e5)" };

export function EvaluationVariants() {
  const [variant, setVariant] = useState<"a" | "b" | "c">("a");
  const [month, setMonth] = useState<number | null>(9);
  const [category, setCategory] = useState<string | null>(null);
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-6 pt-4 pb-8">
      <Segmented
        label="Variante"
        value={variant}
        onChange={setVariant}
        options={[
          { value: "a", label: "A Graphit" },
          { value: "b", label: "B Eine Farbe je Reihe" },
          { value: "c", label: "C Wie 1.0" },
        ]}
      />
      {variant === "c" ? (
        <Tiles month={month} onMonth={setMonth} category={category} onCategory={setCategory} />
      ) : (
        <Sections colors={variant === "a" ? GRAPHIT : PALETTE} month={month} onMonth={setMonth} category={category} onCategory={setCategory} />
      )}
    </div>
  );
}

type Choice = { month: number | null; onMonth: (month: number) => void; category: string | null; onCategory: (name: string) => void };

function Section({ title, figure, children }: { title: string; figure?: string; children: React.ReactNode }) {
  return (
    <section aria-label={title} className="flex flex-col gap-3 border-t border-line pt-4">
      <div className="flex items-baseline justify-between gap-3">
        <h2 className="text-heading font-semibold">{title}</h2>
        {figure && <span className="text-small text-fg-muted tabular-nums">{figure}</span>}
      </div>
      {children}
    </section>
  );
}

/** A and B: one column of sections up to 1120 px, the key figures first. */
function Sections({ colors, month, onMonth, category, onCategory }: Choice & { colors: Colors }) {
  return (
    <div className="mx-auto flex w-full max-w-[1120px] flex-col gap-6">
      <div className="grid grid-cols-3 gap-6">
        <Figure label="Einnahmen 2026" value={euros(incomeTotal, true)} detail={`2025 bis Oktober: ${euros(before.income, true)}`} large />
        <Figure label="Ausgaben 2026" value={euros(spendingTotal)} detail={`2025 bis Oktober: ${euros(before.spending)}`} large />
        <Figure label="Saldo" value={euros(incomeTotal - spendingTotal, true)} detail={`${Math.round(((incomeTotal - spendingTotal) / incomeTotal) * 100)} % der Einnahmen, Januar bis Oktober`} large />
      </div>
      <Section title="Ein- und Ausgaben" figure="nach Monat">
        <MonthBars
          series={[
            { name: "Einnahmen", values: income, color: colors.income },
            { name: "Ausgaben", values: spending, color: colors.spending },
          ]}
          chosen={month}
          onChoose={onMonth}
        />
      </Section>
      <div className="grid grid-cols-2 gap-6">
        <Section title="Ausgaben nach Kategorie" figure={euros(spendingTotal)}>
          <RankBars parts={categories} color={colors.category} chosen={category} onChoose={onCategory} />
        </Section>
        <Section title="Kontostand" figure={euros(balances.at(-1) ?? 0)}>
          <BalanceLine values={balances} labels={months} color={colors.line} />
        </Section>
      </div>
      <div className="grid grid-cols-2 gap-6">
        <Section title="Gehalt" figure="Netto, 1 Monat fehlt">
          <MonthBars series={[{ name: "Netto", values: salary, color: colors.income }]} missing height={120} />
        </Section>
        <Section title="Im Vergleich zu 2025" figure="Januar bis Oktober">
          <Comparison colors={colors} />
        </Section>
      </div>
    </div>
  );
}

function Comparison({ colors }: { colors: Colors }) {
  const rows = [
    { name: "Einnahmen", now: incomeTotal, last: before.income, color: colors.income },
    { name: "Ausgaben", now: spendingTotal, last: before.spending, color: colors.spending },
  ];
  const top = Math.max(...rows.flatMap((row) => [row.now, row.last]));
  return (
    <div className="flex flex-col gap-3">
      {rows.map((row) => (
        <div key={row.name} className="flex flex-col gap-1">
          <div className="flex justify-between text-small">
            <span>{row.name}</span>
            <span className="tabular-nums text-fg-muted">
              {row.now >= row.last ? "+" : "-"}
              {euros(Math.abs(row.now - row.last))}
            </span>
          </div>
          <span title={`2026: ${euros(row.now)}`} className="h-2.5 rounded-r-[4px]" style={{ width: `${(row.now / top) * 100}%`, background: row.color }} />
          <span title={`2025: ${euros(row.last)}`} className="h-2.5 rounded-r-[4px] bg-line" style={{ width: `${(row.last / top) * 100}%` }} />
        </div>
      ))}
    </div>
  );
}

/** C: the layout of 1.0 in Graphit: a row of tiles, then a grid of cards. */
function Tiles({ month, onMonth, category, onCategory }: Choice) {
  const card = "flex flex-col gap-3 rounded-panel bg-surface p-4 shadow-panel";
  return (
    <div className="mx-auto flex w-full max-w-[1280px] flex-col gap-4">
      <div className="grid grid-cols-4 gap-4">
        {[
          ["Einnahmen", euros(incomeTotal, true), `2025 bis Oktober: ${euros(before.income, true)}`],
          ["Ausgaben", euros(spendingTotal), `2025 bis Oktober: ${euros(before.spending)}`],
          ["Saldo", euros(incomeTotal - spendingTotal, true), "Januar bis Oktober"],
          ["Sparquote", `${Math.round(((incomeTotal - spendingTotal) / incomeTotal) * 100)} %`, "der Einnahmen"],
        ].map(([label, value, detail]) => (
          <div key={label} className={cn(card, "gap-0.5")}>
            <Figure label={label ?? ""} value={value ?? ""} detail={detail} />
          </div>
        ))}
      </div>
      <div className="grid grid-cols-2 gap-4">
        <div className={card}>
          <h2 className="text-heading font-semibold">Ein- und Ausgaben</h2>
          <MonthBars
            series={[
              { name: "Einnahmen", values: income, color: GRAPHIT.income },
              { name: "Ausgaben", values: spending, color: GRAPHIT.spending },
            ]}
            chosen={month}
            onChoose={onMonth}
          />
        </div>
        <div className={card}>
          <h2 className="text-heading font-semibold">Ausgaben nach Kategorie</h2>
          <RankBars parts={categories} color={GRAPHIT.category} chosen={category} onChoose={onCategory} limit={6} />
        </div>
        <div className={card}>
          <h2 className="text-heading font-semibold">Gehalt</h2>
          <MonthBars series={[{ name: "Netto", values: salary, color: GRAPHIT.income }]} missing height={120} />
        </div>
        <div className={card}>
          <h2 className="text-heading font-semibold">Kontostand</h2>
          <BalanceLine values={balances} labels={months} color={GRAPHIT.line} height={120} />
        </div>
      </div>
    </div>
  );
}
