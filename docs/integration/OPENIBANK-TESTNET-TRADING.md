# openibank on testnet CCC — a trading proving ground

How openibank.com uses Argos testnet CCC to advance the trading business.
Companion to [OPENIBANK-CCC-GAS.md](OPENIBANK-CCC-GAS.md), which covers gas and
onboarding.

## The constraint that shapes the whole design

Testnet CCC has **no monetary value**, and Argos **may be reset**. Both are true
today and both are published on creditchain.org.

So there is a design the business must not build: anything whose value to the
user depends on believing test CCC is worth something, or will be. That means no
purchase of test CCC for real money, no promise that testnet balances convert to
mainnet CCC, and no competition framed as *earnings*. Beyond being untrue, a
promise of future convertibility turns a faucet token into something a regulator
would read as an investment contract, and it makes the faucet worth attacking.

That constraint removes one bad idea. What remains is better, because it is the
thing that actually shortens the path to a working mainnet trading business:

> **Testnet CCC is not a product. It is the settlement asset of a full-fidelity
> rehearsal of the real exchange — same code, same settlement path, no financial
> risk.**

Everything below follows from that.

---

## 1. What a rehearsal is worth

Three concrete assets, none of which require test CCC to be worth anything:

**A settlement path proven under load.** Order matching is the easy part; the
part that breaks a launch is settlement, reconciliation, and what happens when a
transaction is dropped, reorged, or stuck. Argos has real finality (Casper FFG),
so a rehearsal here exercises the same failure modes mainnet will have. Bugs
found now cost nothing; the same bug at mainnet costs custody.

**A queue of qualified users at launch.** People who have already connected a
wallet, placed orders, and understand the product. Acquisition happens before
launch rather than after.

**A portable, on-chain track record.** `AgentReputation` is already deployed and
verified on Argos. A trader's or agent's history can be recorded on-chain as
reputation — and reputation is exactly the thing that *should* survive to
mainnet, because carrying it forward makes no value claim.

---

## 2. Architecture

```
  trader / AI agent
        |  signed order intent
        v
  openibank order gateway ──────────► openmatch (batch auction, off-chain)
        |                                     |  deterministic clearing
        |  authority check                    v
        v                             settlement batch
  AgentSpendVault  ◄──────────────────────────┘
   (mandate: caps, rate limit, expiry, revoke)
        |
        v
  CreditChain Argos — chain 2026042404, CCC native gas + settlement
```

Matching stays off-chain: a batch auction needs latency and ordering guarantees
a block time cannot give. Settlement and **authority** go on-chain, which is
where the guarantees actually matter.

### Why the mandate is the interesting part

Every exchange has the same problem: to trade on a user's behalf, something must
hold authority over their funds. Conventionally that is a hot wallet key, and its
blast radius is the whole balance.

`AgentSpendVault` replaces the key with a **bounded mandate** — budget ceiling,
per-transaction cap, rolling-window rate limit, optional recipient allowlist,
expiry, and instant owner revocation, all enforced by the chain. A compromised
trading session can lose at most what the mandate allows, and the owner revokes
without asking openibank to do anything.

This is openibank's actual differentiator, it is already deployed and verified,
and testnet is where it gets proven. See the live proof on forge.creditchain.org.

---

## 3. The asset set

One asset is not a market. A rehearsal needs at least a quote asset:

| Asset | Role | Source |
|---|---|---|
| CCC | native gas + base asset | genesis, faucet, `ecosystem` role |
| test stablecoin (e.g. `tUSD`) | quote asset | ERC-20 deployed to Argos, minted by `ecosystem` |
| test wrapped majors (optional) | more pairs | same |

Mint the quote asset from the `ecosystem` role rather than the faucet, so
issuance is deliberate and accounted. Name every test asset with a `t` prefix and
render it that way in the UI — a user must never confuse `tUSD` for a real
stablecoin.

---

## 4. Incentives without a value claim

The rehearsal still needs people to show up. Reward participation with things
that are honestly transferable, and never with tokens:

**Good — no value claim, and each survives to mainnet:**
- **Reputation.** Recorded on-chain via `AgentReputation`. Portable, verifiable,
  and it is the record itself that has worth, not a balance.
- **Early mainnet access.** A place in the queue.
- **Disclosed fee credits at mainnet** — a discount on a future service, with the
  amount, the expiry, and the conditions published up front. This is a commercial
  promotion, not a token, and it must be described as one.
- **Higher API rate limits** for agent builders.
- **Standing** — a public leaderboard by risk-adjusted return, not raw PnL, so it
  measures skill rather than who drained the faucet fastest.

**Not this:**
- Test CCC purchasable for money, or tradable off-platform.
- Any statement, in any channel, that testnet balances convert to mainnet CCC.
- "Prizes" denominated in a token that will have value later.

The line is simple: reward the *behaviour* with things openibank can honestly
deliver, never with a claim about what a test token will be worth.

### Faucet abuse is a design problem, not a support problem

A leaderboard plus a public faucet is an invitation to sybil. Rate-limit per
account, and make the leaderboard rank risk-adjusted return on a **fixed
starting allocation** — a fresh account gets the same notional as everyone else,
so farming more test CCC buys no advantage. That single rule removes most of the
incentive to attack the faucet.

---

## 5. Migrating to mainnet

The rehearsal is only worth running if the code does not change at launch. Four
rules, extending §4 of the gas doc:

1. **Network is config, never constants.** Chain id, RPC, and every contract
   address live in one object per network.
2. **Verify `eth_chainId` at startup and refuse on mismatch.** This is the check
   that stops a mainnet order being signed against testnet config.
3. **Settlement code is identical across networks.** Only the config differs. If a
   branch says `if (testnet)` anywhere in the settlement path, the rehearsal has
   stopped proving anything.
4. **Balances do not migrate. Reputation does.** At mainnet, testnet balances are
   discarded — say so, loudly, from the first day a user touches the product, not
   in a footnote at launch. Reputation and standing carry over; that is the whole
   reward design.

---

## 6. Sequence

1. Deploy the test quote asset (`tUSD`) to Argos from `ecosystem`.
2. Point openmatch settlement at Argos; settle a single pair, `CCC/tUSD`.
3. Route session authority through `AgentSpendVault` mandates instead of a hot key.
4. Run a closed rehearsal — a handful of accounts, deliberate failure injection:
   dropped transactions, stuck nonces, a node restart mid-batch.
5. Record outcomes to `AgentReputation`.
6. Open it publicly with the fixed-allocation leaderboard.
7. At mainnet: flip config, redeploy contracts, fund the relayer. No app changes.

---

## 7. Honest caveats

- Argos may be reset; everything on it is disposable, including the leaderboard.
- The vault, treasury, and mandate contracts are **unaudited**. An audit is a
  prerequisite for holding real value, not for the rehearsal.
- Mainnet does not exist yet — it is blocked on the key ceremony
  (`internal/MAINNET-LAUNCH-ADVISORY.md`). No launch date should be implied to
  users from anything in this document.
- Off-chain matching means users trust openibank's sequencing between batches.
  That is a real trust assumption and it should be documented publicly, not
  glossed as "decentralised".
- Nothing here is investment advice, and test CCC is not an investment.
