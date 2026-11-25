// How the quick search matches: folding case, accents and ß, so "strasse" finds "Straße" and "bucherei" finds
// "Bücherei". Apart from QuickSearch.tsx, so it is tested without a window.

/** Lower case without accents, ß as ss, so the owner can type without them. */
export function fold(text: string): string {
  return text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase().replace(/ß/g, "ss");
}

/**
 * The filter of cmdk: 1 for hits at word starts, .5 when a word hits inside a word, 0 when a word of the query hits
 * nowhere. An entry's value is its id, which is not searched; its label and detail come as keywords.
 */
export function matchScore(value: string, search: string, keywords: string[] = []): number {
  const haystack = fold((keywords.length > 0 ? keywords : [value]).join(" "));
  const words = fold(search).split(/\s+/).filter(Boolean);
  let score = 1;
  for (const word of words) {
    const at = haystack.indexOf(word);
    if (at < 0) return 0;
    if (at > 0 && /[\p{L}\p{N}]/u.test(haystack[at - 1] ?? "")) score = 0.5;
  }
  return score;
}
