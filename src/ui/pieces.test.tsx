// The pieces of #45, rendered as the window gets them.
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";
import { nextOption, Segmented, Switch } from "./Choice";
import { Consequences, focusFirst } from "./Dialog";
import { DenseRow, FilterMark, GroupHead } from "./List";
import { LoadingRows } from "./Loading";
import { ValueRow } from "./Values";

describe("the switch and the segmented choice", () => {
  it("name themselves and say their state for a screen reader", () => {
    const on = renderToStaticMarkup(<Switch label="Nachdenken" checked onChange={() => undefined} />);
    expect(on).toContain('role="switch"');
    expect(on).toContain('aria-checked="true"');
    expect(on).toMatch(/<label for="[^"]+"[^>]*>Nachdenken<\/label>/);
    const segments = renderToStaticMarkup(
      <Segmented label="Größe" value="m" onChange={() => undefined} options={[{ value: "s", label: "Klein" }, { value: "m", label: "Mittel" }, { value: "l", label: "Groß" }]} />,
    );
    expect(segments).toContain('role="radiogroup" aria-label="Größe"');
    expect(segments.match(/aria-checked="true"/g)).toHaveLength(1);
    // Only the chosen segment is reached by Tab, the arrows move inside.
    expect(segments.match(/tabindex="0"/g)).toHaveLength(1);
  });

  it("move with the arrow keys around the ends", () => {
    const options = [{ value: "s" }, { value: "m" }, { value: "l" }];
    expect(nextOption(options, "m", "ArrowRight")?.value).toBe("l");
    expect(nextOption(options, "l", "ArrowRight")?.value).toBe("s");
    expect(nextOption(options, "s", "ArrowLeft")?.value).toBe("l");
    expect(nextOption(options, "s", "Enter")).toBeUndefined();
  });
});

describe("the dense row and the head of a group", () => {
  it("is 40 px high, keeps its height with its actions, and the head stays at the top", () => {
    const row = renderToStaticMarkup(
      <DenseRow lead="03.11." title="Beispiel Markt" detail="Lebensmittel" figure="34,12 €" onSelect={() => undefined} actions={<Button kind="plain" size="small" icon="more" label="Mehr" />} />,
    );
    expect(row).toContain("h-(--dense-height)");
    expect(row).toContain('role="option"');
    // The actions are only hidden, never taken out, so the row does not change.
    expect(row).toMatch(/opacity-0[^"]*group-hover:opacity-100/);
    expect(renderToStaticMarkup(<GroupHead title="November 2026" figures="+2.594,40 €" />)).toContain("sticky top-0");
  });
});

describe("loading", () => {
  it("shows nothing within the first 200 ms", () => {
    expect(renderToStaticMarkup(<LoadingRows loading />)).toBe("");
  });
});

describe("a read value, a filter mark and what a dialog removes", () => {
  it("say where a value stands, how a filter goes, and what goes and stays", () => {
    const value = renderToStaticMarkup(<ValueRow name="Brutto" value="119,00 €" source="checked" amount />);
    expect(value).toContain("stimmt");
    expect(value).toContain("tabular-nums");
    expect(renderToStaticMarkup(<FilterMark label="Beispiel Markt" onClear={() => undefined} />)).toContain('aria-label="Filter „Beispiel Markt“ entfernen"');
    const listed = renderToStaticMarkup(<Consequences goes={["Das Dokument"]} stays={["Die Buchung"]} />);
    expect(listed).toContain("Geht");
    expect(listed).toContain("Bleibt");
  });
});

describe("the focus of a dialog", () => {
  function fake(nodes: { field?: boolean; buttons: string[] }) {
    const focus = vi.fn();
    const buttons = nodes.buttons.map((kind) => ({ dataset: { kind }, focus: () => focus(kind) }));
    const dialog = {
      querySelector: () => (nodes.field ? { focus: () => focus("field") } : null),
      querySelectorAll: () => buttons,
    };
    focusFirst(dialog as unknown as HTMLElement);
    return focus;
  }

  it("goes to the field, else to the safe action, never to the close button", () => {
    expect(fake({ field: true, buttons: ["secondary", "primary"] })).toHaveBeenCalledWith("field");
    expect(fake({ buttons: ["secondary", "danger"] })).toHaveBeenCalledWith("secondary");
    expect(fake({ buttons: ["primary"] })).toHaveBeenCalledWith("primary");
  });

  it("lets a Button be the trigger of a Menu", () => {
    const button = renderToStaticMarkup(<Button ref={() => undefined} popoverTarget="m" aria-haspopup="menu" kind="secondary">Importieren</Button>);
    expect(button.toLowerCase()).toContain('popovertarget="m"');
    expect(button).toContain('data-kind="secondary"');
  });
});
