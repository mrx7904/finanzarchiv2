// The navigation of the app (#44, knowledge/decision-views.md "How one moves"): one state without a routing package.
// It holds the view, the place inside each view, and a stack of the places a jump left. The sidebar changes the view
// and a view keeps its place; a jump opens its target in the target's own view and can be gone back; the arrow at the
// left of the path and Alt+Left go back.
import type { ReviewAt } from "./review/Review";
import type { SettingsPageId } from "./settings/Settings";

/** A filter a jump from a figure sets on Umsätze (#38): a period, a category, and the words of its mark. */
export type BookingJump = { from?: string; to?: string; category?: number; label: string };

/** The place inside each view that is built. A view that is not built yet has none and is not shown. */
export type Places = {
  review: ReviewAt;
  /** The document chosen in Dokumente, and whether its page "Bestellungen" is open. */
  documents: { focus: number | null; orders: boolean };
  /** In Konten: the account whose bookings are open, all of them (null), or none open (undefined); and a text a jump set. */
  accounts: { bookingsOf: { id: number; name: string } | null | undefined; text: string; recurring?: boolean; filter?: BookingJump };
  /** The year of Auswertungen, null for the newest. */
  evaluations: { year: number | null };
  /** Start has no place of its own. */
  start: Record<string, never>;
  /** Steuern keeps its year inside the view. */
  taxes: Record<string, never>;
  /** The page of Einstellungen. */
  settings: { page: SettingsPageId };
};

export type ViewId = keyof Places;

/** A view with its place. */
export type Spot = { [V in ViewId]: { view: V; place: Places[V] } }[ViewId];

export type Nav = { view: ViewId; places: Places; back: Spot[] };

export type NavAction =
  /** The sidebar: the view keeps its place, the way back of jumps is left. */
  | { type: "go"; view: ViewId }
  /** A move inside a view. */
  | { type: "set"; spot: Spot }
  /** A jump from a thing to the thing that belongs to it. */
  | { type: "jump"; to: Spot }
  | { type: "back" };

export const initialNav: Nav = {
  view: "documents",
  places: {
    review: { group: null, question: null },
    documents: { focus: null, orders: false },
    accounts: { bookingsOf: undefined, text: "" },
    settings: { page: "ordner" },
    evaluations: { year: null },
    start: {},
    taxes: {},
  },
  back: [],
};

function here(nav: Nav): Spot {
  return { view: nav.view, place: nav.places[nav.view] } as Spot;
}

function put(nav: Nav, spot: Spot): Nav {
  return { ...nav, view: spot.view, places: { ...nav.places, [spot.view]: spot.place } };
}

/** How many places a jump keeps to go back to. */
export const BACK_LIMIT = 30;

export function navigate(nav: Nav, action: NavAction): Nav {
  switch (action.type) {
    case "go":
      return { ...nav, view: action.view, back: [] };
    case "set":
      return { ...put(nav, action.spot), view: nav.view };
    case "jump":
      return { ...put(nav, action.to), back: [...nav.back, here(nav)].slice(-BACK_LIMIT) };
    case "back": {
      const to = nav.back.at(-1);
      if (!to) return nav;
      return { ...put(nav, to), back: nav.back.slice(0, -1) };
    }
  }
}

/**
 * The place a URL of the dev server names, for the check of the views (#51): `view` (documents, review, accounts,
 * settings), `orders` for Bestellungen, `bookings` (`all` or an account id) for Umsätze, `page` for Einstellungen.
 * Anything else keeps the start. The release never reads it.
 */
export function navFromUrl(nav: Nav, search: string): Nav {
  const params = new URLSearchParams(search);
  const view = params.get("view");
  if (view !== "documents" && view !== "review" && view !== "accounts" && view !== "settings" && view !== "evaluations" && view !== "start" && view !== "taxes") return nav;
  const places = { ...nav.places };
  if (view === "documents") places.documents = { focus: null, orders: params.has("orders") };
  if (view === "accounts") {
    const bookings = params.get("bookings");
    places.accounts = { bookingsOf: bookings === null ? undefined : bookings === "all" ? null : { id: Number(bookings), name: "Konto" }, text: "", recurring: params.has("recurring") };
  }
  if (view === "settings") places.settings = { page: (params.get("page") ?? "ordner") as Places["settings"]["page"] };
  return { ...nav, view, places };
}
