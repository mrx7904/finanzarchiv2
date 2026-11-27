// The navigation of the app (#44, knowledge/decision-views.md "How one moves"): one state without a routing package.
// It holds the view, the place inside each view, and a stack of the places a jump left. The sidebar changes the view
// and a view keeps its place; a jump opens its target in the target's own view and can be gone back; the arrow at the
// left of the path and Alt+Left go back.
import type { ReviewAt } from "./review/Review";
import type { SettingsPageId } from "./settings/Settings";

/** The place inside each view that is built. A view that is not built yet has none and is not shown. */
export type Places = {
  review: ReviewAt;
  /** The document chosen in Dokumente, and whether its page "Bestellungen" is open. */
  documents: { focus: number | null; orders: boolean };
  /** In Konten: the account whose bookings are open, all of them (null), or none open (undefined); and a text a jump set. */
  accounts: { bookingsOf: { id: number; name: string } | null | undefined; text: string };
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
