#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");

const binName = process.platform === "win32" ? "vtx.exe" : "vtx";
const binPath = path.join(__dirname, binName);

if (!fs.existsSync(binPath)) {
  console.error("[vtx] Binary not found. Please reinstall @vtx/cli.");
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });

child.on("exit", (code) => {
  process.exit(code === null ? 1 : code);
});
