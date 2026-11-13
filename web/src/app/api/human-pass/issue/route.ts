import { createHash, createHmac, createPrivateKey, randomUUID, sign, timingSafeEqual } from "node:crypto";
import { PublicKey } from "@near-js/crypto";
import { NextResponse } from "next/server";
import { rpcUrl } from "@/lib/near";

export const runtime = "nodejs";
type Challenge = { v: number; coinId: string; accountId: string; doorId: number | null; nonce: string; expiresAtNs: string; recipient: string };
type Signed = { accountId: string; publicKey: string; signature: string };
function secret() { const value = process.env.HUMAN_PASS_CHALLENGE_SECRET; if (!value) throw new Error("Human-pass challenge secret is not configured."); return value; }
function open(value: string): Challenge {
  const [body, mac] = value.split("."); if (!body || !mac) throw new Error("Malformed challenge.");
  const expected = createHmac("sha256", secret()).update(body).digest("base64url");
  if (expected.length !== mac.length || !timingSafeEqual(Buffer.from(expected), Buffer.from(mac))) throw new Error("Invalid challenge.");
  return JSON.parse(Buffer.from(body, "base64url").toString("utf8")) as Challenge;
}
function u32(value: number) { const bytes = Buffer.alloc(4); bytes.writeUInt32LE(value); return bytes; }
function text(value: string) { const bytes = Buffer.from(value); return Buffer.concat([u32(bytes.length), bytes]); }
function nep413Hash(message: string, nonce: string, recipient: string) {
  return createHash("sha256").update(Buffer.concat([u32(2 ** 31 + 413), text(message), Buffer.from(nonce, "base64"), text(recipient), Buffer.from([0])])).digest();
}
async function fullAccessKey(accountId: string, publicKey: string) {
  const response = await fetch(rpcUrl, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ jsonrpc: "2.0", id: "auspice-pass", method: "query", params: { request_type: "view_access_key", finality: "optimistic", account_id: accountId, public_key: publicKey } }) });
  const data = await response.json();
  return !data.error && data.result?.permission === "FullAccess";
}
export async function POST(request: Request) {
  if (process.env.TESTNET_DEMO_MODE !== "true") return NextResponse.json({ error: "Human-pass adapter is disabled." }, { status: 403 });
  try {
    const { challenge: encoded, signed }: { challenge?: string; signed?: Signed } = await request.json();
    if (!encoded || !signed) return NextResponse.json({ error: "Missing signed challenge." }, { status: 400 });
    const challenge = open(encoded);
    if (challenge.v !== 1 || BigInt(challenge.expiresAtNs) <= BigInt(Date.now()) * 1_000_000n || signed.accountId !== challenge.accountId) throw new Error("Expired or mismatched challenge.");
    const message = `Auspice human-pass request\ncoin=${challenge.coinId}\naccount=${challenge.accountId}\ndoor=${challenge.doorId ?? "genesis"}\nnonce=${challenge.nonce}\nexpires_at_ns=${challenge.expiresAtNs}`;
    if (!(await fullAccessKey(signed.accountId, signed.publicKey))) throw new Error("Signing key is not a current full-access key.");
    if (!PublicKey.from(signed.publicKey).verify(nep413Hash(message, challenge.nonce, challenge.recipient), Buffer.from(signed.signature, "base64"))) throw new Error("Invalid NEP-413 signature.");
    const privatePem = process.env.HUMAN_PASS_SIGNING_KEY_PEM;
    const maxAllocation = process.env.HUMAN_PASS_MAX_ALLOCATION;
    if (!privatePem || !maxAllocation) throw new Error("Human-pass signer is not configured.");
    const passNonce = `hp-${challenge.nonce.replace(/[^a-zA-Z0-9]/g, "").slice(0, 40)}`;
    const passExpiresAtNs = (BigInt(Date.now() + 10 * 60_000) * 1_000_000n).toString();
    const kind = challenge.doorId === null ? "genesis" : "signal_door";
    const door = challenge.doorId === null ? "" : String(challenge.doorId);
    const payload = `auspice:v1:${kind}:${challenge.coinId}:${door}:${challenge.accountId}:${passNonce}:${passExpiresAtNs}:${maxAllocation}`;
    const signature = sign(null, Buffer.from(payload), createPrivateKey(privatePem)).toString("base64");
    return NextResponse.json({ kind, coin_id: challenge.coinId, door_id: challenge.doorId, account_id: challenge.accountId, nonce: passNonce, expires_at_ns: passExpiresAtNs, max_allocation: maxAllocation, signature, issuance_id: randomUUID(), demo_mode: true });
  } catch (cause) { return NextResponse.json({ error: cause instanceof Error ? cause.message : "Could not issue a human pass." }, { status: 400 }); }
}
