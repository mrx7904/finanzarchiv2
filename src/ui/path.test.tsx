import { isValidElement, type ReactElement, type ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { Menu } from "./Menu";
import { PathItems, collapsePath, type PathSegment } from "./Path";

type Props = { children?: ReactNode; onClick?: () => void; entries?: { label: string; onSelect: () => void }[]; "aria-current"?: string };

/** Every element in what a component returned, depth first. */
function elements(node: ReactNode): ReactElement<Props>[] {
  if (Array.isArray(node)) return node.flatMap(elements);
  if (!isValidElement<Props>(node)) return [];
  return [node, ...elements(node.props.children)];
}

function segments(count: number) {
  const calls: string[] = [];
  const path: PathSegment[] = Array.from({ length: count }, (_, i) => ({ label: `Seite ${i + 1}`, onSelect: vi.fn(() => calls.push(`Seite ${i + 1}`)) }));
  return { path, calls };
}

function rendered(path: PathSegment[], keep: number) {
  const all = elements(PathItems({ items: collapsePath(path, keep) }));
  return {
    buttons: all.filter((e) => e.type === "button"),
    current: all.filter((e) => e.props["aria-current"] === "page"),
    menus: all.filter((e) => e.type === Menu),
  };
}

describe("the path in the top bar", () => {
  it("shows one segment as the current page, like a title, without a button", () => {
    const { path } = segments(1);
    const { buttons, current } = rendered(path, 0);
    expect(buttons).toHaveLength(0);
    expect(current).toHaveLength(1);
    expect(current[0]?.type).toBe("span");
    expect(current[0]?.props.children).toBe("Seite 1");
  });

  it("makes the last segment the current page and not a button, every other one a button that leads back", () => {
    const { path, calls } = segments(3);
    const { buttons, current } = rendered(path, 1);
    expect(current.map((e) => e.props.children)).toEqual(["Seite 3"]);
    expect(buttons.map((e) => e.props.children)).toEqual(["Seite 1", "Seite 2"]);
    for (const button of buttons) button.props.onClick?.();
    expect(calls).toEqual(["Seite 1", "Seite 2"]);
    expect(path[2]?.onSelect).not.toHaveBeenCalled();
  });

  it("folds the middle into a menu that still leads to each folded page, and keeps the first and the current", () => {
    const { path, calls } = segments(6);
    const items = collapsePath(path, 1);
    expect(items.map((item) => item.kind)).toEqual(["page", "folded", "page", "current"]);
    const { buttons, current, menus } = rendered(path, 1);
    expect(buttons.map((e) => e.props.children)).toEqual(["Seite 1", "Seite 5"]);
    expect(current.map((e) => e.props.children)).toEqual(["Seite 6"]);
    const entries = menus[0]?.props.entries ?? [];
    expect(entries.map((entry) => entry.label)).toEqual(["Seite 2", "Seite 3", "Seite 4"]);
    for (const entry of entries) entry.onSelect();
    expect(calls).toEqual(["Seite 2", "Seite 3", "Seite 4"]);
  });

  it("keeps the first and the current segment even when nothing else fits", () => {
    const { path } = segments(6);
    expect(collapsePath(path, 0).map((item) => (item.kind === "folded" ? "…" : item.segment.label))).toEqual(["Seite 1", "…", "Seite 6"]);
    expect(collapsePath(path, 99).map((item) => item.kind)).toEqual(["page", "page", "page", "page", "page", "current"]);
    expect(collapsePath(segments(2).path, 0).map((item) => item.kind)).toEqual(["page", "current"]);
    expect(collapsePath([], 0)).toEqual([]);
  });
});
