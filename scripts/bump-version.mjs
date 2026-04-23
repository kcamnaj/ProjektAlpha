#!/usr/bin/env node
// Bumps version in package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json
// Usage: node scripts/bump-version.mjs 0.2.0
import { readFileSync, writeFileSync } from "node:fs"
import { resolve, dirname } from "node:path"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, "..")

const newVersion = process.argv[2]
if (!newVersion || !/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error("Usage: node scripts/bump-version.mjs X.Y.Z")
  console.error("Received:", newVersion ?? "(nothing)")
  process.exit(1)
}

// 1. package.json
{
  const file = resolve(root, "package.json")
  const j = JSON.parse(readFileSync(file, "utf8"))
  const prev = j.version
  j.version = newVersion
  writeFileSync(file, JSON.stringify(j, null, 2) + "\n")
  console.log(`package.json:        ${prev} → ${newVersion}`)
}

// 2. src-tauri/Cargo.toml — nur die [package]-version ersetzen, nicht die von Dependencies
{
  const file = resolve(root, "src-tauri/Cargo.toml")
  const src = readFileSync(file, "utf8")
  const re = /^(\[package\][\s\S]*?\nversion\s*=\s*")[^"]+(")/m
  const match = src.match(re)
  if (!match) {
    console.error("Could not find [package] version in Cargo.toml")
    process.exit(1)
  }
  const prev = src.match(/^\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/m)[1]
  const out = src.replace(re, `$1${newVersion}$2`)
  writeFileSync(file, out)
  console.log(`Cargo.toml:          ${prev} → ${newVersion}`)
}

// 3. src-tauri/tauri.conf.json — nur top-level "version"
{
  const file = resolve(root, "src-tauri/tauri.conf.json")
  const j = JSON.parse(readFileSync(file, "utf8"))
  const prev = j.version
  j.version = newVersion
  writeFileSync(file, JSON.stringify(j, null, 2) + "\n")
  console.log(`tauri.conf.json:     ${prev} → ${newVersion}`)
}

console.log(`\n✓ Version bumped to ${newVersion}`)
console.log(`Next: git add -A && git commit -m "release: v${newVersion}" && git tag v${newVersion} && git push && git push --tags`)
