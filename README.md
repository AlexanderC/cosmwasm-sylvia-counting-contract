# CosmWasm/Sylvia test contracts (learning Sylvia/CosmWasm)

This repository contains test contracts created during study of CosmWasm/Sylvia based contracts development for Cosmos blockchains written using Rust.

### Intro

Rust library skeleton generated using:

```
cargo new --lib ./contract
cd ./contract
cargo check
```

## Build

> Aliases in `.cargo/config`

```
// cargo wasm-debug
cargo wasm
cosmwasm-check target/wasm32-unknown-unknown/release/contract.wasm
```

## Testing

```
cargo test
```
