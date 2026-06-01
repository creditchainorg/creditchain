# CreditChain Native Token and Self-Custody Wallet

CreditChain uses **CCC** as its native gas token.

CCC is not an ERC-20 contract. It is the EVM native currency used to pay gas,
fund validators/operators, preallocate genesis accounts, and display balances
in wallets. It has 18 decimals, like ETH.

## Registry-First Token Metadata

Every network declares its native currency in:

```text
deploy/shared/networks.json
```

The canonical CreditChain entry is:

```json
"native_currency": {
  "name": "CreditChain Token",
  "symbol": "CCC",
  "decimals": 18
}
```

That metadata is mirrored into:

```text
creditchain-browser/app/browser-web/lib/networks.ts
iwallet/crates/iwallet-core/src/creditchain.rs
```

The chain itself only enforces balances, gas accounting, and chain id. Wallets,
faucets, explorers, and onboarding tools use the registry metadata to decide
what humans see.

## Enterprise Forks

An institution can fork CreditChain and choose its own native gas token symbol
without changing EVM transaction semantics.

Example:

```json
"native_currency": {
  "name": "Example Bank Settlement Token",
  "symbol": "EBST",
  "decimals": 18
}
```

Recommended customization points:

| Surface | What to change |
|---|---|
| `deploy/shared/networks.json` | `display_name`, `chain_id`, RPC URLs, `native_currency` |
| `genesis/*.json` | `config.chainId`, `extraData`, preallocated balances |
| `deploy/*/.env` | `NATIVE_TOKEN_NAME`, `NATIVE_TOKEN_SYMBOL`, `NATIVE_TOKEN_DECIMALS` |
| Browser mirror | `nativeCurrency` in `lib/networks.ts` |
| iWallet mirror | `native_currency` in `creditchain.rs` |

Keep decimals at 18 unless the institution has a strong legacy-system reason.
Many EVM tools assume 18-decimal native gas units.

## Wallet Model

iWallet is the native CreditChain wallet surface. Its security posture is
self-custody:

- users generate or import a BIP-39 mnemonic;
- private keys stay on the user's device or institution-controlled signing
  environment;
- the wallet signs locally and broadcasts signed transactions to the selected
  CreditChain RPC;
- agents and backends request actions, but they do not custody keys.

This avoids custodian risk and is easier to align with regulated operating
models:

- retail users can hold their own keys;
- institutions can plug in HSM/MPC/cold-signing policy later;
- audit logs can record approvals and transaction hashes without exposing seed
  material;
- compliance controls can live around signing policy instead of inside a pooled
  custodian account.

## AI Financial-Market Positioning

CreditChain's native token is deliberately simple. The differentiator is the
market layer built above it:

- EVM compatibility for wallets, Solidity, Foundry, and institutional tooling;
- self-custody wallet flows for users and regulated desks;
- Agent Finance events for AI-driven payment intents, spend permits, task
  receipts, credit objects, and AgentIDs;
- Browser and Bot surfaces that explain on-chain state with evidence rather
  than asking users to trust a black box.

CCC pays for execution. CreditChain's product power comes from making AI agents,
wallets, markets, and compliance share one verifiable settlement layer.
