#!/usr/bin/env node
/**
 * IPC Contract Checker
 *
 * Catches two classes of bug that silently drop multi-word parameters:
 *
 * CHECK 1: No `rename_all` in Rust command files.
 *   rename_all = "snake_case" forces Tauri to expect snake_case from JS.
 *   tauri-specta sends camelCase. Multi-word params silently become None.
 *
 * CHECK 2: No snake_case keys in TAURI_INVOKE calls in bindings.ts.
 *   Any key containing an underscore in an INVOKE object is wrong.
 *   tauri-specta should always generate camelCase keys.
 *
 * Run: node scripts/check-ipc.mjs
 * Exit 0 = PASS, Exit 1 = FAIL
 */

import { readFileSync, readdirSync, statSync } from "fs";
import { join, extname } from "path";

let failed = false;

function fail(msg) {
  console.error("FAIL:", msg);
  failed = true;
}

// ── CHECK 1: No rename_all in Rust command files ───────────────────────────

function walkRust(dir, files = []) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) walkRust(full, files);
    else if (extname(entry) === ".rs") files.push(full);
  }
  return files;
}

const rustFiles = walkRust("src-tauri/src");
const renameAllPattern = /rename_all\s*=\s*"snake_case"/;

for (const file of rustFiles) {
  const src = readFileSync(file, "utf8");
  const lines = src.split("\n");
  lines.forEach((line, i) => {
    if (renameAllPattern.test(line)) {
      fail(
        `${file}:${i + 1}: rename_all = "snake_case" found on a Tauri command.\n` +
        `  Tauri v2 default converts JS camelCase → Rust snake_case automatically.\n` +
        `  Adding rename_all = "snake_case" overrides this and silently drops multi-word params.\n` +
        `  Remove rename_all entirely.`
      );
    }
  });
}

// ── CHECK 2: No snake_case keys in TAURI_INVOKE calls ─────────────────────

const bindingsPath = "src/lib/bindings.ts";
let bindings;
try {
  bindings = readFileSync(bindingsPath, "utf8");
} catch {
  console.warn("WARN: bindings.ts not found — run 'pnpm tauri dev' to generate it. Skipping check 2.");
  bindings = null;
}

if (bindings) {
  // Match TAURI_INVOKE("cmd", { ... }) blocks
  const invokeRe = /TAURI_INVOKE\s*\(\s*"[^"]+"\s*,\s*\{([^}]*)\}/g;
  let m;
  let lineNum = 0;
  const lines = bindings.split("\n");

  while ((m = invokeRe.exec(bindings)) !== null) {
    const argsBlock = m[1];
    const invokeStart = bindings.lastIndexOf("\n", m.index) + 1;
    const lineIndex = bindings.slice(0, m.index).split("\n").length;

    // Extract all identifiers used as keys (shorthand or explicit)
    // Shorthand: { foo, barBaz } — key = variable name
    // Explicit:  { foo: x, barBaz: y } — key = left side
    const keyRe = /\b([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:[:,}]|\s*$)/g;
    let km;
    while ((km = keyRe.exec(argsBlock)) !== null) {
      const key = km[1];
      if (key.includes("_")) {
        fail(
          `${bindingsPath}:${lineIndex}: snake_case key '${key}' found in TAURI_INVOKE call.\n` +
          `  tauri-specta should generate camelCase keys. This key would not reach Rust.\n` +
          `  If bindings.ts was auto-generated, check for tauri-specta version issues.`
        );
      }
    }
  }
}

// ── Result ─────────────────────────────────────────────────────────────────

if (failed) {
  console.error("\nIPC contract check FAILED. Fix the issues above before shipping.");
  process.exit(1);
} else {
  console.log("IPC contract check PASSED.");
  process.exit(0);
}
