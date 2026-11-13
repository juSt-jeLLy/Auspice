#!/usr/bin/env sh
set -eu

: "${AUSPICE_FACTORY_ID:?Set AUSPICE_FACTORY_ID, for example auspice.testnet}"
: "${AUSPICE_FACTORY_OWNER_ID:?Set AUSPICE_FACTORY_OWNER_ID to the account recorded as factory owner}"

root_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
coin_wasm="$root_dir/target/near/auspice_coin/auspice_coin.wasm"
factory_wasm="$root_dir/target/near/auspice_factory/auspice_factory.wasm"

test -x "$(command -v near)" || { printf '%s\n' "Install near-cli-rs first: cargo install near-cli-rs" >&2; exit 1; }
test -f "$coin_wasm" || { printf '%s\n' "Run scripts/build-contracts.sh first." >&2; exit 1; }
test -f "$factory_wasm" || { printf '%s\n' "Run scripts/build-contracts.sh first." >&2; exit 1; }

hash=$(near contract deploy-as-global use-file "$coin_wasm" as-global-hash "$AUSPICE_FACTORY_ID" network-config testnet sign-with-keychain send | sed -n 's/.*\([1-9A-HJ-NP-Za-km-z]\{32,\}\).*/\1/p' | tail -n 1)
test -n "$hash" || { printf '%s\n' "Could not read the global coin code hash from the NEAR CLI output." >&2; exit 1; }

near contract deploy "$AUSPICE_FACTORY_ID" use-file "$factory_wasm" with-init-call new json-args "{\"owner_id\":\"$AUSPICE_FACTORY_OWNER_ID\",\"coin_code_hash\":\"$hash\"}" prepaid-gas '30 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send
printf '%s\n' "Factory deployed: $AUSPICE_FACTORY_ID"
printf '%s\n' "Pinned coin code hash: $hash"
