# CosmWasm/Sylvia test contracts (learning Sylvia/CosmWasm)

This repository contains test contracts created during study of CosmWasm/Sylvia based contracts development for Cosmos blockchains written using Rust.

> The IBC implementation attempt on Sylvia is based on https://github.com/0xekez/cw-ibc-example as at the time of
> writing this the https://cosmwasm.github.io/sylvia-book/ibc.html#ibc section is empty...

## Preface

Rust library skeleton generated using:

```
cargo new --lib ./contract
cd ./contract
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
git checkout v9.0.0
```

## Build

> Aliases in `.cargo/config`

Built contract to artifacts:

```
// cargo wasm-debug
cargo wasm
cosmwasm-check target/wasm32-unknown-unknown/release/contract.wasm
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
cosmwasm-check artifacts/contract.wasm
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

> `junod` is a binary similar to `wasmd` working with `Juno - Interoperable Smart Contract Network`


We will deploy the contract to `uni-6` and `juno-1` using `junod` binary. 

> Latest faucet working is `https://faucet.reece.sh/uni-6/YOUR-ADDRESS-HERE`. Probably won't work for you though...

First we upload the code:

```
junod tx wasm store artifacts/contract.wasm --chain-id=uni-6 --from wallet -y --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox" -b block
junod tx wasm store artifacts/contract.wasm --chain-id=juno-1 --from wallet -y --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox" -b block
```

> To get contract code ID you must query the `txhash`, e.g.: `junod q tx 8618FB149493E47E42372C88B4B20161819C446AF5A75FCD5F776622F3655E51 --output=json`

Than we instantiate the contracts on both networks:

```
CODE_ID=1
junod tx wasm instantiate "$CODE_ID" '{"count":0,"admins":[]}' --label "counting-contract" --chain-id=uni-6 -y --admin wallet --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox"

CODE_ID=2
junod tx wasm instantiate "$CODE_ID" '{"count":0,"admins":[]}' --label "counting-contract" --chain-id=juno-1 -y --admin wallet --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox"
```

Afterwards, in order to get IBC running we need to start the relayer.

```
hermes create channel --a-chain uni-6 --b-chain juno-1 --a-port wasm.juno1r8k4hf7umksu9w53u4sz0jsla5478am6yxr0mhkuvp00yvtmxexsj8wazt --b-port wasm.juno1fsay0zux2vkyrsqpepd08q2vlytrfu7gsqnapsfl9ge8mp6fvx3qf062q9 --channel-version counter-1
hermes start
```
