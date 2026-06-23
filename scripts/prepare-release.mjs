#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const dryRun = args.includes("--dry-run");
const versionArg = args.find((arg) => !arg.startsWith("-"));

if (!versionArg) {
  console.error("Usage: node scripts/prepare-release.mjs <version> [--dry-run]");
  console.error("Example: node scripts/prepare-release.mjs 0.2.0");
  process.exit(1);
}

const version = versionArg.replace(/^v/, "");
const tag = `v${version}`;
const semver =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

if (!semver.test(version)) {
  console.error(`Invalid semantic version: ${versionArg}`);
  console.error("Expected a version like 0.2.0 or v0.2.0-beta.1.");
  process.exit(1);
}

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const changes = [];

function readProjectFile(path) {
  return readFileSync(resolve(root, path), "utf8");
}

function writeProjectFile(path, contents) {
  if (!dryRun) {
    writeFileSync(resolve(root, path), contents);
  }
}

function record(path, changed) {
  changes.push({ path, changed });
}

function updateJson(path, mutate) {
  const before = readProjectFile(path);
  const data = JSON.parse(before);
  const beforeData = JSON.stringify(data);
  mutate(data);
  const hasDataChanged = beforeData !== JSON.stringify(data);
  const after = `${JSON.stringify(data, null, 2)}\n`;
  record(path, hasDataChanged);
  if (hasDataChanged) {
    writeProjectFile(path, after);
  }
}

function replaceOnce(path, pattern, replacement, label) {
  const before = readProjectFile(path);
  const matches = before.match(pattern);
  if (!matches) {
    throw new Error(`Could not find ${label} in ${path}`);
  }
  const after = before.replace(pattern, replacement);
  record(path, before !== after);
  if (before !== after) {
    writeProjectFile(path, after);
  }
}

replaceOnce(
  "Cargo.toml",
  /(\[workspace\.package\][\s\S]*?version\s*=\s*")[^"]+(")/,
  `$1${version}$2`,
  "workspace package version",
);

replaceOnce(
  "inspectra-gui/src-tauri/Cargo.toml",
  /(^version\s*=\s*")[^"]+(")/m,
  `$1${version}$2`,
  "Tauri package version",
);

replaceOnce(
  "bindings/python/pyproject.toml",
  /(\[project\][\s\S]*?version\s*=\s*")[^"]+(")/,
  `$1${version}$2`,
  "Python package version",
);

updateJson("inspectra-gui/src-tauri/tauri.conf.json", (data) => {
  data.package.version = version;
});

updateJson("inspectra-gui/package.json", (data) => {
  data.version = version;
});

updateJson("inspectra-gui/package-lock.json", (data) => {
  data.version = version;
  if (data.packages?.[""]) {
    data.packages[""].version = version;
  }
});

const changed = changes.filter((entry) => entry.changed);
const unchanged = changes.filter((entry) => !entry.changed);

console.log(`${dryRun ? "Validated" : "Prepared"} Inspectra ${tag}.`);

if (changed.length > 0) {
  console.log("\nUpdated files:");
  for (const entry of changed) {
    console.log(`- ${entry.path}`);
  }
}

if (unchanged.length > 0) {
  console.log("\nAlready up to date:");
  for (const entry of unchanged) {
    console.log(`- ${entry.path}`);
  }
}

console.log("\nNext release commands:");
console.log("git add Cargo.toml inspectra-gui bindings/python/pyproject.toml");
console.log(`git commit -m "Release ${tag}"`);
console.log(`git tag ${tag}`);
console.log(`git push origin main ${tag}`);
