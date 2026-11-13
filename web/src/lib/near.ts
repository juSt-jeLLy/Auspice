export const networkId = process.env.NEXT_PUBLIC_NETWORK_ID ?? "testnet";
export const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL ?? "https://rpc.testnet.fastnear.com";
export const factoryId = process.env.NEXT_PUBLIC_FACTORY_ID ?? "auspice.testnet";
export const coinId = process.env.NEXT_PUBLIC_COIN_ID ?? "";

export type Lifecycle = "Draft" | "GenesisOpen" | "GenesisSettling" | "Live" | "DoorOpen" | "Finished";

export type Door = {
  config: { door_id: number; source_id: string; source_url: string; reserve: string; user_cap: string; claim_window_ns: number; new_humans_only: boolean; predicate_version: number };
  status: "Locked" | "Open" | "ClosedClaimed" | "ClosedBurned";
  opened_at_ns?: number | null;
  closes_at_ns?: number | null;
  proof_hash?: string | null;
  minted: string;
  burned: string;
};

export type CoinSnapshot = {
  lifecycle: Lifecycle;
  metadata: { name: string; symbol: string; decimals: number };
  launch: { creator_id: string; genesis_open_at_ns: number; genesis_close_at_ns: number; maximum_mintable_supply: string; genesis_reserve: string; genesis_price_yocto_near_per_token: string };
  doors: Door[];
  supply: { maximum_mintable_supply: string; minted_supply: string; permanently_unmintable: string; remaining_mintable_supply: string };
  events: Array<{ id: number; kind: string; actor_id: string; door_id?: number | null; amount: string; timestamp_ns: number }>;
};

async function rpc<T>(method: string, params: object): Promise<T> {
  const response = await fetch(rpcUrl, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: "auspice", method, params }),
    cache: "no-store",
  });
  const payload = await response.json();
  if (!response.ok || payload.error) throw new Error(payload.error?.message ?? "NEAR RPC request failed");
  return payload.result as T;
}

export async function view<T>(accountId: string, methodName: string, args: object = {}): Promise<T> {
  const result = await rpc<{ result: number[] }>("query", {
    request_type: "call_function",
    finality: "optimistic",
    account_id: accountId,
    method_name: methodName,
    args_base64: btoa(JSON.stringify(args)),
  });
  return JSON.parse(new TextDecoder().decode(new Uint8Array(result.result))) as T;
}

export async function readCoin(accountId = coinId): Promise<CoinSnapshot | null> {
  if (!accountId) return null;
  const [metadata, launch, lifecycle, doors, supply, events] = await Promise.all([
    view<CoinSnapshot["metadata"]>(accountId, "ft_metadata"),
    view<CoinSnapshot["launch"]>(accountId, "get_launch_config"),
    view<Lifecycle>(accountId, "get_lifecycle"),
    view<Door[]>(accountId, "list_doors"),
    view<CoinSnapshot["supply"]>(accountId, "get_supply_status"),
    view<CoinSnapshot["events"]>(accountId, "events", { limit: 24 }),
  ]);
  return { metadata, launch, lifecycle, doors, supply, events };
}

export const explorerAccount = (accountId: string) => `https://testnet.nearblocks.io/address/${accountId}`;
export const formatUnits = (value: string, decimals = 0) => {
  if (decimals === 0) return new Intl.NumberFormat().format(Number(value));
  const raw = BigInt(value);
  const divisor = 10n ** BigInt(decimals);
  const whole = raw / divisor;
  const fraction = (raw % divisor).toString().padStart(decimals, "0").replace(/0+$/, "").slice(0, 4);
  return `${new Intl.NumberFormat().format(Number(whole))}${fraction ? `.${fraction}` : ""}`;
};
