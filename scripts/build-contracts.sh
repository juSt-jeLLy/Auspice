#!/usr/bin/env sh
set -eu

root_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

(cd "$root_dir/contracts/coin" && cargo near build non-reproducible-wasm)
(cd "$root_dir/contracts/factory" && cargo near build non-reproducible-wasm)

printf '%s\n' "Contract artifacts built under target/near/."
