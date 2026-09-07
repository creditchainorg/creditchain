# CreditChain Ecosystem v2.3.0 — Agent Clearing (Public Testnet)

> **Network:** public **testnet** (chainId `2026042404`). **Mainnet is not
> launched** and remains gated pending external audit and the validator key
> ceremony. **CCC on testnet has no monetary value. Nothing here is investment
> advice.**

v2.2.0 gave an agent bounded authority to spend. This release gives it somewhere
to spend that does not cost more than the thing it is buying.

## The problem this fixes

`AgentSpendVault` settles every payment on-chain at roughly 42,000 gas. That is
right for a payment a human would notice. It is wrong for how agents actually
transact — thousands of sub-cent payments to a handful of counterparties, at
machine speed. **An agent paying per API call was spending more on gas than on
the service.** No fee schedule fixes that; the primitive had to change.

## Highlights

### `AgentClearing` — deposit, payment, clearing, settlement

Four layers of financial plumbing in one contract, because the deposit layer is
what makes an off-chain voucher worth anything and clearing settles directly
against it.

- **DEPOSIT** — `available` / `reserved` collateral. Collateral committed to an
  open channel cannot be withdrawn while a voucher can still be redeemed
  against it.
- **PAYMENT** — off-chain EIP-712 vouchers carrying a **cumulative running
  total**, not an increment. The newest voucher supersedes every older one, so
  the total doubles as the nonce and replay is arithmetically inert rather than
  defended against. Issuing a payment costs one local signature and nothing else.
- **CLEARING** — `settleBatch()` nets many vouchers across many channels into
  **at most one balance write per account**, and skips already-settled channels
  instead of failing the whole batch.
- **SETTLEMENT** — net positions applied atomically; a payer closing a channel
  waits out a one-hour window first, so a voucher in flight is never stranded.

### Measured on the public testnet

Against the deployed, source-verified contract:

| | |
|---|---|
| 1,000 payments issued off-chain | 4.3 s · **0 transactions · 0 gas** |
| Redeemed in one transaction | **93,315 gas** (~93 gas per payment) |
| Same run as individual transfers | ~21,000,000 gas |
| Two channels netted in one settlement | 102,322 gas |
| Contract solvency | held ≥ owed at every step |

Gas per payment keeps falling the longer a channel runs — the cost is per
*settlement*, not per payment.

### ERC-ACS standard

`docs/standards/ERC-ACS.md` — EIP-style specification of the four layers, the
voucher type, the clearing rules, and the security requirements (malleability
rejection, chain-split-safe domain separator, credit disclosure). `AgentClearing`
is the reference implementation.

### Reference off-chain client, zero dependencies

`contracts/agent-finance/demo/voucher_client.py` carries its own keccak-256 and
secp256k1 (sign **and recover**, RFC 6979 deterministic nonces, malleable
signatures rejected). The claim is that an agent can pay without touching the
chain, so the client proves it: no node, no RPC, no wallet software, no pip
install. Its `selftest` checks the primitives against known vectors, and its
digests are checked against `voucherDigest()` on-chain before the proof runs.

### Tests

- 19 example tests, including adversarial cases: stale replay, cap exceeded,
  wrong signer, malleable signature, cross-channel voucher reuse, closing inside
  the window, and a batch trying to invent an obligation.
- **4 invariants over 12,800 randomised calls each**: the contract holds at
  least what its books owe; value is conserved; no channel pays out more than it
  committed; reserved collateral matches the sum of open channels.
- Non-vacuity of the invariant handler was checked directly rather than assumed.
- Full suite: **108 tests, 0 failures.**

### Network registry correction

`deploy/shared/networks.json` described the testnet as
`dev-auto-mine-single-sealer`. It is Argos PoS with Casper FFG finality —
verified live: difficulty 0, with `safe` and `finalized` trailing `latest` by
about two epochs. A consumer trusting the old label would have treated `latest`
as settled and could act on a block that can still reorganise. The entry now
reads `pos-casper-ffg` and carries an explicit `finality` block naming
`finalized` as the settled tag.

### Upstream Reth sync — fork base → v2.5.2

This release also closes a four-month drift from upstream Reth: **698 non-merge
commits** (196 fix, 161 feat, 89 perf), **no breaking changes**. Full decision
record in [`docs/UPSTREAM-SYNC-2026-09-07.md`](docs/UPSTREAM-SYNC-2026-09-07.md).

