#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const fsp = require("node:fs/promises");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const tar = require("tar");
const AdmZip = require("adm-zip");

const REPO = process.env.VTX_CLI_REPO || process.env.REPO || "vtxdeo/vtx-cli";
const VERSION_ENV = process.env.VTX_CLI_VERSION || process.env.VERSION || "latest";
const BIN_NAME = "vtx";
const BIN_EXT = process.platform === "win32" ? ".exe" : "";

function platformTag() {
  switch (process.platform) {
    case "linux":
      return "linux";
    case "darwin":
      return "darwin";
    case "win32":
      return "windows";
    default:
      return null;
  }
}

function archTag() {
  switch (process.arch) {
    case "x64":
      return "amd64";
    case "arm64":
      return "arm64";
    default:
      return null;
  }
}

function requestJson(url, token) {
  return new Promise((resolve, reject) => {
    const req = https.get(
      url,
      {
        headers: token ? { Authorization: `Bearer ${token}` } : undefined,
      },
      (res) => {
        if (res.statusCode && res.statusCode >= 400) {
          reject(new Error(`Request failed: ${res.statusCode}`));
          return;
        }
        let data = "";
        res.setEncoding("utf8");
        res.on("data", (chunk) => {
          data += chunk;
        });
        res.on("end", () => {
          try {
            resolve(JSON.parse(data));
          } catch (err) {
            reject(err);
          }
        });
      }
    );
    req.on("error", reject);
  });
}

function download(url, dest, token) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const req = https.get(
      url,
      {
        headers: token ? { Authorization: `Bearer ${token}` } : undefined,
      },
      (res) => {
        if (res.statusCode && res.statusCode >= 400) {
          reject(new Error(`Download failed: ${res.statusCode}`));
          return;
        }
        res.pipe(file);
        file.on("finish", () => {
          file.close(resolve);
        });
      }
    );
    req.on("error", reject);
  });
}

async function resolveVersion() {
  if (VERSION_ENV !== "latest") {
    return VERSION_ENV.startsWith("v") ? VERSION_ENV : `v${VERSION_ENV}`;
  }
  const url = `https://api.github.com/repos/${REPO}/releases/latest`;
  const token = process.env.GITHUB_TOKEN || "";
  const json = await requestJson(url, token);
  if (!json.tag_name) {
    throw new Error("Failed to resolve latest version");
  }
  return json.tag_name;
}

async function readChecksum(pathname) {
  const content = await fsp.readFile(pathname, "utf8");
  return content.trim().split(/\s+/)[0];
}

async function fileSha256(pathname) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash("sha256");
    const stream = fs.createReadStream(pathname);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", reject);
    stream.on("end", () => resolve(hash.digest("hex")));
  });
}

async function main() {
  const osTag = platformTag();
  const arch = archTag();
  if (!osTag || !arch) {
    throw new Error(`Unsupported platform: ${process.platform}/${process.arch}`);
  }

  const version = await resolveVersion();
  const archiveExt = osTag === "windows" ? "zip" : "tar.gz";
  const assetName = `${BIN_NAME}-${osTag}-${arch}.${archiveExt}`;
  const checksumName = `${assetName}.sha256`;
  const baseUrl = `https://github.com/${REPO}/releases/download/${version}`;
  const token = process.env.GITHUB_TOKEN || "";

  const tmpDir = await fsp.mkdtemp(path.join(os.tmpdir(), "vtx-cli-"));
  const archivePath = path.join(tmpDir, assetName);
  const checksumPath = path.join(tmpDir, checksumName);

  await download(`${baseUrl}/${assetName}`, archivePath, token);
  await download(`${baseUrl}/${checksumName}`, checksumPath, token);

  const expected = await readChecksum(checksumPath);
  const actual = await fileSha256(archivePath);
  if (expected !== actual) {
    throw new Error("Checksum verification failed");
  }

  const extractDir = path.join(tmpDir, "extract");
  await fsp.mkdir(extractDir);
  if (archiveExt === "zip") {
    const zip = new AdmZip(archivePath);
    zip.extractAllTo(extractDir, true);
  } else {
    await tar.x({ file: archivePath, cwd: extractDir });
  }

  const extractedBin = path.join(extractDir, `${BIN_NAME}${BIN_EXT}`);
  const targetBin = path.join(__dirname, "..", "bin", `${BIN_NAME}${BIN_EXT}`);

  await fsp.copyFile(extractedBin, targetBin);
  if (process.platform !== "win32") {
    await fsp.chmod(targetBin, 0o755);
  }
}

main().catch((err) => {
  console.error(`[vtx] Install failed: ${err.message}`);
  process.exit(1);
});
