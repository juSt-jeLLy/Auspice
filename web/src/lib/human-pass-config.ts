import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

/**
 * Local testnet development can keep the signing PEM in an owner-only file.
 * Hosted deployments must set the PEM and challenge secret directly in their
 * environment; neither value is ever exposed to the browser.
 */
export function signingPem() {
  if (process.env.HUMAN_PASS_SIGNING_KEY_PEM) return process.env.HUMAN_PASS_SIGNING_KEY_PEM;
  const path = process.env.HUMAN_PASS_SIGNING_KEY_PEM_FILE;
  if (!path) throw new Error("Human-pass signer is not configured.");
  return readFileSync(path, "utf8");
}

export function challengeSecret() {
  if (process.env.HUMAN_PASS_CHALLENGE_SECRET) return process.env.HUMAN_PASS_CHALLENGE_SECRET;
  // This fallback exists only for the explicitly labelled local testnet mode.
  // It avoids copying a second secret into a shell while deriving a stable HMAC
  // key from the already protected signing PEM.
  if (process.env.TESTNET_DEMO_MODE === "true" && process.env.HUMAN_PASS_SIGNING_KEY_PEM_FILE) {
    return createHash("sha256").update(signingPem()).digest("hex");
  }
  throw new Error("Human-pass challenge secret is not configured.");
}
