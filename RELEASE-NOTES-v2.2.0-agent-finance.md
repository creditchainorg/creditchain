# CreditChain Ecosystem v2.2.0 — Agent Finance (Public Testnet GA)

> **Network:** public **testnet** (chainId `2026042404`). **Mainnet is not
> launched** and remains gated pending external audit, the PoS transition, and
> the validator key ceremony. **CCC on testnet has no monetary value. Nothing
> here is investment advice.**

This release makes CreditChain's **AI-agent-finance** layer real and usable on a
publicly reachable testnet: the chain-enforced spending mandate primitive, an
open standard for it, a live agent-commerce demo, self-serve contract
verification, and a wallet that grants/monitors/revokes mandates.

## Highlights

### Agent Finance (the breakthrough primitive)
- **`AgentSpendVault`** — chain-enforced, bounded, revocable spending authority
  for AI agents: budget cap, per-transaction max, rolling-window rate limit,
  recipient allowlist, expiry, instant owner revoke-with-refund, no global admin
  escape hatch. Native CCC, reentrancy-guarded.
- **ERC-AGM standard** — the `IAgentSpendMandate` interface + EIP-style spec
  (`docs/standards/ERC-AGM.md`), with `AgentSpendVault` as the conformance-tested
  reference implementation.
- **Audit-grade tests** — 19 Foundry tests: 13 example, **4 invariant properties
  over ~12,800 randomized sequences each** (solvency, budget, conservation), and
  an ERC-AGM conformance test.
- **Source-verified on the public explorer**, with a threat model in
  `contracts/agent-finance/SECURITY.md`.

### Live agent-commerce demo
- `contracts/agent-finance/demo/agent_commerce_demo.sh` runs an AI agent paying a
  metered service autonomously on the public testnet — every rail enforced
  on-chain, the chain rejecting over-budget / over-cap / non-allowlisted /
  post-revoke payments by name.

### Explorer + API
- **Self-serve contract verification**: `POST /v1/contracts/:address/verify`
  recompile-free bytecode match against `eth_getCode`, recording verified
  source + ABI (rejects mismatches).
- **Honest profiles**: unindexed addresses/contracts no longer show fabricated
  stats.

### iWallet (beta)
- **Agent Mandate Console** (Settings → Agent Mandates): grant, monitor, and
  revoke ERC-AGM mandates — "grant a mandate, not your keys."
- **CreditChain wired to the live testnet by default.**
- **Security UX**: Face-ID-first unlock with a friendly in-app password fallback
  (no more iOS system passcode sheet), and a **session vault** — unlock once,
  sign for the rest of the session with no re-prompt.

## Try it (public testnet, free)

```bash
# RPC
curl -s https://testnet.creditchain.org -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}'
# Faucet
curl -s -X POST https://faucet.creditchain.org/testnet/drip \
  -H 'content-type: application/json' -d '{"address":"0xYOURADDRESS"}'
# Agent-commerce demo
RPC_URL=https://testnet.creditchain.org \
  contracts/agent-finance/demo/agent_commerce_demo.sh
```

- RPC: `https://testnet.creditchain.org`
- Faucet: `https://faucet.creditchain.org`
- Explorer: `https://explorer.creditchain.org`
- Explorer API: `https://api-testnet.creditchain.org`

## Honest status

- **Done & live on public testnet:** the items above, end-to-end.
- **Gated (not in this release):** mainnet launch — pending external audit, a
  PoS testnet stable ≥30 days with ≥5 validators, signed genesis, and an HSM key
  ceremony (see `ECOSYSTEM-MASTERPLAN.md`).
- **Post-release verification:** the on-device send/biometric smoke tests in
  `iwallet/RELEASE-RUNBOOK.md` §3 are the final hardware pass before a store
  listing. (The `scan.creditchain.org` explorer is live and TLS-valid.)
- **Security:** `AgentSpendVault` is invariant-tested and threat-modeled but
  **not yet externally audited** — do not custody value of consequence.
