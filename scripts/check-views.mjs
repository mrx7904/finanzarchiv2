// The check of the views (#51, knowledge/decision-views.md): every view and sub page of the app, with the stand-in of
// the core (src/dev/mock.ts, invented data), in light and dark at 960 × 600, 1440 × 900 and 2560 × 1440, and every
// list empty, failing and loading. It fails when something runs out of the window sideways, a control is covered, the
// content starts below a quarter of the height at 1440 × 900 (rule 4), the top bar holds more than one primary button
// (rule 2), or a state word is not one of rule 11. `--self-test` plants one fault per rule and fails unless each is
// found. It drives the Edge or Chrome of the system over the DevTools protocol, so no package is added.
//
//   npm run check:views
//   npm run check:views -- --self-test
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { createServer } from "vite";

const SIZES = [
  [960, 600],
  [1440, 900],
  [2560, 1440],
];
const THEMES = ["light", "dark"];
const VIEWS = [
  "view=documents",
  "view=documents&orders",
  "view=review",
  "view=accounts",
  "view=accounts&bookings=1",
  "view=accounts&bookings=all",
  "view=accounts&recurring",
  "view=evaluations",
  ...["ordner", "postfaecher", "namen", "kategorien", "modell", "archiv", "darstellung", "ueber"].map((page) => `view=settings&page=${page}`),
];
const LISTS = ["view=documents", "view=documents&orders", "view=review", "view=accounts", "view=accounts&bookings=all"];
const STATES = ["empty", "error", "slow"];
/** The words of rule 11, "ohne Bank" that #48 added for an account without a balance of its bank, "gestiegen" of #52. */
const WORDS = ["prüfen", "doppelt?", "fehlt", "bestätigt", "stimmt", "vollständig", "gelesen", "läuft", "beendet", "ohne Bank", "gestiegen"];

/** Runs in the page: every rule, each finding one sentence. */
const RULES = `(() => {
  const out = [];
  const W = innerWidth, H = innerHeight;
  const content = document.querySelector("[data-content]");
  if (document.documentElement.scrollWidth > W + 1) out.push("runs out sideways: the page is " + document.documentElement.scrollWidth + " px wide");
  for (const el of content ? content.querySelectorAll("*") : []) {
    const r = el.getBoundingClientRect();
    if (r.width > 0 && r.height > 0 && r.right > W + 1) { out.push("runs out sideways: " + el.tagName.toLowerCase() + " ends at " + Math.round(r.right) + " px"); break; }
  }
  const clipped = (el, x, y) => {
    for (let p = el.parentElement; p; p = p.parentElement) {
      const s = getComputedStyle(p);
      if (/(auto|scroll|hidden)/.test(s.overflow + s.overflowX + s.overflowY)) {
        const q = p.getBoundingClientRect();
        if (y < q.top || y > q.bottom || x < q.left || x > q.right) return true;
      }
    }
    return false;
  };
  const underSticky = (hit) => { for (let p = hit; p; p = p.parentElement) if (getComputedStyle(p).position === "sticky") return true; return false; };
  for (const el of document.querySelectorAll("button, [role=option], input, select, a[href]")) {
    const r = el.getBoundingClientRect();
    if (r.width < 4 || r.height < 4) continue;
    const x = r.left + r.width / 2, y = r.top + r.height / 2;
    if (x < 0 || y < 0 || x >= W || y >= H || clipped(el, x, y)) continue;
    const hit = document.elementFromPoint(x, y);
    if (hit && !el.contains(hit) && !hit.contains(el) && !underSticky(hit)) out.push("covered: " + (el.getAttribute("aria-label") || el.textContent || el.tagName).trim().slice(0, 40));
  }
  // An empty state is the one thing that stands in the middle (rule 4).
  if (W === 1440 && H === 900 && content && !content.querySelector("[data-empty]")) {
    const first = [...content.querySelectorAll("section, nav, form, .rounded-panel, [role=listbox], p, h2, [role=status]")].find((e) => e.getBoundingClientRect().height > 0);
    if (first && first.getBoundingClientRect().top > H / 4) out.push("the content starts at " + Math.round(first.getBoundingClientRect().top) + " px");
  }
  const header = document.querySelector("header");
  const primaries = header ? header.querySelectorAll("button[data-kind=primary]").length : 0;
  if (primaries > 1) out.push(primaries + " primary buttons in the top bar");
  const words = ${JSON.stringify(WORDS)};
  for (const capsule of document.querySelectorAll("[data-state]")) {
    const word = capsule.textContent.trim();
    if (!words.includes(word)) out.push("a state word outside rule 11: " + word);
  }
  return out;
})()`;

