# CosmWasm/Sylvia counting contract w/ IBC enabled (Cosmos, Rust, CosmWasm, Sylvia)

This repository contains counting contract created during the study of CosmWasm/Sylvia and IBC based contracts development for Cosmos blockchains written using Rust.

> The IBC implementation attempt on Sylvia is based on https://github.com/0xekez/cw-ibc-example as at the time of
> writing this the https://cosmwasm.github.io/sylvia-book/ibc.html#ibc section is empty.
> Moreover Sylvia does NOT support natively IBC, see maintainer comment https://github.com/CosmWasm/sylvia/issues/19#issuecomment-1792586062.

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

To deploy and operate the contracts on Juno we need `junod` (https://docs.junonetwork.io/validators/getting-setup):

```
git clone https://github.com/CosmosContracts/juno
cd juno
git fetch
git checkout v17.1.1
```

To deploy and operate the contracts on Osmosis we need `osmosisd` (https://docs.osmosis.zone/cosmwasm/testnet/cosmwasm-deployment#setup-osmosis-testnet):

```
curl -sL https://get.osmosis.zone/install > i.py && python3 i.py
```

> Choose option #2 (Client Node) and #2 (Testnet) in order. You may run with sudo but don't forget to chown afterwards...

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

First we need to configure Juno node:

```
junod config node https://juno-testnet-rpc.polkachu.com:443
junod config chain-id uni-6
```

> You may explore the testnet tools here: https://polkachu.com/testnets/juno

We will create two Juno wallets first:

```
junod keys add wallet
// wallet address: juno1dkgs7ymhmnnu3c874wyaakh03jn9l3fes52jxg
junod keys add wallet2
// wallet2 address: juno1qqkj8r6hfqh93jq65jermsmq288je7873jmjh5
junod keys add wallet-ibc --output json > .wallet.junod
// wallet-ibc address: juno1hhfw77usyd6t8y9xuj6xlmqq5nkuqyc9vkcsha
```

> Latest faucet working is `https://faucet.reece.sh/uni-6/YOUR-ADDRESS-HERE`. Probably won't work for you though...

> May you want to query wallet balances use `junod query bank balances juno1dkgs7ymhmnnu3c874wyaakh03jn9l3fes52jxg`

...and create two Osmosis wallets:

```
osmosisd keys add wallet
// wallet address: osmo1kzd8am90ktcye0yrf7p6z5z4lgz3pg6sp9qu6s
osmosisd keys add wallet2
// wallet2 address: osmo1xdk392u7y36s4fe06eu04e6mt7x4kt5793takz
osmosisd keys add wallet-ibc --output json > .wallet.osmosis
// wallet-ibc address: osmo1fywaxmn73dja3kmd5deuv2gf46ktna6f6el3aw
```

> The faucet address is `https://faucet.testnet.osmosis.zone`

> May you want to query wallet balances use `osmosisd query bank balances osmo1kzd8am90ktcye0yrf7p6z5z4lgz3pg6sp9qu6s`

**We will deploy the contract to `uni-6` and `osmo-test-5` using `junod` binary.**

First we upload the code:

```
// Chain A
junod tx wasm store artifacts/counting_contract.wasm --chain-id=uni-6 --from wallet -y --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox" -b sync

// Chain B
osmosisd tx wasm store artifacts/counting_contract.wasm --chain-id=osmo-test-5 --from wallet -y --gas=auto --gas-adjustment=1.15 --gas-prices="0.025uosmo" -b sync
```

> To get contract code ID you must query the `txhash`, e.g.: `junod q tx 9C3193DF97AEED37250FE99752DA17482E242B444FDD6D90449715027C72A823 --output=json` and `osmosisd q tx 8A8E552A55D24D0068647BEE6B03E04EC069272F341467955E1FABE37720BAFB --output=json`

Than we instantiate the contracts on both networks:

```
// Chain A
CODE_ID=3860
junod tx wasm instantiate "$CODE_ID" '{"count":0,"admins":[]}' --label "counting-contract" --chain-id=uni-6 -y --from wallet --admin wallet --gas=auto --gas-adjustment=1.15 --gas-prices="0.025ujunox"

// Chain B
CODE_ID=5085
osmosisd tx wasm instantiate "$CODE_ID" '{"count":0,"admins":[]}' --label "counting-contract" --chain-id=osmo-test-5 -y --from wallet --admin wallet --gas=auto --gas-adjustment=1.15 --gas-prices="0.025uosmo"
```

> To get initiation info you must query the `txhash`, e.g.: `junod q tx 4FA0F94DFB11CA03A7476051367A6EB97B2DAED38E494C25E8F7BCA3543F523C --output=json` and `osmosisd q tx 197AC7FA66DA0E5E35EDE2F822F8DC6A54DDD5936D28B7812DC2DC7DE3A41414 --output=json`

Now we need to query the instantiated contracts:

```
// Chain A
CONTRANT_ADDRESS=juno10ddh59fvqsjjkz87edsn3368ph0xd2fsw9a0dsm3waqkg9e4r23qakjkrf
junod query wasm contract "$CONTRANT_ADDRESS"

// Chain B
CONTRANT_ADDRESS=osmo1xz00vhlm7e3ysj9f2v3jtcjpqvectwgdkkxuau8rw290ys087s6qtk24hy
osmosisd query wasm contract "$CONTRANT_ADDRESS"
```

> Look at `ibc_port_id` in the output. Should the contract have IBC entry points enabled it will contain a value e.g. `wasm.juno175jthggp4drryjhfljzxxy0lnxfq3g4dfehahct95e22m57lhrxsw07f6e`

Before starting the relayer we need to configure it:

```
mkdir ~/.hermes && touch ~/.hermes/config.toml
cat .hermes.config.toml > ~/.hermes/config.toml
hermes config validate
```

...and generate wallets for relaying:

```
hermes keys add --chain uni-6 --key-file .wallet.junod
hermes keys add --chain osmo-test-5 --key-file .wallet.osmosis
```

> May you want to query balances use `hermes keys balance --chain osmo-test-5`
 and `hermes keys balance --chain uni-6`
Afterwards, in order to get IBC running we need to create the relayer channel and start it:

```
IBC_PORT_A=wasm.juno10ddh59fvqsjjkz87edsn3368ph0xd2fsw9a0dsm3waqkg9e4r23qakjkrf
IBC_PORT_B=wasm.osmo1xz00vhlm7e3ysj9f2v3jtcjpqvectwgdkkxuau8rw290ys087s6qtk24hy
hermes create channel --a-chain uni-6 --b-chain osmo-test-5 --a-port "$IBC_PORT_A" --b-port "$IBC_PORT_B" --channel-version counter-contract-1 --new-client-connection
hermes start
```
