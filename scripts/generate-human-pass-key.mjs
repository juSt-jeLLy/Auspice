#!/usr/bin/env node
import { generateKeyPairSync } from "node:crypto";

const { privateKey, publicKey } = generateKeyPairSync("ed25519");
const publicDer = publicKey.export({ format: "der", type: "spki" });
const rawPublicKey = publicDer.subarray(-32).toString("base64");
console.log(`HUMAN_PASS_SIGNING_KEY_PEM=${JSON.stringify(privateKey.export({ format: "pem", type: "pkcs8" }).toString())}`);
console.log(`Human-pass public key for LaunchConfig (Base64VecU8)=${rawPublicKey}`);
