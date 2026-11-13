import { createHmac, randomBytes } from "node:crypto";
import { NextResponse } from "next/server";

export const runtime = "nodejs";
type Input = { coinId?: string; accountId?: string; doorId?: number | null };

function secret() { const value = process.env.HUMAN_PASS_CHALLENGE_SECRET; if (!value) throw new Error("Human-pass challenge secret is not configured."); return value; }
function recipient() { return new URL(process.env.NEXT_PUBLIC_SITE_URL ?? "http://localhost:3000").host; }
function seal(payload: object) { const body = Buffer.from(JSON.stringify(payload)).toString("base64url"); const mac = createHmac("sha256", secret()).update(body).digest("base64url"); return `${body}.${mac}`; }

export async function POST(request: Request) {
  if (process.env.TESTNET_DEMO_MODE !== "true") return NextResponse.json({ error: "Human-pass adapter is disabled." }, { status: 403 });
  try {
    const { coinId, accountId, doorId }: Input = await request.json();
    if (!coinId || !accountId || (doorId !== null && doorId !== undefined && (!Number.isInteger(doorId) || doorId < 0))) return NextResponse.json({ error: "Invalid pass request." }, { status: 400 });
    const expiresAtNs = (BigInt(Date.now() + 5 * 60_000) * 1_000_000n).toString();
    const nonce = randomBytes(32).toString("base64");
    const payload = { v: 1, coinId, accountId, doorId: doorId ?? null, nonce, expiresAtNs, recipient: recipient() };
    const message = `Auspice human-pass request\ncoin=${coinId}\naccount=${accountId}\ndoor=${doorId ?? "genesis"}\nnonce=${nonce}\nexpires_at_ns=${expiresAtNs}`;
    return NextResponse.json({ challenge: seal(payload), message, nonce, recipient: payload.recipient, expiresAtNs });
  } catch (cause) { return NextResponse.json({ error: cause instanceof Error ? cause.message : "Could not create challenge." }, { status: 500 }); }
}