Fixes that matter for a running chain: a sparse-trie **deadlock**
(`#26356`), stalled state-root proofs now erroring instead of hanging silently
(`#26325`), and several trie correctness fixes. Three networking fixes land
directly on problems this project hand-fixed at the deployment layer — ENR ports
kept paired with their IP (`#26529`), the bound RLPx port advertised in discv5
(`#26265`), and the fork ENR entry on custom chain ids (`#26449`), which matters
because CreditChain *is* a custom chain id.

Upstream's DNS resolver also fixes a real bug of ours: our hand-written TXT
extraction took the first record and its first character-string, which breaks on
any domain carrying SPF or DKIM records and truncates EIP-1459 entries over 255
bytes. Node discovery here is DNS-based.

`v2.5.2` was chosen over `main` deliberately — `main` carries 183 unreleased
commits, and a chain preparing for mainnet tracks a release tag.

**New:** the binary's default features now include the `revmc` EVM JIT, which
requires LLVM 22. CI and all three Dockerfiles install it automatically; a
developer machine needs `.github/scripts/install_llvm.sh macos` once. **This
release is built and verified without `jit`** — a JIT compiles
consensus-critical paths at runtime and deserves its own soak before a validator
runs it.

`tools/upstream-report.sh` measures the gap to upstream on demand, so the next
sync starts from a measurement rather than a guess.

## Deployment

| Contract | Network | Address |
|---|---|---|
| `AgentClearing` | testnet (2026042404) | `0x741a5E7DA765069Fda4B415a73f21FfF204a5B75` |

Source-verified against deployed bytecode and recorded in
`deploy/shared/networks.json` under `agent_finance_contracts`, alongside the
already-live `AgentSpendVault` (`0xE506…f994`), `AgentReputation`
(`0x715F…3162`) and `ReputationGate` (`0x30eA…389B`), which this release adds to
the registry so consumers stop reading addresses out of prose.

### Demo script fix

`demo/prove_against_deployed.sh` hardcoded 3 CCC amounts, which required a
pre-funded owner and so could not be re-run self-service: the faucet gives 1 CCC
per address per day and rate-limits by IP as well. Amounts are now overridable
via `BUDGET_CCC` / `PERTX_CCC` / `SPEND_CCC` / `GAS_CCC`, defaults unchanged, and
the proof was re-run end to end to confirm it still passes.

## Try it (public testnet, free)

```bash
# Prove the whole stack against the deployed contract — funds itself from the faucet
CLEARING=0x741a5E7DA765069Fda4B415a73f21FfF204a5B75 \
  contracts/agent-finance/demo/prove_clearing_against_deployed.sh
```

```bash
# The mandate proof (shrink the amounts to fit a faucet-funded owner)
VAULT=0xE50680e68451A07810205d0258eb567470Bdf994 OWNER_KEY=0x… \
BUDGET_CCC=1.2 PERTX_CCC=0.4 \
  contracts/agent-finance/demo/prove_against_deployed.sh
```

```bash
# The off-chain client on its own
python3 contracts/agent-finance/demo/voucher_client.py selftest
```

## Honest status

- **Not audited.** Neither `AgentClearing` nor ERC-ACS has had an external
  review. Do not put real value behind it.
- **Credit is real risk.** A channel may be opened with `cap > committed`, which
  lets a payee accept vouchers beyond the posted collateral. That is unsecured
  lending: the payee can lose the uncollateralised portion. It is opt-in,
  capped, and continuously measurable via `backing()`, which reports
  collateralisation in basis points. The contract never invents value to honour
  a voucher — an overdrawn channel pays what it holds and no more.
- **Expiry is a hard cliff.** A payee that has not redeemed by `expiry` loses
  the claim.
- **Testnet only.** No mainnet deployment exists. Test CCC has no monetary
  value.
- **Addresses are not stable across a re-genesis.** Everything above is
  recorded in the network registry precisely because it will change again the
  next time the testnet is re-genesised. Read addresses from the registry, not
  from prose.
- **Two orphan contracts exist on testnet** from a mistaken redeployment during
  this release: an `AgentSpendVault` at `0x67C357819e4c65e859CF786feA80e896D1186Ffa`
  and an `AgentReputation` at `0x7d4A628a41496E685E4B10a98f2FDaB0B8BD5827`. They
  are verified but **not canonical** and nothing references them. Ignore them.
