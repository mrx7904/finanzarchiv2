#!/usr/bin/env node
/**
 * The one check of the repository (knowledge/conventions.md, FA-02).
 *
 *   node scripts/check.mjs              exit 0 when every step passes, 1 otherwise
 *   node scripts/check.mjs --self-test  proves that each step can fail
 *
 * Steps: relative Markdown links, traceability between spec/requirements.md and issues/,
 * no personal data in a tracked file (FA-03), then cargo fmt, clippy and test in src-tauri/
 * and typecheck, lint and test of the UI (#01). The self-test plants one fault per step.
 * Nothing here needs GitHub Actions.
 */
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// ---------- files ----------

function trackedFiles(root) {
  if (existsSync(path.join(root, ".git"))) {
    const out = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "-z"], { cwd: root });
    return out.toString("utf8").split("\0").filter((f) => f && existsSync(path.join(root, f)));
  }
  return walk(root, root);
}

function walk(dir, root) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if ([".git", "node_modules", "target"].includes(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) found.push(...walk(full, root));
    else found.push(path.relative(root, full).split(path.sep).join("/"));
  }
  return found;
}

function readText(root, file) {
  const bytes = readFileSync(path.join(root, file));
  return bytes.includes(0) ? null : bytes.toString("utf8");
}

/** The text without fenced blocks and inline code, so a path in code is not a link. */
function prose(text) {
  return text.replace(/```[\s\S]*?```/g, (block) => block.replace(/[^\n]/g, " ")).replace(/`[^`\n]*`/g, "``");
}

// ---------- step: links ----------

function checkLinks(root, files) {
  const errors = [];
  for (const file of files.filter((f) => f.endsWith(".md"))) {
    const text = prose(readText(root, file) ?? "");
    for (const match of text.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = match[1];
      if (/^[a-z]+:/i.test(target) || target.startsWith("#")) continue;
      const clean = decodeURIComponent(target.split("#")[0].replace(/:\d+$/, ""));
      if (!existsSync(path.resolve(root, path.dirname(file), clean))) errors.push(`${file}: broken link ${target}`);
    }
  }
  return errors;
}

// ---------- step: traceability ----------

function readRequirements(root) {
  const file = path.join(root, "spec", "requirements.md");
  if (!existsSync(file)) return null;
  const requirements = new Map();
  let current = null;
  for (const line of readFileSync(file, "utf8").split(/\r?\n/)) {
    const heading = line.match(/^### (FA-\d+)\b/);
    if (heading) {
      current = heading[1];
      requirements.set(current, []);
      continue;
    }
    const issue = line.match(/^Issue:\s*(.+)$/);
    if (issue && current) requirements.get(current).push(...(issue[1].match(/#\d+/g) ?? []));
  }
  return requirements;
}

function readIssues(root) {
  const dir = path.join(root, "issues");
  const issues = new Map();
  if (!existsSync(dir)) return issues;
  for (const name of readdirSync(dir)) {
    const number = name.match(/^(\d{3})-.+\.md$/);
    if (!number) continue;
    const text = readFileSync(path.join(dir, name), "utf8");
    const line = text.match(/\*\*Requirements\*\*:\s*([^·\n]+)/);
    const ids = line ? (line[1].match(/FA-\d+/g) ?? []) : [];
    issues.set(`#${number[1].slice(1)}`, { name, ids });
  }
  return issues;
}

function checkTraceability(root) {
  const requirements = readRequirements(root);
  if (!requirements) return [];
  const issues = readIssues(root);
  const indexFile = path.join(root, "issues", "README.md");
  const index = existsSync(indexFile) ? readFileSync(indexFile, "utf8") : "";
  const errors = [];
  for (const [id, named] of requirements) {
    if (named.length === 0) errors.push(`${id} names no issue`);
    for (const issue of named) {
      if (!issues.has(issue)) errors.push(`${id} names ${issue}, which has no file`);
      else if (!issues.get(issue).ids.includes(id)) errors.push(`${id} names ${issue}, which does not list ${id}`);
    }
  }
  for (const [issue, { name, ids }] of issues) {
    if (ids.length === 0) errors.push(`${issue} (${name}) lists no requirement`);
    for (const id of ids) if (!requirements.has(id)) errors.push(`${issue} lists ${id}, which is not in requirements.md`);
    if (!index.includes(`(${name})`)) errors.push(`${issue} (${name}) is missing in issues/README.md`);
  }
  return errors;
}

