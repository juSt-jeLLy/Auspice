import { NextResponse } from "next/server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const upstream = process.env.NEAR_UPSTREAM_RPC_URL ?? "https://rpc.testnet.near.org";

/**
 * Browser-safe relay for read-only NEAR JSON-RPC calls. Wallet signatures
 * remain in the user's wallet; this route only forwards JSON-RPC requests.
 */
export async function POST(request: Request) {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: { message: "Invalid JSON-RPC body" } }, { status: 400 });
  }

  const response = await fetch(upstream, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
    cache: "no-store",
  });
  const payload = await response.text();
  return new NextResponse(payload, {
    status: response.status,
    headers: { "content-type": "application/json", "cache-control": "no-store" },
  });
}
