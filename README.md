# CreditChain

CreditChain is a Rust-first, Reth-based, EVM-compatible public chain for
AI-native financial infrastructure. The primary node binary is `creditchaind`.

- Official website: https://www.creditchain.org
- Documentation: https://docs.creditchain.org
- Canonical repository: https://github.com/creditchainorg/creditchain
- Copyright: CreditChain Research Team

## Status

This repository is the CreditChain chain-core workspace. It preserves standard
Ethereum JSON-RPC behavior and EVM semantics while adding CreditChain branding,
environment manifests, genesis artifacts, packaging, and operator workflows.

The upstream Reth implementation remains the execution-client foundation. Keep
CreditChain-specific behavior layered around owned crates, manifests, and
configuration so upstream sync remains practical.

## Agent Finance

CreditChain's differentiating protocol layer is Agent Finance: AgentID,
SpendPermit, PaymentIntent, TaskReceipt, SettlementReceipt, and CreditObject
events for AI-native financial trust.

The contracts live in their own repository,
[creditchainorg/contracts](https://github.com/creditchainorg/contracts) (`agent-finance/`). See
`docs/agent-finance.md`.

## Public Manuals

- [Documentation site](https://docs.creditchain.org) — Argos testnet, running a node, connecting
- [User manual](docs/public/user-manual.md)
- [Native token and self-custody wallet model](docs/native-token-and-wallet.md)

## Build

```sh
cargo build --bin creditchaind --profile maxperf
```

For a debug build:

```sh
cargo build --bin creditchaind
```

## Run A Local Node

```sh
target/maxperf/creditchaind node \
  --chain genesis/local-single.json \
  --datadir .creditchain/local-single \
  --http --http.addr 127.0.0.1 --http.port 8545 \
  --http.api eth,net,web3,debug,trace \
  --metrics 127.0.0.1:9001
```

Then check the node:

```sh
CREDITCHAIND_BIN=target/maxperf/creditchaind \
CREDITCHAIN_RPC_URL=http://127.0.0.1:8545 \
scripts/creditchain-smoke.sh
```

## Environments

CreditChain treats environments as first-class release artifacts:

- `local-single`: one-node local developer network.
- `local-multinode`: local bootnode, validators, and RPC integration network.
- `devnet`: public resettable network by release.
- `testnet`: public stable network with no casual resets.
- `mainnet`: public production network.

Environment manifests live in `config/environments/`. Genesis files live in
`genesis/`.

## Compatibility

CreditChain should remain straightforward for wallets, explorers, SDKs, and
operators:

- Preserve standard Ethereum JSON-RPC methods.
- Preserve EVM execution semantics unless a protocol change is explicitly
  designed and reviewed.
- Keep the `reth` RPC namespace available while CreditChain-specific RPC
  additions mature in isolated modules.

## Packaging

Preferred public package and service names:

- `creditchaind`
- `creditchain-validator`
- `creditchain-rpc`
- `creditchaind.service`
- `creditchain-validator.service`
- `creditchain-rpc.service`

The Docker image is expected under `ghcr.io/openibank/creditchain`.
