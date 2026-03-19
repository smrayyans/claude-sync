#!/usr/bin/env node
"use strict";

const path = require("path");
const { execFileSync } = require("child_process");

function getBinaryPath() {
  const platform = process.platform;
  const arch = process.arch;

  const pkgMap = {
    "linux-x64": "claude-sync-linux-x64",
    "win32-x64": "claude-sync-win32-x64",
  };

  const key = `${platform}-${arch}`;
  const pkgName = pkgMap[key];

  if (!pkgName) {
    throw new Error(
      `claude-sync: Unsupported platform ${platform}-${arch}. ` +
        `Supported: linux-x64, win32-x64`
    );
  }

  let pkgDir;
  try {
    pkgDir = path.dirname(require.resolve(`${pkgName}/package.json`));
  } catch {
    throw new Error(
      `claude-sync: Could not find platform package ${pkgName}. ` +
        `Try: npm install -g claude-sync`
    );
  }

  const ext = platform === "win32" ? ".exe" : "";
  return path.join(pkgDir, "bin", `claude-sync${ext}`);
}

module.exports = { getBinaryPath };
