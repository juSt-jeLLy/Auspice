# Auspice OutLayer verifier

`auspice-verifier` is the WASI program run by OutLayer for a Signal Door. It does not accept arbitrary website URLs or executable rules. Version one permits only a GitHub `releases/latest` endpoint whose `owner/repository`, required tag, and verifier identity were frozen in the coin at launch.

Build and test it from the repository root:

```sh
rustup target add wasm32-wasip2
cargo test -p auspice-verifier
cargo build -p auspice-verifier --target wasm32-wasip2 --release
shasum -a 256 target/wasm32-wasip2/release/auspice-verifier.wasm
```

After signing in to the OutLayer CLI, publish that exact artifact:

```sh
outlayer upload target/wasm32-wasip2/release/auspice-verifier.wasm
```

Use the returned FastFS URL and SHA-256 in the OutLayer project form (project name `auspice`, build target `wasm32-wasip2`) and in each Auspice launch form. Do not use the NEAR contract WASM or a guessed URL/hash.

The verifier output binds a request ID/nonce, coin/Door, source identity, predicate version, response SHA-256, fetched time, and the pinned WASM hash. The NEAR coin rechecks every one before opening a Door.
