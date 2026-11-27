// The pages the window already has (#55). A page asked twice at one width is one call of the core: the second view of
// a page, a thumbnail shown in two places, or a component mounted again gets the answer of the first. A failed page is
// not kept, so the next view asks again.

type Answer = { status: "ok"; data: string } | { status: "error"; error: unknown };
type Ask = (document: number, page: number, width: number) => Promise<Answer>;

/** Enough for the pages of a few documents at two sizes, small enough that the window stays light. */
const KEPT = 48;
const pages = new Map<string, Promise<string | null>>();

/** The page as a data URL, or `null` when the core could not render it. */
export function loadPage(document: number, page: number, width: number, ask: Ask): Promise<string | null> {
  const key = `${document}:${page}:${width}`;
  const known = pages.get(key);
  if (known) {
    // Read again, so it is the last to go.
    pages.delete(key);
    pages.set(key, known);
    return known;
  }
  const loading = ask(document, page, width).then(
    (answer) => (answer.status === "ok" ? answer.data : null),
    () => null,
  );
  pages.set(key, loading);
  void loading.then((source) => {
    if (source === null) pages.delete(key);
  });
  while (pages.size > KEPT) {
    const oldest = pages.keys().next().value;
    if (oldest === undefined) break;
    pages.delete(oldest);
  }
  return loading;
}

/** For the tests: forget every page. */
export function forgetPages() {
  pages.clear();
}
