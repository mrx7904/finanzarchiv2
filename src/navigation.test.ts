import { describe, expect, it } from "vitest";
import { BACK_LIMIT, initialNav, navigate, type Nav } from "./navigation";

const account = { id: 3, name: "Girokonto" };

function inAccounts(): Nav {
  let nav = navigate(initialNav, { type: "go", view: "accounts" });
  nav = navigate(nav, { type: "set", spot: { view: "accounts", place: { bookingsOf: account, text: "" } } });
  return nav;
}

describe("the navigation", () => {
  it("keeps the place of a view when the owner comes back by the sidebar", () => {
    let nav = inAccounts();
    nav = navigate(nav, { type: "go", view: "documents" });
    expect(nav.view).toBe("documents");
    nav = navigate(nav, { type: "go", view: "accounts" });
    expect(nav.places.accounts.bookingsOf).toEqual(account);
  });

  it("opens a jump in the target's view and goes back to where it came from", () => {
    let nav = inAccounts();
    nav = navigate(nav, { type: "jump", to: { view: "documents", place: { focus: 42 } } });
    expect(nav.view).toBe("documents");
    expect(nav.places.documents.focus).toBe(42);
    expect(nav.back).toHaveLength(1);
    nav = navigate(nav, { type: "back" });
    expect(nav.view).toBe("accounts");
    expect(nav.places.accounts.bookingsOf).toEqual(account);
    expect(nav.back).toHaveLength(0);
    // With the stack empty, back does nothing.
    expect(navigate(nav, { type: "back" })).toBe(nav);
  });

  it("goes back from a jump from a jump one place at a time", () => {
    let nav = inAccounts();
    nav = navigate(nav, { type: "jump", to: { view: "documents", place: { focus: 42 } } });
    nav = navigate(nav, { type: "jump", to: { view: "documents", place: { focus: 7 } } });
    expect(nav.back.map((spot) => spot.view)).toEqual(["accounts", "documents"]);
    nav = navigate(nav, { type: "back" });
    expect(nav.places.documents.focus).toBe(42);
    nav = navigate(nav, { type: "back" });
    expect(nav.view).toBe("accounts");
  });

  it("leaves the way back when the owner changes the view by the sidebar", () => {
    let nav = inAccounts();
    nav = navigate(nav, { type: "jump", to: { view: "documents", place: { focus: 42 } } });
    nav = navigate(nav, { type: "go", view: "review" });
    expect(nav.back).toHaveLength(0);
  });

  it("moves inside a view without a way back and keeps at most a limit of places", () => {
    let nav = navigate(initialNav, { type: "set", spot: { view: "documents", place: { focus: 1 } } });
    expect(nav.back).toHaveLength(0);
    for (let at = 0; at < BACK_LIMIT + 5; at += 1) nav = navigate(nav, { type: "jump", to: { view: "documents", place: { focus: at } } });
    expect(nav.back).toHaveLength(BACK_LIMIT);
  });
});