/** One fault per rule, planted into the page of Dokumente for the self-test. */
const PLANTS = {
  sideways: `document.querySelector("[data-content]").append(Object.assign(document.createElement("div"), { style: "width:4000px;height:10px" }))`,
  covered: `(() => { const b = [...document.querySelectorAll("[data-content] button, [data-content] [role=option]")].find((e) => e.getBoundingClientRect().width > 8); const r = b.getBoundingClientRect(); document.body.append(Object.assign(document.createElement("div"), { style: "position:fixed;z-index:50;left:" + r.left + "px;top:" + r.top + "px;width:" + r.width + "px;height:" + r.height + "px;background:red" })); })()`,
  late: `document.querySelector("[data-content]").prepend(Object.assign(document.createElement("div"), { style: "height:400px;flex:none" }))`,
  primary: `(() => { const h = document.querySelector("header"); for (let i = 0; i < 2; i += 1) { const b = document.createElement("button"); b.dataset.kind = "primary"; b.textContent = "Mehr"; h.append(b); } })()`,
  word: `(() => { const s = document.createElement("span"); s.dataset.state = ""; s.textContent = "erledigt"; document.querySelector("[data-content]").append(s); })()`,
};

function browserPath() {
  const candidates = [
    process.env.CHECK_VIEWS_BROWSER,
    "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
    "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
    "C:/Program Files/Google/Chrome/Application/chrome.exe",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium",
  ].filter(Boolean);
  return candidates.find((candidate) => existsSync(candidate));
}

/** A small client of the DevTools protocol over the WebSocket of Node. */
async function connect(url) {
  const socket = new WebSocket(url);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  let next = 0;
  const waiting = new Map();
  const listeners = new Set();
  socket.addEventListener("message", (event) => {
    const message = JSON.parse(String(event.data));
    if (message.id && waiting.has(message.id)) {
      const { resolve, reject } = waiting.get(message.id);
      waiting.delete(message.id);
      if (message.error) reject(new Error(message.error.message));
      else resolve(message.result);
    } else for (const listener of listeners) listener(message);
  });
  return {
    send(method, params = {}) {
      next += 1;
      const id = next;
      socket.send(JSON.stringify({ id, method, params }));
      return new Promise((resolve, reject) => waiting.set(id, { resolve, reject }));
    },
    once(method) {
      return new Promise((resolve) => {
        const listener = (message) => {
          if (message.method === method) {
            listeners.delete(listener);
            resolve(message.params);
          }
        };
        listeners.add(listener);
      });
    },
    close: () => socket.close(),
  };
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  const selfTest = process.argv.includes("--self-test");
  const executable = browserPath();
  if (!executable) {
    console.error("Kein Edge oder Chrome gefunden, CHECK_VIEWS_BROWSER setzen.");
    process.exit(1);
  }
  const server = await createServer({ server: { port: 1431, strictPort: true }, logLevel: "error" });
  await server.listen();
  const profile = mkdtempSync(path.join(tmpdir(), "check-views-"));
  const browser = spawn(executable, ["--headless=new", "--remote-debugging-port=9333", `--user-data-dir=${profile}`, "--no-first-run", "--hide-scrollbars", "about:blank"], { stdio: "ignore" });
  let page;
  const started = Date.now();
  const findings = [];
  try {
    let target = null;
    for (let tries = 0; tries < 50 && !target; tries += 1) {
      await sleep(200);
      try {
        const listed = await (await fetch("http://127.0.0.1:9333/json/list")).json();
        target = listed.find((entry) => entry.type === "page");
      } catch {
        // The browser is not listening yet.
      }
    }
    if (!target) throw new Error("Der Browser antwortet nicht.");
    page = await connect(target.webSocketDebuggerUrl);
    await page.send("Page.enable");
    const open = async (query, [width, height], theme, settle = 900) => {
      await page.send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
      await page.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: theme }] });
      const loaded = page.once("Page.loadEventFired");
      await page.send("Page.navigate", { url: `http://localhost:1431/?${query}` });
      await loaded;
      await sleep(settle);
    };
    const rules = async () => (await page.send("Runtime.evaluate", { expression: RULES, returnByValue: true })).result.value ?? ["the rules did not run"];

    if (selfTest) {
      for (const [rule, plant] of Object.entries(PLANTS)) {
        await open("mock&view=documents", [1440, 900], "light");
        await page.send("Runtime.evaluate", { expression: plant });
        const found = await rules();
        const ok = found.length > 0;
        console.log(`${ok ? "ok  " : "FAIL"} ${rule}${ok ? `: ${found[0]}` : " was not found"}`);
        if (!ok) findings.push(rule);
      }
    } else {
      for (const view of VIEWS) {
        for (const size of SIZES) {
          for (const theme of THEMES) {
            await open(`mock&${view}`, size, theme);
            for (const found of await rules()) findings.push(`${view} ${size.join("×")} ${theme}: ${found}`);
          }
        }
      }
      for (const view of LISTS) {
        for (const state of STATES) {
          await open(`mock=${state}&${view}`, [1440, 900], "light", state === "slow" ? 600 : 900);
          for (const found of await rules()) findings.push(`${view} ${state}: ${found}`);
        }
      }
    }
  } finally {
    page?.close();
    browser.kill();
    await server.close();
    await sleep(300);
    rmSync(profile, { recursive: true, force: true });
  }
  const seconds = Math.round((Date.now() - started) / 1000);
  for (const finding of findings) console.log(finding);
  console.log(`${findings.length} finding(s), ${seconds} s`);
  process.exit(findings.length > 0 ? 1 : 0);
}

await main();
