/** Where the owner is in "Prüfen": a list with a chosen row, or one question, maybe with its readings open. */
export type ReviewPlace = { count: number; index: number; open: boolean; readings: number };

export type ReviewAction = { kind: "move"; index: number } | { kind: "open" } | { kind: "back" } | { kind: "pick"; reading: number } | { kind: "none" };

/** What a key does in "Prüfen" (#17): J and K move, Enter opens, 1 and 2 pick a reading, Escape goes back. */
export function reviewKey(key: string, place: ReviewPlace): ReviewAction {
  if (key === "Escape") return { kind: "back" };
  if (place.open) {
    const reading = Number.parseInt(key, 10) - 1;
    if (Number.isInteger(reading) && reading >= 0 && reading < Math.min(place.readings, 9)) return { kind: "pick", reading };
    return { kind: "none" };
  }
  if (place.count === 0) return { kind: "none" };
  switch (key.toLowerCase()) {
    case "j":
      return { kind: "move", index: Math.min(place.index + 1, place.count - 1) };
    case "k":
      return { kind: "move", index: Math.max(place.index - 1, 0) };
    case "enter":
      return { kind: "open" };
    default:
      return { kind: "none" };
  }
}

/** The path in the top bar: "Prüfen › <group> › <document>", each earlier segment leads back. */
export function reviewPath(group: string | null, document: string | null, back: { toStart: () => void; toGroup: () => void }) {
  const path: { label: string; onSelect?: () => void }[] = [{ label: "Prüfen", onSelect: group ? back.toStart : undefined }];
  if (group) path.push({ label: group, onSelect: document ? back.toGroup : undefined });
  if (document) path.push({ label: document });
  return path;
}

/** The key of "Der Reihe nach prüfen": every question of every reason, as one group (#47). */
export const SEQUENCE = "*";

/**
 * The questions in the order "Der Reihe nach prüfen" goes: the reasons in their order, inside a reason its documents
 * in theirs, each document once (a document waiting for two reasons is asked where it comes first).
 */
export function inSequence<Q extends { document: number }>(groups: { key: string }[], byKey: Record<string, Q[]>): Q[] {
  const seen = new Set<number>();
  const order: Q[] = [];
  for (const group of groups) {
    for (const question of byKey[group.key] ?? []) {
      if (seen.has(question.document)) continue;
      seen.add(question.document);
      order.push(question);
    }
  }
  return order;
}

/** "Frage n von m" in the top bar. */
export function questionLine(index: number, count: number): string {
  return `Frage ${index + 1} von ${count}`;
}
