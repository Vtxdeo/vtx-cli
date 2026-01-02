#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const { spawn } = require("node:child_process");

const platformMap = {
  "linux-x64": {
    pkg: "@vtxdeo/cli-linux-x64",
  },
  "darwin-arm64": {
    pkg: "@vtxdeo/cli-darwin-arm64",
  },
  "win32-x64": {
    pkg: "@vtxdeo/cli-win32-x64",
  },
};

const platformKey = `${process.platform}-${process.arch}`;
const platformEntry = platformMap[platformKey];

if (!platformEntry) {
  console.error(`[vtx] Unsupported platform: ${platformKey}`);
  process.exit(1);
}

let binPath;
try {
  binPath = require(platformEntry.pkg);
} catch (err) {
  console.error(
    `[vtx] Missing platform package (${platformEntry.pkg}). Reinstall @vtxdeo/cli.`
  );
  process.exit(1);
}

if (!binPath || !fs.existsSync(binPath)) {
  console.error(`[vtx] Binary not found at ${binPath}.`);
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });

child.on("exit", (code) => {
  process.exit(code === null ? 1 : code);
});
