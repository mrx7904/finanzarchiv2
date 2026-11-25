import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const config = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")) as {
  app: { windows: { title: string }[]; security: { csp: string } };
};
const html = readFileSync("index.html", "utf8");

describe("the shell", () => {
  it("names the window the same in Tauri and in the page", () => {
    const pageTitle = html.match(/<title>([^<]+)<\/title>/)?.[1];
    expect(config.app.windows[0]?.title).toBe(pageTitle);
  });

  it("speaks German", () => {
    expect(html).toContain('<html lang="de">');
  });

  it("loads nothing from the network and allows no plugins, forms or base changes", () => {
    const csp = config.app.security.csp;
    expect(csp).toContain("default-src 'self'");
    expect(csp).toContain("object-src 'none'");
    expect(csp).toContain("base-uri 'none'");
    expect(csp).toContain("form-action 'none'");
    expect(csp).not.toMatch(/https?:\/\/(?![a-z]+\.localhost)/);
  });
});
