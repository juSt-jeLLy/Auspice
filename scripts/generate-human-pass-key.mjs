#!/usr/bin/env node
import { generateKeyPairSync } from "node:crypto";
import { chmodSync, writeFileSync } from "node:fs";

const privateKeyPath = process.argv[2];
if (!privateKeyPath) {
  console.error("Usage: node scripts/generate-human-pass-key.mjs /absolute/path/to/human-pass.pem");
  process.exit(1);
}
const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const publicDer = publicKey.export({ format: "der", type: "spki" });
const rawPublicKey = publicDer.subarray(-32).toString("base64");
writeFileSync(privateKeyPath, privateKey.export({ format: "pem", type: "pkcs8" }).toString(), { mode: 0o600, flag: "wx" });
chmodSync(privateKeyPath, 0o600);
console.log(`Private key written with owner-only permissions: ${privateKeyPath}`);
console.log(`Human-pass public key for LaunchConfig (Base64VecU8)=${rawPublicKey}`);
