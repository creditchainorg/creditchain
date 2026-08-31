# openibank on CCC — how it actually works

Answering two questions: how do you get CCC on testnet, and can openibank.com use
CCC as native gas.

---

## 1. Getting CCC on testnet (Argos)

Three ways, in order of how much you can get.

**Faucet — 1 CCC, anyone, no permission.** Verified working against Argos:

```bash
curl -X POST https://faucet.creditchain.org/testnet/drip \
  -H 'content-type: application/json' \
  -d '{"address":"0xYourAddress"}'
```

**Role accounts — for development at scale.** The Argos genesis funds six role
accounts totalling 1,000,000,000 CCC (published on `/tokenomics`). The ecosystem
account already funded the faucet with 1,000,000 CCC. For an integration that
needs more than drips, fund a dedicated account from `ecosystem` rather than
hammering the faucet.

**Run a validator.** Argos has 60 validators across 5 nodes today, all operator-run.
External validators earn from the staking reserve. See `docs/releases/GOVERNANCE.md`.

Test CCC has no monetary value and Argos may be reset. Do not build a business on
a balance here.

---

## 2. "Can openibank use CCC as native gas?"

**It already would, and that is the important thing to understand.**

CCC *is* the native gas token of CreditChain — not an ERC-20 sitting on top of
another chain's gas. Deploy openibank's contracts to CreditChain and every
transaction pays gas in CCC by construction. There is nothing to enable.

So the interesting question is not "can we use CCC as gas". It is:

> **How does a new openibank user, who has never heard of CCC, do their first
> transaction?**

That is the real problem, and it is the same cliff every chain has. A user arrives,
signs up, tries to do something, and is told to go acquire a token they have never
heard of, from an exchange that may not list it, before anything works. Most people
leave at that point.

---

## 3. Four ways past the onboarding cliff

Ordered by how well they work for a consumer product.

### A. Sponsored transactions — openibank pays the gas

The user never holds CCC. openibank runs a relayer holding CCC; the user signs an
intent, the relayer submits and pays.

- **Best UX by a distance.** The user never learns the word "gas".
- Costs openibank real CCC on mainnet, so it needs a budget and a rate limit.
- Standard shape: ERC-4337 paymaster, or a simpler in-house relayer if you do not
  need the whole account-abstraction stack.

**This is the recommendation for consumer flows.**

### B. Bounded mandates — the thing already built

`AgentSpendVault` (ERC-AGM, deployed and verified) is a closer fit than it first
appears. openibank funds a mandate; the user's session spends under it with a
budget cap, a per-transaction max, a rate limit, an allowlist, an expiry, and an
instant revoke — all enforced by the chain rather than by openibank's backend.

- Gas *and* spending authority in one object.
- The blast radius of a compromised session is bounded by the mandate, not by the
  account balance.
- Already live on Argos; the contracts and tests exist.

**Recommended for anything automated or agent-driven**, which is openibank's
actual differentiator.

### C. Deposit-and-swap

The user deposits a stablecoin; openibank swaps a slice to CCC internally to cover
gas. Familiar to anyone who has used a centralised exchange.

- Works, but requires liquidity and a swap venue, which do not exist on Argos yet.
- Defer until there is a market.

### D. Make the user buy CCC

Honest, simple, and the reason most consumer crypto products fail at signup. Fine
for validators and integrators. Not for end users.

---

## 4. Chain-switch design — testnet today, mainnet later

Testnet is chain `2026042404`; mainnet will be `2026042405` with a **different
genesis and different contract addresses**. If openibank hardcodes either, the
switch becomes a rewrite.

Make the network a **config object**, not constants — the same shape
`creditchain-browser/app/browser-web/lib/networks.ts` already uses:

```ts
export const NETWORK = {
  chainId: 2026042404,
  rpcUrl: "https://testnet.creditchain.org",
  explorer: "https://scan.creditchain.org/?net=testnet",
  contracts: { spendVault: "0x…", /* per-network */ },
} as const;
```

Rules that make the switch a config change rather than a migration:

1. **Never hardcode a chain id or a contract address** outside that object.
2. **Verify the chain id at startup.** Compare `eth_chainId` against the config and
   refuse to run on a mismatch. This is the single check that prevents signing a
   mainnet transaction against testnet config, or the reverse.
3. **Re-deploy contracts per network.** Addresses will differ. Record them in the
   config, not in code.
4. **Keep the gas strategy identical.** If sponsorship works on testnet, the same
   relayer works on mainnet with a funded account — no code change.
5. **Show the network in the UI, always.** The colour/word/shape scheme now used on
   creditchain.org and the block explorer exists for exactly this reason.

---

## 5. Suggested sequence

1. **Now** — deploy openibank's contracts to Argos. Gas is CCC automatically.
2. **Now** — faucet for developer accounts; fund a dedicated account from
   `ecosystem` for load testing.
3. **Next** — build the relayer (A) so a user's first action needs no CCC. This is
   the work that decides whether openibank is usable by normal people.
4. **Next** — use mandates (B) for automated and agent flows, since the contracts
   already exist and enforce the limits on-chain.
5. **At mainnet** — flip the config object, re-deploy contracts, fund the relayer
   with real CCC. Application code should not change.

## 6. Honest caveats

- Argos **may be reset**. Anything on it is disposable.
- Mainnet does not exist yet — it is halted pending re-genesis, and that is blocked
  on the key ceremony (`internal/MAINNET-LAUNCH-ADVISORY.md`).
- Sponsored gas on mainnet costs real money. Budget it, rate-limit it, and expect
  it to be abused if it is not bounded.
- The treasury and mandate contracts are **unaudited**.
