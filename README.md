# CosmWasm/Sylvia test contracts (learning Sylvia/CosmWasm)

This repository contains test contracts created during study of CosmWasm/Sylvia based contracts development for Cosmos blockchains written using Rust.

> The IBC implementation attempt on Sylvia is based on https://github.com/0xekez/cw-ibc-example as at the time of
> writing this the https://cosmwasm.github.io/sylvia-book/ibc.html#ibc section is empty...

## Preface

Rust library skeleton generated using:

```
cargo new --lib ./counting-contract
cd ./counting-contract
cargo check
```

## Prerequisites

Besides regular dependencies like Go, Rust and CosmWasm binaries, we need additional software to deploy and operate the contracts...

As the IBC relayer is used https://hermes.informal.systems.

```
cargo install ibc-relayer-cli --bin hermes --locked
```

To deploy and operate the contracts `junod` (https://docs.junonetwork.io/validators/getting-setup) is used:

```
git clone https://github.com/CosmosContracts/juno
cd juno
git fetch
git checkout v17.1.1
```

## Build

> Aliases in `.cargo/config`

Built contract to artifacts:

```
// cargo wasm-debug
cargo wasm
cosmwasm-check target/wasm32-unknown-unknown/release/counting_contract.wasm
```

Generate contract schemas:

```
cargo schema
```

Build optimized deployment version (**run in repository root!**):

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.14.0
cosmwasm-check artifacts/counting_contract.wasm
```

## Unit Testing

```
cargo test
```

## Deploying (Testnet)

First we need to configure Juno network:

```
junod config node https://juno-testnet-rpc.polkachu.com:443
junod config chain-id uni-6
```

> You may explore the testnet tools here: https://polkachu.com/testnets/juno

We will create two wallets first:

```
junod keys add wallet
// wallet address: juno1dkgs7ymhmnnu3c874wyaakh03jn9l3fes52jxg
junod keys add wallet2
// wallet2 address: juno1qqkj8r6hfqh93jq65jermsmq288je7873jmjh5
```

> `junod` is a binary similar to `wasmd` working with `Juno - Interoperable Smart Contract Network`.

> May you want to query wallet balances use `junod query bank balances juno1dkgs7ymhmnnu3c874wyaakh03jn9l3fes52jxg`

We will deploy the contract to `uni-6` and `juno-1` using `junod` binary. 

> Latest faucet working is `https://faucet.reece.sh/uni-6/YOUR-ADDRESS-HERE`. Probably won't work for you though...

First we upload the code:

```
// Chain A
junod tx wasm store artifacts/counting_contract.wasm --chain-id=uni-6 --from wallet -y --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox" -b sync

// Chain B
junod tx wasm store artifacts/counting_contract.wasm --chain-id=juno-1 --from wallet -y --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox" -b sync
```

> To get contract code ID you must query the `txhash`, e.g.: `junod q tx 5BCC9D7259D706F4683A38935B43D5136E026BAEC20A805D335DF3B532F66489 --output=json`

Than we instantiate the contracts on both networks:

```
// Chain A
CODE_ID=3859
junod tx wasm instantiate "$CODE_ID" '{"count":0,"admins":[]}' --label "counting-contract" --chain-id=uni-6 -y --from wallet --admin wallet --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox"

// Chain B
CODE_ID=xxxx
junod tx wasm instantiate "$CODE_ID" '{"count":0,"admins":[]}' --label "counting-contract" --chain-id=juno-1 -y --from wallet --admin wallet --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox"
```

> To get initiation info you must query the `txhash`, e.g.: `junod q tx 0DF4AC04842D9C2056E99CA32C5EE39E9B5FDB206C3EDADC7F7F81CB000306E8 --output=json`

Now we need to query the instantiated contracts:

```
// Chain A
CONTRANT_ADDRESS=juno175jthggp4drryjhfljzxxy0lnxfq3g4dfehahct95e22m57lhrxsw07f6e
junod query wasm contract "$CONTRANT_ADDRESS"

// Chain B
CONTRANT_ADDRESS=xxxxxxxxxxxxxxxxxxx
junod query wasm contract "$CONTRANT_ADDRESS"
```

> Look at `ibc_port_id` in the output. Should the contract have IBC entry points enabled it will contain a value e.g. `wasm.juno175jthggp4drryjhfljzxxy0lnxfq3g4dfehahct95e22m57lhrxsw07f6e`

Afterwards, in order to get IBC running we need to start the relayer.

```
IBC_PORT_A=wasm.juno175jthggp4drryjhfljzxxy0lnxfq3g4dfehahct95e22m57lhrxsw07f6e
IBC_PORT_B=xxxxxxxxx
hermes create channel --a-chain uni-6 --b-chain juno-1 --a-port "$IBC_PORT_A" --b-port "$IBC_PORT_B" --channel-version counter-contract-1
hermes start
```
