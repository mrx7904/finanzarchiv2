// A stand-in for the core in the dev server only, opened with `?mock` (http://localhost:1420/?mock): the commands
// answer with invented data, so a view can be seen in a browser without the Tauri window and without touching the
// owner's archive. Loaded by main.tsx in the dev build only, so it is in no release. Every name, amount and IBAN here
// is invented (knowledge/decision-public-repository.md).

type Args = Record<string, unknown> | undefined;

const page = (title: string, lines: string[]) =>
  `data:image/svg+xml;utf8,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="595" height="842" viewBox="0 0 595 842"><rect width="595" height="842" fill="#fff"/><text x="48" y="80" font-family="Segoe UI, sans-serif" font-size="22" font-weight="600">${title}</text>${lines
      .map((line, at) => `<text x="48" y="${140 + at * 26}" font-family="Segoe UI, sans-serif" font-size="13" fill="#333">${line}</text>`)
      .join("")}</svg>`,
  )}`;

const documents = [
  { id: 1, name: "Rechnung Beispiel Musik 2026-03", kind: "invoice", state: "filed", importedAt: new Date(Date.now() - 86400000).toISOString(), originalName: "rechnung-0815.pdf" },
  { id: 2, name: "Quittung Beispiel Markt 2026-03-11", kind: "receipt", state: "waiting", importedAt: "2026-03-11T17:20:00Z", originalName: "scan_0031.pdf" },
  { id: 3, name: "Kontoauszug Girokonto 2026-02", kind: "statement", state: "filed", importedAt: "2026-03-02T09:00:00Z", originalName: "auszug_02_2026.pdf" },
  { id: 4, name: "Rechnung Stadtwerke Musterstadt 2026-02", kind: "invoice", state: "filed", importedAt: "2026-02-20T10:00:00Z", originalName: "SW-2026-0413.pdf" },
  { id: 5, name: "Gehaltsabrechnung 2026-02", kind: "payslip", state: "open", importedAt: "2026-03-01T07:00:00Z", originalName: "lohn_02.pdf" },
].map((row) => ({ ...row, fileType: "pdf", pages: 1, textState: "text", nameByOwner: false, arrival: null, paired: null }));

const booking = (id: number, day: string, party: string, purpose: string, amount: string, incoming: boolean) => ({
  id, account: "Girokonto", day, party, otherSide: party, purpose, kind: incoming ? "Gutschrift" : "Lastschrift", incoming, amount, category: null, categoryByParty: false, documents: [], movement: null, statement: 3, page: 1,
});
const bookings = [
  booking(1, "2026-03-28", "Beispiel Software GmbH", "Gehalt März", "2.594,40 €", true),
  booking(2, "2026-03-25", "Beispiel Markt", "Lebensmittel Filiale 12", "34,12 €", false),
  booking(3, "2026-03-21", "Beispiel Musik", "Abo März", "11,99 €", false),
  booking(4, "2026-03-03", "Wohnbau Musterstadt eG", "Miete März Wohnung 3", "780,00 €", false),
  booking(5, "2026-02-12", "Stadtwerke Musterstadt", "Abschlag Strom", "85,00 €", false),
  booking(6, "2026-02-03", "Beispiel Markt", "Lebensmittel Filiale 12", "88,12 €", false),
];

const orders = [
  { id: 1, shop: "Amazon", number: "123-1234567-1234567", day: "2026-03-01", total: "49,95 €", invoice: 1, invoiceName: "Rechnung Beispiel Musik 2026-03", noInvoice: false },
  { id: 2, shop: "Amazon", number: "123-7654321-7654321", day: "2026-02-14", total: "19,99 €", invoice: null, invoiceName: null, noInvoice: false },
  { id: 3, shop: "Amazon", number: "123-1111111-2222222", day: "2026-01-09", total: "12,49 €", invoice: null, invoiceName: null, noInvoice: true },
];

const answers: Record<string, (args: Args) => unknown> = {
  app_status: () => ({ version: "0.0.0-dev", archive: { state: "ready" } }),
  review_groups: () => [{ key: "check:agree", label: "Werte stimmen nicht überein", count: 1 }, { key: "check:kind", label: "Art unklar", count: 1 }],
  review_questions: (args) =>
    args?.key === "check:kind"
      ? [{ document: 5, key: "check:kind", name: "Gehaltsabrechnung 2026-02", text: "Was ist das für ein Dokument?", page: 1, readings: [] }]
      : [{ document: 2, key: "check:agree", name: "Quittung Beispiel Markt 2026-03-11", text: "Ist das Brutto 34,12 €?", page: 1, readings: [{ reader: "Code", printed: "34,12", shown: "34,12 €", mark: [0.08, 0.46, 0.5, 0.03] }, { reader: "Modell", printed: "34,21", shown: "34,21 €", mark: null }] }],
  intake_overview: () => ({ paused: false, open: 2, running: [{ id: 9, name: "Rechnung_Nov.pdf", stage: "text", error: null }, { id: 10, name: "Scan_0032.pdf", stage: "queued", error: null }], failed: [], addedToday: 3 }),
  list_documents: () => documents,
  page_image: (args) => {
    const row = documents.find((document) => document.id === args?.document);
    return page(row?.name ?? "Dokument", ["Beispiel GmbH · Musterstraße 1 · 12345 Musterstadt", "", "Pos. 1  Gitarrensaiten          10,00 €", "Pos. 2  Versand                 0,00 €", "", "Netto                           100,00 €", "Umsatzsteuer 19 %                19,00 €", "Brutto                          119,00 €"]);
  },
  document_detail: (args) => ({
    values:
      args?.document === 3
        ? []
        : [
            { field: "sender", name: "Absender", value: "Beispiel Musik", amount: false, source: "read" },
            { field: "invoice_date", name: "Datum", value: "02.03.2026", amount: false, source: "read" },
            { field: "net", name: "Netto", value: "100,00 €", amount: true, source: "sender" },
            { field: "gross", name: "Brutto", value: "119,00 €", amount: true, source: "checked" },
          ],
    paidBy: args?.document === 1 ? [{ booking: 7, account: 1, accountName: "Girokonto", day: "2026-03-05", amount: "119,00 €", incoming: false }] : [],
  }),
  document_checks: () => ({ state: "filed", rule: "file.checks:1", decidedAt: "2026-03-04T08:01:00Z", checks: [{ check: "check:sum", field: "gross", passed: true, detail: "Netto und Umsatzsteuer ergeben das Brutto." }] }),
  list_orders: () => orders,
  order_invoice_candidates: () => [{ document: 4, name: "Rechnung Stadtwerke Musterstadt 2026-02", gross: "85,00 €", day: "2026-02-18" }],
  list_accounts: () => [{ id: 1, name: "Girokonto", iban: "DE89370400440532013000", balance: "2.345,67 €", bankBalance: "2.345,67 €", bankDay: "2026-02-28", findings: [] }],
  list_bookings: () => ({ rows: bookings, total: bookings.length, income: "2.594,40 €", spending: "999,23 €" }),
  booking_detail: (args) => {
    const row = bookings.find((booking) => booking.id === args?.booking) ?? bookings[0];
    return { ...row, valueDay: null, candidates: [], partyYear: { party: row?.party ?? "", year: 2026, count: 3, sum: row?.amount ?? "", incoming: row?.incoming ?? false, latest: [] }, counterpart: null, statement: { document: 3, name: "Kontoauszug Girokonto 2026-02", page: 1 } };
  },
  balances_by_month: () => ({ accounts: [], total: ["2026-01-31", "2026-02-28", "2026-03-31", "2026-04-30", "2026-05-31", "2026-06-30", "2026-07-31", "2026-08-31", "2026-09-30", "2026-10-31"].map((day, at) => ({ day, cents: 412300 + at * 91000, text: "", byBank: false, differenceText: null })) }),
  evaluation_years: () => [2026, 2025],
  evaluation_year: (args) => {
    const year = Number(args?.year ?? 2026);
    const sum = (cents: number, count = 1) => ({ cents, text: `${(cents / 100).toFixed(2)} €`, count });
    const income = Array.from({ length: 12 }, (_, at) => (year === 2026 && at > 9 ? sum(0, 0) : sum(259440 + (at === 4 ? 19460 : 0))));
    const spending = Array.from({ length: 12 }, (_, at) => (year === 2026 && at > 9 ? sum(0, 0) : sum([182310, 141220, 168900, 152040, 201330, 139870, 233110, 147650, 158220, 149990, 151000, 210000][at] ?? 0)));
    const total = (rows: { cents: number; count: number }[]) => sum(rows.reduce((s, r) => s + r.cents, 0), rows.reduce((s, r) => s + r.count, 0));
    const part = (id: number | null, name: string | null, cents: number) => ({ id, name, income: false, sum: sum(cents, 3) });
    return {
      year, income, spending, incomeTotal: total(income), spendingTotal: total(spending), balance: sum(0), parties: [], previousIncome: sum(0), previousSpending: sum(0),
      categories: [part(1, "Miete und Wohnen", 780000), part(2, "Lebensmittel", 341220), part(3, "Auto und Tanken", 182040), part(4, "Restaurant und Café", 98760), part(null, null, 22950)],
    };
  },
  salary_years: () => [{ year: 2026, missing: 1, totals: [], months: Array.from({ length: 10 }, (_, at) => ({ month: `2026-${String(at + 1).padStart(2, "0")}`, missing: at === 2, payslips: [], amounts: at === 2 ? [] : [{ field: "netto", label: "Netto", cents: 259440, text: "2.594,40 €" }], paidText: null, differenceText: null })) }],
  list_recurring: () => [
    { party: 11, name: "Beispiel Musik", incoming: false, rhythm: "monatlich", since: "2025-04-05", last: "2026-03-05", next: "2026-04-05", count: 12, cents: -1199, text: "11,99 €", perYearCents: -14388, perYearText: "-143,88 €", change: "seit August 2025 2,00 € mehr", ended: false, category: "Abos und Medien", bookings: [21, 22], history: ["2025-04-05", "2025-05-05", "2025-06-05", "2025-07-05", "2025-08-05", "2025-09-05", "2025-10-05", "2025-11-05", "2025-12-05", "2026-01-05", "2026-02-05", "2026-03-05"].map((day, at) => ({ day, cents: at < 4 ? -999 : -1199, text: at < 4 ? "9,99 €" : "11,99 €" })) },
    { party: 12, name: "Wohnbau Musterstadt eG", incoming: false, rhythm: "monatlich", since: "2024-01-03", last: "2026-03-03", next: "2026-04-03", count: 27, cents: -78000, text: "780,00 €", perYearCents: -936000, perYearText: "-9.360,00 €", change: null, ended: false, category: "Miete und Wohnen", bookings: [31], history: [{ day: "2026-03-03", cents: -78000, text: "780,00 €" }] },
    { party: 13, name: "Beispiel Film", incoming: false, rhythm: "monatlich", since: "2025-01-20", last: "2025-06-20", next: null, count: 6, cents: -1399, text: "13,99 €", perYearCents: -16788, perYearText: "-167,88 €", change: null, ended: true, category: "Abos und Medien", bookings: [41], history: [] },
  ],
  list_hints: () => [],
  tax_years: () => [2026],
  tax_categories: () => [{ key: "arbeitsmittel", label: "Arbeitsmittel", group: "Werbungskosten", elster: "Anlage N · Werbungskosten · Arbeitsmittel" }, { key: "fortbildung", label: "Fortbildung und Studium", group: "Werbungskosten", elster: "Anlage N · Werbungskosten · Fortbildungskosten" }],
  tax_year: (args) => {
    const amount = (cents: number) => ({ cents, text: `${(cents / 100).toFixed(2).replace(".", ",")} €` });
    return {
      year: args?.year ?? 2026, notice: "Keine Steuerberatung, bitte alles selbst prüfen.", valuesSource: "Steueränderungsgesetz 2025, Stand 31.10.2026",
      profile: { commuteKm: 18, commuteDays: 150, ownCar: true, employerCommuteCents: 12000, homeofficeDays: 60 },
      sections: [
        { category: { key: "fortbildung", label: "Fortbildung und Studium", group: "Werbungskosten", elster: "Anlage N · Werbungskosten · Fortbildungskosten" }, counted: amount(100000), entries: [{ document: 1, booking: null, day: "2026-01-08", gross: amount(300000), counted: amount(100000), notes: [] }] },
        { category: { key: "arbeitsmittel", label: "Arbeitsmittel", group: "Werbungskosten", elster: "Anlage N · Werbungskosten · Arbeitsmittel" }, counted: amount(8990), entries: [{ document: 4, booking: null, day: "2026-03-01", gross: amount(8990), counted: amount(8990), notes: ["Jahr nach Rechnungsdatum, keine Zahlung gefunden"] }] },
      ],
      entfernungspauschale: amount(90600), homeoffice: amount(36000), werbungskosten: amount(235590), pauschbetrag: amount(123000), abovePauschbetrag: amount(112590), erstausbildung: amount(0), haushaltReduction: amount(0), handwerkerReduction: amount(0),
    };
  },
  set_tax_profile: () => null,
  list_chats: () => [{ id: 1, title: "Was habe ich im März ausgegeben?", createdAt: "2026-03-30T10:00:00Z" }],
  chat_messages: () => [
    { id: 1, role: "owner", text: "Was habe ich im März ausgegeben?", sources: [], askedAt: "2026-03-30T10:00:00Z" },
    { id: 2, role: "archive", text: "Im März 2026 hast du 1.689,00 € ausgegeben, am meisten für Miete und Wohnen mit 780,00 €.", sources: [{ document: 1, booking: null }, { document: null, booking: 4 }], askedAt: "2026-03-30T10:00:05Z" },
  ],
  assign_tax: () => null,
  unassign_tax: () => null,
  rename_account: () => null,
  list_folders: () => [{ id: 1, path: "D:\\Beispiel\\Eingang", isInbox: true, problem: null }],
  downloads_state: () => ({ on: false, taken: 0, leftOut: 0, lying: 0 }),
  list_mailboxes: () => [],
  list_categories: () => [{ id: 1, name: "Lebensmittel", direction: "out" }],
  name_templates: () => ({ templates: [{ kind: "invoice", template: "", defaultTemplate: "Rechnung {absender} {monat}" }], placeholders: [{ name: "absender", meaning: "der Absender" }] }),
  archive_info: () => ({ folder: "C:\\Beispiel\\Finanzarchiv", documents: 5, bookings: 120, accounts: 1, snapshots: [{ name: "finanzarchiv-20260302T080000Z-taeglich.db", takenAt: "2026-03-02T08:00:00Z", reason: "täglich" }] }),
  model_status: () => ({ name: "gemma4:12b-it-qat", address: "http://127.0.0.1:11434", reached: true, installed: true, problem: null }),
};

/** `?mock=empty`, `?mock=error` or `?mock=slow`: the lists answer empty, with an error, or after 1.5 s (#51). */
const mode = new URLSearchParams(window.location.search).get("mock") ?? "";
const LISTS = ["list_documents", "list_accounts", "list_orders", "review_groups", "list_bookings"];

let callbacks = 0;
const internals = {
  invoke: async (command: string, args: Args) => {
    if (command.startsWith("plugin:event|")) return 0;
    if (LISTS.includes(command)) {
      if (mode === "empty") return command === "list_bookings" ? { rows: [], total: 0, income: "0,00 €", spending: "0,00 €" } : [];
      if (mode === "error") throw { kind: "internal", message: "Die Liste ließ sich nicht laden." };
      if (mode === "slow") await new Promise((resolve) => setTimeout(resolve, 1500));
    }
    const answer = answers[command];
    if (!answer) throw { kind: "internal", message: `Im Mock gibt es ${command} nicht.` };
    return answer(args);
  },
  transformCallback: () => (callbacks += 1),
  unregisterCallback: () => undefined,
  metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
};

(window as unknown as { __TAURI_INTERNALS__: typeof internals }).__TAURI_INTERNALS__ = internals;
(window as unknown as { __TAURI_EVENT_PLUGIN_INTERNALS__: object }).__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => undefined };

export {};
