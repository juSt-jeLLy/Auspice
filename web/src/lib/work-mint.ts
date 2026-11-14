const encoder = new TextEncoder();

function bytesToHex(bytes: Uint8Array) {
  return Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
}

export function leadingZeroBits(bytes: Uint8Array) {
  let count = 0;
  for (const value of bytes) {
    if (value === 0) { count += 8; continue; }
    count += Math.clz32(value) - 24;
    break;
  }
  return count;
}

export async function workDigest(coinId: string, accountId: string, nonce: string) {
  const first = new Uint8Array(await crypto.subtle.digest("SHA-256", encoder.encode(`${coinId}:${accountId}:${nonce}`)));
  const proof = bytesToHex(first);
  return new Uint8Array(await crypto.subtle.digest("SHA-256", encoder.encode(proof)));
}

export async function mineWorkNonce(
  coinId: string,
  accountId: string,
  difficultyBits: number,
  onProgress: (attempts: number) => void,
  signal?: AbortSignal,
) {
  const seed = `${Date.now().toString(36)}-${crypto.getRandomValues(new Uint32Array(1))[0].toString(36)}`;
  let attempts = 0;
  const batchSize = 128;
  while (!signal?.aborted) {
    const nonces = Array.from({ length: batchSize }, (_, index) => `${seed}-${attempts + index}`);
    const digests = await Promise.all(nonces.map((nonce) => workDigest(coinId, accountId, nonce)));
    const match = digests.findIndex((digest) => leadingZeroBits(digest) >= difficultyBits);
    attempts += batchSize;
    onProgress(attempts);
    if (match >= 0) return { nonce: nonces[match], attempts: attempts - batchSize + match + 1 };
    await new Promise<void>((resolve) => window.setTimeout(resolve, 0));
  }
  throw new DOMException("Mining cancelled", "AbortError");
}
