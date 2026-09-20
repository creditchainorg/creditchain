# CreditChain User Manual

> **Superseded in part.** Network details here predate Argos testnet. Current
> network, wallet, and node instructions are at
> [docs.creditchain.org](https://docs.creditchain.org).

This manual is for developers, wallet users, and enterprise teams adopting or
forking CreditChain.

## 1. What CreditChain Is

CreditChain is an EVM-compatible L1 for AI-native financial markets. It keeps
standard Ethereum JSON-RPC and Solidity tooling while adding an Agent Finance
layer for AI-driven payment intents, spend permits, task receipts, credit
objects, and AgentIDs.

The native gas token on public CreditChain networks is:

| Field | Value |
|---|---|
| Name | `CreditChain Coin` |
| Symbol | `CCC` |
| Decimals | `18` |

CCC pays gas. It is not an ERC-20 contract.

Enterprise forks may customize the native token metadata before launch. Keep
`18` decimals unless there is a strong compatibility reason to do otherwise.

## 2. Networks

| Network | Chain ID | RPC URL | Faucet | Status |
|---|---:|---|---|---|
| Devnet | `2026042403` | `https://devnet.creditchain.org` | `https://faucet.creditchain.org/devnet` | active, resettable |
| Testnet | `2026042404` | `https://testnet.creditchain.org` | `https://faucet.creditchain.org/testnet` | active, stable |
| Mainnet | `2026042405` | `https://rpc.creditchain.org` | n/a | planned |

Devnet is for fast iteration and may reset by release. Testnet is for durable
integrations and should not reset casually.

## 3. Add CreditChain To A Wallet

Any EVM wallet works. Add the network manually:

| Field | Devnet | Testnet |
|---|---|---|
| Network name | CreditChain Devnet | CreditChain Testnet |
| RPC URL | `https://devnet.creditchain.org` | `https://testnet.creditchain.org` |
| Chain ID | `2026042403` | `2026042404` |
| Currency symbol | `CCC` | `CCC` |
| Explorer URL | `https://explorer.creditchain.org/devnet` | `https://explorer.creditchain.org/testnet` |

For app-based onboarding:

```js
await window.ethereum.request({
  method: "wallet_addEthereumChain",
  params: [{
    chainId: "0x78c2f424",
    chainName: "CreditChain Testnet",
    nativeCurrency: {
      name: "CreditChain Coin",
      symbol: "CCC",
      decimals: 18
    },
    rpcUrls: ["https://testnet.creditchain.org"],
    blockExplorerUrls: ["https://explorer.creditchain.org/testnet"]
  }]
});
```

## 4. Claim Test CCC

```bash
curl -fsS -H 'content-type: application/json' \
  --data '{"address":"0xYOUR_ADDRESS"}' \
  https://faucet.creditchain.org/testnet/drip
```

The faucet response includes the chain id, amount, base units, and transaction
hash so clients can verify the drip.

## 5. Use JSON-RPC

```bash
curl -fsS -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}' \
  https://testnet.creditchain.org

curl -fsS -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"]}' \
  https://testnet.creditchain.org
```

CreditChain uses standard EVM hex quantities. Wallets display balances in CCC
using 18 decimals.

## 6. Use iWallet

iWallet is the native self-custody wallet surface. Users or institutions hold
their own BIP-39 mnemonic/private keys. iWallet signs locally and broadcasts
signed transactions to the selected RPC.

```bash
iwallet create
iwallet networks
iwallet balance --network testnet --address 0xYOUR_ADDRESS
iwallet status --network testnet 0xYOUR_TX_HASH
```

Security model:

- iWallet does not custody user funds.
- AI agents can request actions but do not receive private keys.
- Policy and audit controls can wrap signing for regulated deployments.
- Institutions can later plug in HSM, MPC, or cold-signing workflows.

## 7. Deploy Contracts

CreditChain works with Foundry:

```bash
forge create src/Counter.sol:Counter \
  --rpc-url https://testnet.creditchain.org \
  --private-key $PRIVATE_KEY \
  --broadcast
```

Use funded test accounts only on devnet/testnet. Never use public test keys on
mainnet or private production networks.

## 8. Agent Finance

Operators can deploy the Agent Finance MVP contract from
[creditchainorg/contracts](https://github.com/creditchainorg/contracts):

```bash
git clone https://github.com/creditchainorg/contracts
cd contracts/agent-finance
forge script script/Deploy.s.sol:Deploy --rpc-url testnet --private-key "$PRIVATE_KEY" --broadcast
```

The contract emits the event model that CreditChain Browser indexes:

- SpendPermit
- PaymentIntent
- TaskReceipt
- SettlementReceipt
- CreditObject
- AgentID

## 9. Enterprise Fork Checklist

Before launching a fork:

- Pick unique chain ids.
- Customize the network registry (chain ids, RPC URLs, native currency).
- Customize native token metadata.
- Update genesis balances and prefunded operator accounts.
- Replace public faucet keys with institution-controlled keys.
- Mirror registry metadata into Browser and iWallet.
- Publish RPC, explorer, faucet, chain id, and native token metadata to users.
- Keep direct node RPC ports private behind the hardened RPC frontend.

Core customization files:

| File | Purpose |
|---|---|
| `genesis/*.json` | chain id, genesis metadata, prealloc |
| `docs/native-token-and-wallet.md` | token and wallet model |

## 10. Compliance Posture

CreditChain is designed so regulated users can separate decisioning, custody,
and settlement:

- AI agents propose or request actions.
- Policy engines approve or deny.
- iWallet signs locally.
- CreditChain settles and provides audit evidence.
- Browser and Bot surfaces explain what happened from indexed on-chain data.

This minimizes custodian risk while preserving a clear audit trail.
