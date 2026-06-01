# Get Started on CreditChain

Welcome to **CreditChain**, the EVM-compatible L1 for AI-native financial
infrastructure. This page is the single source of truth for joining the
public **Devnet** and **Testnet** as a developer, wallet user, or node
operator. Anything not on this page lives at
[docs.creditchain.org](https://docs.creditchain.org).

For the fuller end-user and enterprise-adoption guide, see
[`docs/public/user-manual.md`](./user-manual.md).

## Networks at a glance

| Network                    | Chain ID    | RPC URL                              | WS URL                                  | Faucet                                            | Status |
|----------------------------|-------------|--------------------------------------|-----------------------------------------|---------------------------------------------------|--------|
| **CreditChain Devnet**     | `2026042403` | `https://devnet.creditchain.org`     | `wss://devnet.creditchain.org/ws`       | `https://faucet.creditchain.org/devnet`           | active |
| **CreditChain Testnet**    | `2026042404` | `https://testnet.creditchain.org`    | `wss://testnet.creditchain.org/ws`      | `https://faucet.creditchain.org/testnet`          | active |
| **CreditChain Mainnet**    | `2026042405` | `https://rpc.creditchain.org`        | `wss://rpc.creditchain.org/ws`          | n/a                                                | planned (G5) |

Native gas currency on every public CreditChain network: **`CCC`**, 18
decimals. CCC pays transaction fees and is displayed in MetaMask / Frame /
Rabby as `CCC`.

Institutional forks can customize the native token metadata in
`deploy/shared/networks.json` and the two typed mirrors. See
[`docs/native-token-and-wallet.md`](../native-token-and-wallet.md).

> **Devnet** resets at each release tag. **Testnet** is stable — never
> wiped except via an explicit hard fork.

## 1. Add CreditChain to your wallet

### MetaMask (one-click)

Open MetaMask → *Networks* → *Add a network manually* → fill in:

| Field | Devnet | Testnet |
|---|---|---|
| Network Name        | CreditChain Devnet                    | CreditChain Testnet                    |
| RPC URL             | `https://devnet.creditchain.org`      | `https://testnet.creditchain.org`      |
| Chain ID            | `2026042403`                          | `2026042404`                           |
| Currency Symbol     | `CCC`                                 | `CCC`                                  |
| Block Explorer URL  | `https://explorer.creditchain.org/devnet` | `https://explorer.creditchain.org/testnet` |

### EIP-3085 (`wallet_addEthereumChain`)

```js
await window.ethereum.request({
  method: "wallet_addEthereumChain",
  params: [{
    chainId: "0x78c1643",                          // devnet (0x78c1644 for testnet)
    chainName: "CreditChain Devnet",
    nativeCurrency: { name: "CreditChain Token", symbol: "CCC", decimals: 18 },
    rpcUrls: ["https://devnet.creditchain.org"],
    blockExplorerUrls: ["https://explorer.creditchain.org/devnet"]
  }]
});
```

### Other wallets

Any EVM-compatible wallet works (Rabby, Frame, Coinbase Wallet, Trust). Use
the table above for the four required fields.

## 2. Claim test CCC from the faucet

Each network ships a public faucet with a 1-CCC-per-address-per-day cap
(testnet) or 10-CCC-per-address-per-hour (devnet).

```bash
curl -fsS -H 'content-type: application/json' \
  --data '{"address":"0xYOUR_ADDRESS"}' \
  https://faucet.creditchain.org/testnet/drip
```

Success response:
```json
{
  "network": "creditchain-testnet",
  "chain_id": 2026042404,
  "tx": "0x9f…",
  "to":  "0xYOUR_ADDRESS",
  "amount":     "1.0 CCC",
  "amount_base_units": "1000000000000000000"
}
```

Rate-limited response:
```json
{ "code": "faucet.ip_rate_limited", "message": "...", "retry_after_seconds": 3600 }
```

Open `https://explorer.creditchain.org/<network>/r/<tx>` to see the
on-chain receipt.

The faucet refuses to start if its configured chain id disagrees with the
RPC's `eth_chainId`, so a drip you receive on testnet is guaranteed to
have settled on testnet — not on devnet by accident.

## 3. Use the JSON-RPC

Every standard Ethereum JSON-RPC method works. Quick checks:

```bash
# chain id (must match the table above)
curl -fsS -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}' \
  https://testnet.creditchain.org

# latest block
curl -fsS -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}' \
  https://testnet.creditchain.org

# balance
curl -fsS -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"]}' \
  https://testnet.creditchain.org
```

The public RPC enforces:

- **30 req/s per IP (burst 60)** on testnet; **60 req/s per IP (burst 120)**
  on devnet.
- **50 concurrent connections per IP** (testnet) / **80** (devnet).
- A JSON-RPC method **denylist**. Public path rejects `admin_*`,
  `personal_*`, `miner_*`, `engine_*`. Testnet additionally rejects
  `debug_*`, `trace_*`, and `txpool_content` — those remain available on
  devnet for contract developers.

Responses carry `X-CreditChain-Network` and `X-CreditChain-Chain-Id`
headers for client-side validation.

## 4. Deploy a contract with Foundry

```bash
# 0) Install Foundry — https://book.getfoundry.sh/getting-started/installation
curl -L https://foundry.paradigm.xyz | bash && foundryup

# 1) New project
forge init hello-creditchain && cd hello-creditchain

# 2) Sample contract — uses the well-known Foundry test mnemonic account[1]
#    (the faucet key) as a deployer; this account is prefunded on devnet
#    and testnet.
export ANVIL_PRIVATE_KEY=0x<funded-test-key>
export RPC=https://testnet.creditchain.org

# 3) Deploy
forge create src/Counter.sol:Counter \
  --rpc-url $RPC \
  --private-key $ANVIL_PRIVATE_KEY \
  --broadcast

# 4) Read back
cast call <DEPLOYED_ADDR> "number()(uint256)" --rpc-url $RPC
```

## 5. Deploy the Agent Finance protocol (operators)

If you're standing up a private CreditChain network — or you just want to
re-deploy on a fresh devnet/testnet release — the
[`CreditAgentFinance.sol`](https://github.com/openibank/creditchain/blob/main/contracts/agent-finance/src/CreditAgentFinance.sol)
contract is the event-first MVP that emits the `SpendPermit`,
`PaymentIntent`, `TaskReceipt`, `SettlementReceipt`, and `CreditObject`
events the Browser indexes.

```bash
git clone https://github.com/openibank/creditchain
cd creditchain
./deploy/scripts/deploy-agent-finance.sh devnet     # or testnet, local-single, ...
```

What the script does:

1. Resolves the RPC URL + chain id from `deploy/shared/networks.json`.
2. Sanity-checks the RPC's `eth_chainId` matches the registry.
3. Runs `forge script Deploy.s.sol:Deploy` inside a one-shot
   `ghcr.io/foundry-rs/foundry` container, signing with the OpeniBank
   operator key (Foundry account[2], prefunded in genesis).
4. Captures the deployed address and writes it to
   `deploy/<network>/secrets/credit-agent-finance.address`.
5. Prints the snippet to paste into the typed registry mirrors
   (`shared/networks.json`, `iwallet/crates/iwallet-core/src/creditchain.rs`,
   `app/browser-web/lib/networks.ts`) so wallets and the Browser can
   reference the contract.

Operators running their own network can override `OPERATOR_PRIVATE_KEY`
to deploy under a different prefunded address.

## 6. Use iWallet

iWallet is the OpeniBank self-custody wallet surface. Users generate or import
their own BIP-39 mnemonic, private keys stay on the user's device or
institution-controlled signing environment, and the wallet signs locally before
broadcasting to the selected CreditChain RPC. It ships built-in support for all
five CreditChain networks via the canonical registry.

```bash
# list networks
iwallet networks

# JSON form (for tooling / CI)
iwallet networks --json

# query a balance against testnet
iwallet balance --address 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 --network testnet
# Balance for 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 on CreditChain Testnet ...
#   Balance: 100000000.0 CCC

# or pin the network via env var
export IWALLET_NETWORK=testnet
iwallet balance --address 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
iwallet status   0xYOUR_TX_HASH
```

## 7. Use the Browser

The public block explorer at
[`explorer.creditchain.org`](https://explorer.creditchain.org) exposes a
**network switcher** in the top-right of the header. Selecting a network
persists in localStorage and is shareable via the `?net=<id>` URL param,
e.g. `https://explorer.creditchain.org/?net=testnet`.

The Browser indexes Agent Finance primitives natively — `/spend-permits`,
`/payment-intents`, `/credit-objects`, `/agents` — and surfaces them
alongside the standard EVM views (blocks / transactions / contracts / tokens).

## 8. Run your own Browser backend (mirror our indexer)

The public Browser frontend at `https://explorer.creditchain.org` calls
per-network APIs (`api-devnet.creditchain.org`, `api-testnet.creditchain.org`).
You can run those backends yourself if you need:

- a private, write-isolated copy of the indexed data,
- a higher request budget than our public API allows,
- a fork-and-extend playground for your own analytics queries.

```bash
git clone https://github.com/openibank/creditchain-browser
cd creditchain-browser

# Devnet backend (postgres + cc-indexer + cc-browser-api)
cd deploy/devnet
cp .env.example .env
$EDITOR .env                                # set POSTGRES_PASSWORD; optional private RPC URL
docker compose --env-file .env up -d --build
curl -fsS http://localhost:8090/v1/chain/status | jq

# Testnet backend
cd ../testnet
cp .env.example .env && $EDITOR .env
docker compose --env-file .env up -d --build
curl -fsS http://localhost:8091/v1/chain/status | jq
```

Point your frontend (forked or hosted) at your own host by either editing
`app/browser-web/lib/networks.ts`'s `apiUrl` or by setting per-network
env overrides at build time:

```bash
NEXT_PUBLIC_CREDITCHAIN_API_URL_DEVNET=https://your-host:8090
NEXT_PUBLIC_CREDITCHAIN_API_URL_TESTNET=https://your-host:8091
```

Full operator guide: [`creditchain-browser/deploy/README.md`](https://github.com/openibank/creditchain-browser/blob/main/deploy/README.md).

## 9. Run your own node (peer with us)

```bash
git clone https://github.com/openibank/creditchain
cd creditchain
docker build -t creditchain:local .

# Devnet (single node, auto-mining in dev mode)
cd deploy/devnet && ./scripts/init.sh && docker compose --env-file .env up -d --build

# Testnet (peers with our sealer via static enode)
cd deploy/testnet && cp .env.example .env
# edit .env: set PUBLIC_HOST=<your IP>, paste our published enode into TRUSTED_PEERS
./scripts/init.sh && docker compose --env-file .env up -d --build
```

Our published testnet enode (any node operator can use this as a trusted
peer):
```
enode://<published-on-https://docs.creditchain.org/testnet/enode>@testnet.creditchain.org:30303
```

See [`creditchain/deploy/README.md`](../../deploy/README.md) for the full
hardening checklist before exposing your node publicly.

## 10. Known limitations (devnet + testnet)

- Devnet and testnet currently use **Reth dev-mode auto-mining** rather than
  real PoS. Block production is centralized on the sealer (one node per
  network). This is fine for dev + integration; mainnet (G5) will use
  lighthouse-driven PoS.
- Chain may reorg under load — clients should wait **N=3 confirmations**
  before treating a transaction as final on devnet/testnet.
- Devnet resets at each protocol release. Pin to a release tag if you need
  state continuity.
- The well-known prealloc keys in [`genesis/PREALLOC.md`](../../genesis/PREALLOC.md)
  are **public, by design**. Do not reuse them on a network that holds
  real value.

## 11. Get help

- **Issues & feature requests:** [github.com/openibank/creditchain/issues](https://github.com/openibank/creditchain/issues)
- **General questions:** [docs.creditchain.org](https://docs.creditchain.org)
- **Status:** [status.creditchain.org](https://status.creditchain.org)
- **Security disclosures:** [`SECURITY.md`](../../SECURITY.md)
