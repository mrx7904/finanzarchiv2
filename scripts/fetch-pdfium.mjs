#!/usr/bin/env node
/**
 * Fetches the PDFium library the core reads PDFs with (knowledge/decision-pdf.md).
 *
 *   node scripts/fetch-pdfium.mjs           the library of this system
 *   node scripts/fetch-pdfium.mjs --all     the libraries of every system we ship
 *
 * The prebuilt binaries of bblanchon/pdfium-binaries, release chromium/8125, the version
 * pdfium-render 0.9.6 is built against. Each archive is checked against its SHA-256 before it
 * is unpacked into src-tauri/pdfium/<platform>/, which git ignores. Nothing is fetched twice.
 */
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const RELEASE = "chromium/8125";
const BASE = `https://github.com/bblanchon/pdfium-binaries/releases/download/${RELEASE}`;
// Digests as GitHub reports them for the release assets, read on 2026-11-25.
const PLATFORMS = {
  "win-x64": { sha256: "9a964261760cff52570aabf038ab346d3a5034dd8b30d0d5fe8861235d356b8b", library: "bin/pdfium.dll" },
  "mac-arm64": { sha256: "4939bd935ac83ddc09a3a4f038bc43a19cea1660316a3667f26cb07a070e139d", library: "lib/libpdfium.dylib" },
  "mac-x64": { sha256: "8a5193535b80c7215d8932f6647ae5097465b6dba2811602c21e120c2af112b0", library: "lib/libpdfium.dylib" },
  "linux-x64": { sha256: "e1c3ce6e270cb5e0c20816b379ad9f2c1309972697c505f508de92bf5e7d6659", library: "lib/libpdfium.so" },
};

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const TARGET = path.join(ROOT, "src-tauri", "pdfium");

function currentPlatform() {
  const os = { win32: "win", darwin: "mac", linux: "linux" }[process.platform];
  const arch = { x64: "x64", arm64: "arm64" }[process.arch];
  const name = `${os}-${arch}`;
  if (!PLATFORMS[name]) throw new Error(`No PDFium for ${name}`);
  return name;
}

async function fetchPlatform(name) {
  const { sha256, library } = PLATFORMS[name];
  const dir = path.join(TARGET, name);
  const marker = path.join(dir, "RELEASE");
  if (existsSync(path.join(dir, library)) && existsSync(marker) && readFileSync(marker, "utf8").trim() === RELEASE) {
    console.log(`${name}: ${RELEASE} already there`);
    return;
  }
  const response = await fetch(`${BASE}/pdfium-${name}.tgz`);
  if (!response.ok) throw new Error(`${name}: download failed with ${response.status}`);
  const bytes = Buffer.from(await response.arrayBuffer());
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== sha256) throw new Error(`${name}: SHA-256 ${digest} differs from ${sha256}, nothing unpacked`);
  const archive = path.join(tmpdir(), `pdfium-${name}-${process.pid}.tgz`);
  writeFileSync(archive, bytes);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  try {
    execFileSync("tar", ["-xzf", archive, "-C", dir], { stdio: "inherit" });
  } finally {
    rmSync(archive, { force: true });
  }
  if (!existsSync(path.join(dir, library))) throw new Error(`${name}: ${library} missing in the archive`);
  writeFileSync(marker, `${RELEASE}\n`);
  console.log(`${name}: ${RELEASE} unpacked, ${bytes.length} bytes checked`);
}

const names = process.argv.includes("--all") ? Object.keys(PLATFORMS) : [currentPlatform()];
for (const name of names) await fetchPlatform(name);
