// Works out the next release version and writes it into every file that
// carries one. The release workflow (.github/workflows/release.yml) runs it;
// run it yourself with --dry-run to see what the next release would be.
//
//   node scripts/release-version.mjs [--dry-run]
//
// The first release uses the version already in package.json. After that,
// each release bumps the patch number, or the minor or major number when a
// commit since the last release says [minor] or [major].

import { execSync } from "node:child_process";
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";

const sh = (cmd) => execSync(cmd, { encoding: "utf8" }).trim();
const dryRun = process.argv.includes("--dry-run");

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const lastTag = sh("git tag --list 'v*' --sort=-v:refname").split("\n")[0] || null;

function nextVersion() {
  if (!lastTag) return pkg.version;
  const [major, minor, patch] = lastTag.slice(1).split(".").map(Number);
  const messages = sh(`git log ${lastTag}..HEAD --format=%B`);
  if (/\[major\]/i.test(messages)) return `${major + 1}.0.0`;
  if (/\[minor\]/i.test(messages)) return `${major}.${minor + 1}.0`;
  return `${major}.${minor}.${patch + 1}`;
}

// Replaces the first match only, so nothing else in the file changes.
function replaceIn(file, pattern, replacement) {
  const text = readFileSync(file, "utf8");
  if (!pattern.test(text)) throw new Error(`No version found in ${file}`);
  writeFileSync(file, text.replace(pattern, replacement));
}

const version = nextVersion();
if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error(`Not a version: ${version}`);

if (!dryRun) {
  replaceIn("package.json", /("version":\s*")[^"]+(")/, `$1${version}$2`);
  // package-lock.json carries it at the top and again for the root package.
  replaceIn("package-lock.json", /("name":\s*"overnight",\s*"version":\s*")[^"]+(")/, `$1${version}$2`);
  replaceIn("package-lock.json", /(""\s*:\s*\{\s*"name":\s*"overnight",\s*"version":\s*")[^"]+(")/, `$1${version}$2`);
  replaceIn("src-tauri/tauri.conf.json", /("version":\s*")[^"]+(")/, `$1${version}$2`);
  replaceIn("src-tauri/Cargo.toml", /^(version\s*=\s*")[^"]+(")/m, `$1${version}$2`);
  replaceIn("src-tauri/Cargo.lock", /(name = "overnight"\nversion = ")[^"]+(")/, `$1${version}$2`);
}

console.log(version);
if (process.env.GITHUB_OUTPUT) {
  appendFileSync(process.env.GITHUB_OUTPUT, `version=${version}\nprevious=${lastTag ?? ""}\n`);
}
