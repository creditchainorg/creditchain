# CreditChain Public-Test Prealloc

Devnet (chain `2026042403`) and Testnet (chain `2026042404`) share the same
prealloc table. **All addresses below derive from the publicly known
Foundry / Anvil / Hardhat test mnemonic:**

```
test test test test test test test test test test test junk
```

These keys are **public, by design**. They give every developer worldwide a
predictable, well-known funding source on our public test networks. **NEVER
reuse this mnemonic or any address below on a network that holds real value
(including Mainnet when it ships).**

## Accounts

| BIP-44 index | Address                                     | Balance         | Role                                                                                |
|--------------|---------------------------------------------|-----------------|--------------------------------------------------------------------------------------|
| `0`          | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | 1,000,000,000 CCC | Reth `--dev` signer + block coinbase. Reth uses this fixed key in dev mode.        |
| `1`          | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | 100,000,000 CCC | **Public faucet.** Held by the `creditchain-faucet` service to drip test funds.    |
| `2`          | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | 1,000,000 CCC   | OpeniBank operator — used to sign audit-anchor txs and the operator system permits. |
| `3`          | `0x90F79bf6EB2c4f870365E785982E1f101E93b906` | 1,000,000 CCC   | CreditChain Forge devnet deploy permit signer (Forge templates deploy from here).   |

## Private keys (public)

| Index | Private key                                                          |
|-------|----------------------------------------------------------------------|
| `0`   | `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80` |
| `1`   | `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d` |
| `2`   | `0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a` |
| `3`   | `0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6` |

The faucet container reads index `1`'s private key from
`deploy/<env>/secrets/faucet.key` (a hex string, no `0x` prefix). The
`deploy/<env>/scripts/init.sh` script writes this file with the value above
if it does not already exist; operators are free to swap it for a
different prefunded key on private test networks.

## Why these specific accounts?

- **Account 0** is the address Reth's `--dev` mode signs with — see
  `reth/crates/dev-cli/src/lib.rs`. By prealloc'ing it we guarantee the
  sealer always has gas to mint blocks even before the first user funds it.
- **Account 1** is a stable, well-known faucet address. Developers can
  hard-code it in tutorials and CI scripts without coordinating with us.
- **Accounts 2 and 3** are reserved for OpeniBank + Forge operational
  signers so end-to-end demos work the moment a stack comes up.
