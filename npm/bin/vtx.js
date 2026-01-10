#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");

const platformMap = {
  "linux-x64": {
    pkg: "@vtxdeo/cli-linux-x64",
  },
  "linux-arm64": {
    pkg: "@vtxdeo/cli-linux-arm64",
  },
  "darwin-x64": {
    pkg: "@vtxdeo/cli-darwin-x64",
  },
  "darwin-arm64": {
    pkg: "@vtxdeo/cli-darwin-arm64",
  },
  "win32-x64": {
    pkg: "@vtxdeo/cli-win32-x64",
  },
  "win32-arm64": {
    pkg: "@vtxdeo/cli-win32-arm64",
  },
};

const platformKey = `${process.platform}-${process.arch}`;
const platformEntry = platformMap[platformKey];

if (!platformEntry) {
  console.error(`[vtx] Unsupported platform: ${platformKey}`);
  process.exit(1);
}

let binPath;
let usedFallback = false;
try {
  binPath = require(platformEntry.pkg);
} catch (err) {
  binPath = null;
}

if (!binPath || !fs.existsSync(binPath)) {
  const localBin = path.join(
    __dirname,
    `vtx${process.platform === "win32" ? ".exe" : ""}`
  );
  if (fs.existsSync(localBin)) {
    binPath = localBin;
    usedFallback = true;
  } else if (!binPath) {
    console.error(
      `[vtx] Missing platform package (${platformEntry.pkg}) and no local binary found.`
    );
    process.exit(1);
  } else {
    console.error(`[vtx] Binary not found at ${binPath}.`);
    process.exit(1);
  }
}

if (!binPath || !fs.existsSync(binPath)) {
  console.error(`[vtx] Binary not found at ${binPath}.`);
  process.exit(1);
}

if (usedFallback) {
  console.log(`[vtx] Using local binary from postinstall: ${binPath}`);
} else {
  console.log(`[vtx] Using platform package: ${platformEntry.pkg}`);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });

child.on("exit", (code) => {
  process.exit(code === null ? 1 : code);
});