// ---------- step: privacy (FA-03) ----------

const ALLOWED_MAIL = /@(users\.noreply\.github\.com|example\.(com|org|net))$|^noreply@anthropic\.com$/i;
const IMAGE_NAME = /@\d+x\.(png|jpe?g|webp|gif|svg)$/i;
const PATTERNS = [
  { what: "an IBAN", regex: /\b[A-Z]{2}\d{2}(?: ?[A-Z0-9]{4}){3,7}(?: ?[A-Z0-9]{1,3})?\b/g, allowed: (file) => file.startsWith("fixtures/synthetic/") },
  // An image name such as "128x128@2x.png" has the shape of an address and is none.
  { what: "a mail address", regex: /[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, allowed: (_file, hit) => ALLOWED_MAIL.test(hit) || IMAGE_NAME.test(hit) },
  { what: "a path into a home folder", regex: /\b[A-Za-z]:[\\/]+Users[\\/]+[^\\/\s"'`]+|\/(?:Users|home)\/[A-Za-z0-9._-]+/g, allowed: () => false },
];

function checkPrivacy(root, files) {
  const errors = [];
  for (const file of files) {
    if (file === "scripts/check.mjs") continue;
    const text = readText(root, file);
    if (text === null) continue;
    for (const { what, regex, allowed } of PATTERNS) {
      for (const hit of text.match(regex) ?? []) {
        if (!allowed(file, hit)) errors.push(`${file}: looks like ${what}`);
      }
    }
  }
  return errors;
}

// ---------- step: Rust and TypeScript (#01) ----------

const LIB_RS = "src-tauri/src/lib.rs";
const TS_PROBE = "src/check-probe.ts";

/**
 * Each step with the fault that must make it fail. A probe is appended to a file or written as a
 * new one, and the self-test restores the tree afterwards.
 */
const CODE_STEPS = [
  {
    name: "cargo fmt", needs: "src-tauri/Cargo.toml", cwd: "src-tauri", command: "cargo", args: ["fmt", "--check"],
    probe: { file: LIB_RS, append: "\nfn   badly_formatted( ){}\n" },
  },
  {
    name: "cargo clippy", needs: "src-tauri/Cargo.toml", cwd: "src-tauri", command: "cargo", args: ["clippy", "--all-targets", "--", "-D", "warnings"],
    probe: { file: LIB_RS, append: "\npub fn check_probe() -> u8 {\n    let value: Option<u8> = None;\n    value.unwrap()\n}\n" },
  },
  {
    name: "cargo test", needs: "src-tauri/Cargo.toml", cwd: "src-tauri", command: "cargo", args: ["test"],
    probe: { file: LIB_RS, append: "\n#[cfg(test)]\nmod check_probe {\n    #[test]\n    fn fails() {\n        assert_eq!(1 + 1, 3);\n    }\n}\n" },
  },
  {
    name: "typecheck", needs: "package.json", cwd: ".", command: "npm", args: ["run", "typecheck"],
    probe: { file: TS_PROBE, write: 'export const probe: number = "text";\n' },
  },
  {
    name: "lint", needs: "package.json", cwd: ".", command: "npm", args: ["run", "lint"],
    probe: { file: TS_PROBE, write: "export function probe(): void {\n  debugger;\n}\n" },
  },
  {
    name: "test", needs: "package.json", cwd: ".", command: "npm", args: ["test"],
    probe: { file: "src/check-probe.test.ts", write: 'import { expect, it } from "vitest";\n\nit("fails", () => {\n  expect(1 + 1).toBe(3);\n});\n' },
  },
];

function runStep(root, step, quiet = false) {
  const result = spawnSync(step.command, step.args, {
    cwd: path.join(root, step.cwd),
    stdio: quiet ? "ignore" : "inherit",
    shell: process.platform === "win32",
  });
  return result.status === 0 ? [] : [`${step.name} failed`];
}

function presentSteps(root) {
  const steps = CODE_STEPS.filter((step) => existsSync(path.join(root, step.needs)));
  // The tests of the core read PDFs through PDFium. Fetching is a no-op when it is there.
  if (steps.some((step) => step.cwd === "src-tauri") && existsSync(path.join(root, "scripts", "fetch-pdfium.mjs"))) {
    const fetched = spawnSync("node", [path.join("scripts", "fetch-pdfium.mjs")], { cwd: root, stdio: "inherit" });
    if (fetched.status !== 0) throw new Error("PDFium could not be fetched, the Rust tests cannot run");
  }
  return steps;
}

/** Runs one step with its probe in place and gives the tree back as it was. */
function failsWithProbe(root, step) {
  const file = path.join(root, step.probe.file);
  const before = existsSync(file) ? readFileSync(file) : null;
  try {
    writeFileSync(file, step.probe.append ? Buffer.concat([before ?? Buffer.alloc(0), Buffer.from(step.probe.append)]) : step.probe.write);
    return runStep(root, step, true).length > 0;
  } finally {
    if (before === null) rmSync(file, { force: true });
    else writeFileSync(file, before);
  }
}

// ---------- main ----------

function check(root) {
  const files = trackedFiles(root);
  const steps = [
    ["links", checkLinks(root, files)],
    ["traceability", checkTraceability(root)],
    ["privacy", checkPrivacy(root, files)],
  ];
  for (const step of presentSteps(root)) steps.push([step.name, runStep(root, step)]);
  let failed = 0;
  for (const [name, errors] of steps) {
    console.log(`${name}: ${errors.length === 0 ? "ok" : `${errors.length} error(s)`}`);
    for (const error of errors) console.log(`  ${error}`);
    failed += errors.length;
  }
  console.log(`${files.length} files checked, ${failed} error(s)`);
  return failed;
}

function selfTest() {
  const dir = mkdtempSync(path.join(tmpdir(), "fa2-check-"));
  const write = (file, text) => {
    mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
    writeFileSync(path.join(dir, file), text);
  };
  const results = [];
  const expect = (name, errors, shouldFail) => results.push([name, shouldFail ? errors.length > 0 : errors.length === 0]);
  try {
    write("spec/requirements.md", "### FA-01 One\n\nIssue: #01\n");
    write("issues/001-one.md", "# #01 One\n\n**Requirements**: FA-01 · **Depends on**: none\n");
    write("issues/README.md", "| [#01](001-one.md) | One |\n");
    write("README.md", "[ok](spec/requirements.md)\n");
    write("fixtures/synthetic/statement.csv", ["DE89", "3704", "0044", "0532", "0130", "00"].join(" ") + "\n");
    let files = walk(dir, dir);
    expect("links pass on a clean tree", checkLinks(dir, files), false);
    expect("traceability passes on a clean tree", checkTraceability(dir), false);
    expect("privacy passes on a clean tree", checkPrivacy(dir, files), false);

    write("docs/broken.md", "[gone](nothing-here.md)\n");
    write("issues/002-two.md", "# #02 Two\n\n**Requirements**: FA-09 · **Depends on**: none\n");
    write("notes.txt", ["DE89", "3704", "0044", "0532", "0130", "00"].join("") + " " + ["owner", "mailbox.de"].join("@") + " " + ["C:", "Users", "someone", "file"].join("\\") + "\n");
    files = walk(dir, dir);
    expect("links fail on a broken link", checkLinks(dir, files), true);
    expect("traceability fails on an unknown requirement", checkTraceability(dir), true);
    const privacy = checkPrivacy(dir, files);
    expect("privacy fails on an IBAN", privacy.filter((e) => e.includes("IBAN")), true);
    expect("privacy fails on a mail address", privacy.filter((e) => e.includes("mail")), true);
    expect("privacy fails on a home path", privacy.filter((e) => e.includes("home")), true);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
  // A step that fails on the clean tree would fail on its probe too and prove nothing, so both are asked.
  for (const step of presentSteps(ROOT)) {
    results.push([`${step.name} passes on the clean tree`, runStep(ROOT, step, true).length === 0]);
    results.push([`${step.name} fails on its probe`, failsWithProbe(ROOT, step)]);
  }
  for (const [name, ok] of results) console.log(`${ok ? "ok  " : "FAIL"} ${name}`);
  const failed = results.filter(([, ok]) => !ok).length;
  console.log(`${results.length} self-tests, ${failed} failed`);
  return failed;
}

process.exit((process.argv.includes("--self-test") ? selfTest() : check(ROOT)) === 0 ? 0 : 1);
