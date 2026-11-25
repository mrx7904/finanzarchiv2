#!/usr/bin/env node
/**
 * Builds the installer of this system and uploads it to the GitHub release of its version
 * (knowledge/decision-local-builds.md, #08). No GitHub Actions: each system builds its own.
 *
 *   node scripts/release.mjs            check, build, print the installer and its size
 *   node scripts/release.mjs --upload   the same, then upload to the draft release vX.Y.Z
 *
 * Windows builds the NSIS installer, macOS the dmg for Apple silicon, signed ad hoc. The version
 * is read from src-tauri/tauri.conf.json, the one place it lives. A release is created as a
 * draft, the owner publishes it on GitHub. A second system uploads into the same draft.
 */
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const upload = process.argv.includes("--upload");

function run(command, args, options = {}) {
  console.log(`> ${command} ${args.join(" ")}`);
  // Only npm and npx are scripts on Windows and need the shell. Everything else runs directly, so an
  // argument with a space (the title of a release, the path of the installer) stays one argument.
  const shell = process.platform === "win32" && ["npm", "npx"].includes(command);
  const result = spawnSync(command, args, { cwd: ROOT, stdio: options.capture ? "pipe" : "inherit", shell, encoding: "utf8" });
  if (result.status !== 0 && !options.mayFail) {
    console.error(`${command} failed, nothing was released`);
    process.exit(1);
  }
  return result;
}

const version = JSON.parse(readFileSync(path.join(ROOT, "src-tauri", "tauri.conf.json"), "utf8")).version;
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error(`The version "${version}" in tauri.conf.json is no x.y.z`);
  process.exit(1);
}
const tag = `v${version}`;
const platform = { win32: "windows", darwin: "macos" }[process.platform];
if (!platform) {
  console.error(`No release for ${process.platform} yet, Linux comes later`);
  process.exit(1);
}

if (upload) {
  const dirty = run("git", ["status", "--porcelain"], { capture: true }).stdout.trim();
  if (dirty) {
    console.error("The working tree has changes. A release is built from a committed state only.");
    process.exit(1);
  }
}

run("node", [path.join("scripts", "check.mjs")]);
run("node", [path.join("scripts", "fetch-pdfium.mjs")]);
run("npx", ["tauri", "build"]);

const bundle = path.join(ROOT, "src-tauri", "target", "release", "bundle");
const [folder, extension] = platform === "windows" ? ["nsis", ".exe"] : ["dmg", ".dmg"];
const dir = path.join(bundle, folder);
const installers = existsSync(dir) ? readdirSync(dir).filter((name) => name.endsWith(extension) && name.includes(version)) : [];
if (installers.length !== 1) {
  console.error(`Expected one installer of ${version} in ${dir}, found ${installers.length}`);
  process.exit(1);
}
const installer = path.join(dir, installers[0]);
const megabytes = (statSync(installer).size / 1024 / 1024).toFixed(1);
console.log(`Installer: ${installer} (${megabytes} MB)`);

if (upload) {
  const commit = run("git", ["rev-parse", "HEAD"], { capture: true }).stdout.trim();
  const exists = run("gh", ["release", "view", tag], { capture: true, mayFail: true }).status === 0;
  if (!exists) {
    run("gh", ["release", "create", tag, "--draft", "--target", commit, "--title", `Finanzarchiv ${version}`, "--notes", `Finanzarchiv ${version}. Gebaut lokal auf ${platform === "windows" ? "Windows" : "macOS"}, ohne GitHub Actions.`]);
  }
  run("gh", ["release", "upload", tag, installer, "--clobber"]);
  console.log(`${installers[0]} liegt im Entwurf ${tag}. Veröffentlicht wird er auf GitHub von Hand.`);
}
