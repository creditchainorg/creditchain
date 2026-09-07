# Upstream Reth sync — fork base → v2.5.2

**Date:** 2026-09-07 · **Branch:** `sync/upstream-v2.5.2`
**From:** `b850f2a81d` (upstream Reth, 2026-04-28 — the fork point)
**To:** `v2.5.2` (upstream Reth release, 2026-09-02)

CreditChain had drifted four months behind upstream. This merge closes the gap
to the latest upstream *release*, keeping every CreditChain-specific behaviour
intact.

## What was synced

**698 non-merge upstream commits**, bringing 833 files, +91,547 / −42,038 lines.

| Type | Count | |
|---|---:|---|
| `fix` | 196 | correctness, liveness, and protocol fixes |
| `feat` | 161 | new capability |
| `chore` | 106 | dependency and housekeeping |
| `perf` | 89 | throughput and memory |
| `refactor` | 65 | |
| `ci` / `docs` / `test` / `bench` / `revert` | 78 | |

**No breaking-change commits** (`type!:`) in the range — this is an additive sync.

### Why v2.5.2 and not `main`

`upstream/main` is 870 commits ahead of the fork base; `v2.5.2` is 698. The
remaining 183 are unreleased work on the development line. A chain heading for
mainnet tracks a release tag, not `main`. `v2.5.2` is the release branch cut at
`3a83ccc546` (2026-08-11) plus 11 backported stabilisation fixes.

## Fixes that matter for a running chain

Selected from the 196, by whether they can affect a live validator:

**State root / sparse trie** — the subsystem with the most churn in this range:
- `fix(sparse_trie): fix dead lock for the late arrived hints (#26356)` — a real deadlock
- `fix(engine): error on stalled sparse trie proofs (#26325)` — turns a silent hang into an error
- `fix(trie): avoid marking empty storage as deleted (#26526)`
- `fix(refactor): skip transient selfdestruct storage wipes (#26433)`
- `fix(trie): retain sparse trie branch children during prune (#26376)`
- `fix(tree): elide empty new accounts from hashed state (#26367)`
- `fix(trie): support partial persistence in changeset cache (#26612)`
- `fix(trie): return empty proof for empty storage and account tries in eth_getProof (#24719)`

**Engine / payload:**
- `fix(engine): adjust payload validation (#26651)`
- `fix(engine): defer persistence handoff during payload builds (#26559)`
- `fix(engine): avoid rewinding preserved sparse trie anchors (#26422)`
- `fix(engine): restore state root task parallelism gate (#26080)`

**Networking** — three of these land directly on problems this project hit and
hand-fixed at the deployment layer:
- `fix(net): keep ENR ports paired with their IP address (#26529)`
- `fix(net): advertise the bound RLPx port in the discv5 ENR (#26265)`
- `fix(discv5): advertise fork ENR entry on custom chain ids (#26449)` — CreditChain
  *is* a custom chain id (`2026042404`)
- `fix(network): do check the ECIES id in hello (#26639)`
- `fix(eth-wire): reject out-of-range subprotocol message IDs (#26654)`
- `fix(net): recover partial header responses (#26482)`

**DNS discovery** — see "conflicts worth knowing about" below:
- `fix(dns): skip unrelated TXT records (#26603)`
- `fix(dns): rejoin TXT character-strings for EIP-1459 entries over 255 bytes (#26602)`

**Storage:**
- `fix(storage): tolerate unknown RocksDB column families (#26647)`
- `fix(rocksdb): better max_open_files config (#26812)`
- `fix(storage): roll back RocksDB BAL storage (#26646)`

## Features worth adopting

**Directly useful to CreditChain's node topology work:**
- `feat(bootnode): run discv4 and discv5 on the same UDP port (#26089)` — one
  forwarded UDP port per site instead of two
- `feat(bootnode): advertise NAT addresses in discv5 and support dual-stack (#25757)`
- `feat(config): add bootnodes to reth.toml (#26551)` — the node registry can feed
  bootnodes through config rather than command line
- `feat(net): accept discv5 ENR bootnodes via CLI (#26448)`
- `feat(net): allow updating the fork filter at runtime (#26794)`

**Operator surface:**
- `feat(cli): add configurable dev defaults (#26488)`
- `feat(engine): configure dev finality depth (#26451)`
- `feat(rpc): add eth_getMultiProof (#26555)`
- `feat: add --with-receipts to import-era (#26436)`
- `feat(snapshots): resolve base url for static files (#26576)`
- `feat(provider): return header with transaction metadata (#26571)`

**Engine / storage:**
- `feat(engine): build payloads on canonical ancestors above finality (#26567)`
- `feat(stages): handle partial trie unwinds (#26543)`
- `feat(trie): prune sparse trie nodes by epoch (#26485)`
- `feat(storage): add rocksdb BAL store (#25476)`
- `feat(net): serve real snap/2 account and storage range data (#26339)`

**Performance (89 commits)** concentrated in txpool validation, ECIES/RLPx
buffers, sparse trie pruning, and sender-recovery caching — all of which matter
for a chain whose thesis is high-volume, low-value agent payments.

## What CreditChain kept

Verified present after the merge:

- All five chain specs registered (`local-single`, `local-multinode`, `devnet`,
  `testnet`, `mainnet`) with `include_str!` genesis wiring intact
- All 5 `genesis/*.json`, 16 Solidity contracts, 128 `deploy/` files, `services/`,
  and both ERC standards (AGM, ACS)
- Branding: `target: "creditchaind::cli"`, `github.com/openibank/creditchain`,
  `https://www.creditchain.org`, OTLP service name `creditchaind`
- CreditChain's snapshot host (`https://snapshots.creditchain.org`)
- `security@creditchain.org` as the security contact
- The CI test-sharding matrix that splits general-state from fixture tests

## Conflict resolution — 41 files

Policy: **upstream's logic, CreditChain's identity.** Where a conflict was our
branding against upstream's new functionality, upstream won and the branding was
re-applied mechanically afterwards.

Notable individual decisions:

**`crates/net/dns/src/resolver.rs` — took upstream, and it fixes a real bug of
ours.** CreditChain had hand-written TXT extraction that took the *first* TXT
record and its *first* character-string. Upstream's `find_txt_entry()` skips
records that are not ENR-tree entries and rejoins character-strings past 255
bytes. Our version would have broken on any domain that also carries SPF, DKIM,
or verification TXT records — which is most domains — and truncated any ENR tree
entry over 255 bytes. This matters because CreditChain's node discovery is
DNS-based.

**`deny.toml` — took upstream's advisory list, and it checks out.** Ours ignored
`RUSTSEC-2026-0002` (lru) and `RUSTSEC-2026-0097` (rand unsoundness); upstream's
list drops both and adds `RUSTSEC-2026-0247` (bitmaps, via imbl).

Verified against the RustSec database rather than assumed, because the merged
lockfile still contains `lru 0.16.4` (pulled by `alloy-provider 2.3.0` — our old
comment blamed discv5 and was already stale) and moved `rand` 0.10.2 → 0.10.1:

| Advisory | Patched versions | Ours | Affected? |
|---|---|---|---|
| RUSTSEC-2026-0002 (lru) | `>= 0.16.3` | 0.16.4, 0.18.0 | no |
| RUSTSEC-2026-0097 (rand) | `>= 0.10.1`, `>= 0.9.3`, `>= 0.8.6` | 0.8.6, 0.9.4, 0.10.1 | no |

Both dropped ignores were genuinely unnecessary — `0.16.4` was already past the
lru fix, and every `rand` in the tree sits on a patched line. The old entries
were suppressing advisories that no longer applied.

`cargo deny check advisories` itself **cannot run on this workspace**:
cargo-deny 0.20.2 panics inside `krates` with `unable to locate
serde-bincode-compat for alloy-genesis@2.3.0`. That is a tooling bug against this
dependency graph, not a finding. The table above is the manual substitute and
should be replaced by a real run once cargo-deny handles the graph.

**`crates/node/core/src/args/trace.rs`** — upstream replaced hard-coded trace
defaults with a configurable `DefaultTraceValues`. Took the feature; changed the
one default `service_name` from `"reth"` to `"creditchaind"`.

**`crates/cli/commands/src/download/mod.rs`** — took upstream's snapshot API
refactor, then repointed the constants at `snapshots.creditchain.org`.

**`.github/workflows/book.yml`** — kept our branch guards *and* took upstream's
SHA-pinned actions (supply-chain hardening).

**`.github/workflows/unit.yml`** — kept our test-sharding matrix *and* took
upstream's `permissions: contents: read` least-privilege block.

**`Makefile` / `Dockerfile.reproducible`** — kept the `creditchaind` binary name,
adopted upstream's packaging of `LICENSES` and `README.md` into release artifacts.

## Removed by upstream (accepted)

Upstream deleted or restructured these; our only changes to them were lint and
branding, so the deletions were accepted:

| Path | Upstream's disposition |
|---|---|
| `crates/trie/sparse/src/parallel.rs` | replaced by the `arena/` module |
| `crates/net/eth-wire/src/eth_snap_stream.rs` | renamed to `eth_snap.rs` |
| `bin/reth-bench/` | removed from the tree |
| `crates/rpc/rpc-testing-util/` | removed from the tree |
| `crates/node/core/src/args/benchmark_args.rs` | removed with reth-bench |
| `docs/vocs/sidebar-cli-reth.ts` | superseded by our `sidebar-cli-creditchaind.ts` |

`Makefile` targets for `reth-bench` were dropped accordingly.

## Defaults that changed

These alter node behaviour with no configuration edit. Worth knowing before a
validator runs this build:

| Change | Effect |
|---|---|
| `perf(engine): keep 5 in-memory blocks by default (#26462)` | memory footprint and reorg-depth handling |
| `perf(net): raise default ECIES write flush boundary to 64KiB (#26348)` | RLPx write batching |
| `feat: enable gmp by default (#25512)` | GMP bignum arithmetic on by default; `gmp-mpfr-sys` vendors and builds its own source, so no system package is needed (verified: it compiled here) |
| `chore: default to min-trace-logs (#23851)` | less verbose tracing by default |
| `fix(rpc): align eth_fillTransaction default maxFeePerGas with go-ethereum (#25258)` | RPC fee estimation now `2 * base_fee + tip` |
| `feat(net): add default-off snap/2 negotiation (#25774)` | new protocol, off unless enabled |
| `feat(net): default BAL requests to optional (#24095)` | block access list requests no longer mandatory |

## New build requirement: LLVM

`v2.5.2` introduces `revmc` — an EVM JIT — and puts the `jit` feature in the
**default** feature set of the `creditchaind` binary (package `cc-cli`).
`revmc-llvm` shells out to `llvm-config`, and `llvm-sys-221` requires **LLVM 22**.
Pre-merge `Cargo.lock` had **0** revmc entries; `v2.5.2` has **37**.

**Upstream shipped the build-environment change along with the feature, and the
merge brought it with them.** `.github/scripts/install_llvm.sh` (default version
22, with `ubuntu` and `macos` paths) arrived in `feat: integrate revmc JIT
(#23230)`, is invoked by the `book`, `compact`, `check-alloy` and bench
workflows, and all three Dockerfiles install it. So CI and container builds
handle this on their own.

What is *not* handled is a developer machine, which needs a one-off:

```bash
.github/scripts/install_llvm.sh macos     # or: ubuntu
```

**The `jit` feature gates EVM internals only** (`crates/ethereum/evm/src/lib.rs`,
`factory.rs`) — it adds no CLI arguments. A binary built without it produces
byte-identical CLI reference documentation, so `make update-book-cli` can be run
from a no-jit build without diverging from what CI generates.

### Should the JIT be enabled?

The build environment is a solved problem, so this reduces to a real engineering
question rather than a blocker: **a JIT compiles consensus-critical code paths at
runtime.** It arrived here as a default-feature flip inside a version bump, not
as a deliberate adoption. It deserves its own soak on a non-validating testnet
node, with execution results compared against an interpreter build, before a
validator runs it.

Until that soak happens, the conservative build is every default except `jit`:

```bash
cargo +stable build --release -p cc-cli --no-default-features \
  --features "jemalloc,otlp,otlp-logs,reth-revm/portable,js-tracer,\
keccak-cache-global,asm-keccak,gmp,min-trace-logs"
```

Note the `--no-default-features` form means every other default must be listed
explicitly or it is silently dropped.

## Verification status

- [x] All 41 conflicts resolved; no conflict markers remain in tracked files
- [x] CreditChain chain specs, genesis wiring, contracts, deploy tree intact
- [x] Branding re-applied (0 remaining `reth::cli` targets or `paradigmxyz/reth` URLs)
- [x] Every EL flag used by `deploy/` still exists in the merged CLI
- [x] `.github` workflow YAML parses
- [x] Resolution self-reviewed: diffing each resolved file against pure `v2.5.2`
      shows only intentional branding deltas, no stray logic changes
- [x] **`cargo +stable check --workspace --all-targets --exclude cc-cli` — 0 errors** (5m51s)
- [x] **`cargo +stable check -p cc-cli --all-targets` without `jit` — 0 errors**
- [ ] `cc-cli` *with* `jit` — blocked on LLVM 22 (see build requirements above)
- [ ] `cargo +nightly fmt --all --check`
- [ ] `cargo clippy --workspace --all-features`
- [ ] `cargo deny check` (we dropped two RUSTSEC ignores; confirm they are truly resolved)
- [ ] `make update-book-cli` — the `book` CI job regenerates the CLI reference
      pages and runs `git diff --exit-code`; it will fail until they are committed
- [ ] Testnet soak on a non-validating node before any validator takes this build

### Runtime verification (2026-09-07)

Static checks cannot see a state-root divergence, and this merge rewrote the
sparse trie into an arena representation. So the merged binary was run.

**It produces blocks, and it does so through the rewritten trie:**

```
INFO engine::tree::payload_validator: State root job finished
     strategy="sparse-trie" state_root=0xf09d8f7d… elapsed=5.125µs
INFO reth_node_events::node: Canonical chain committed number=19 …
```

**It executes the full application workload.** `AgentClearing` and
`AgentSpendVault` were deployed to a node running this binary and driven through
the complete lifecycle at 5,000 payments:

| Step | Result |
|---|---|
| EIP-712 digest, offline vs on-chain | identical |
| Deposit, channel open | ok |
| 5,000 off-chain payments | 25.9 s, 0 transactions, 0 gas |
| Redemption of the whole run | one transaction, 93,303 gas |
| Two channels netted | one transaction, 102,322 gas |
| Close window enforced | `closeChannel` reverted inside the hour |
| Withdraw and solvency | held ≥ owed at every step |

Sustained multi-agent load (`demo/agent_load.sh`) then ran the same shape of
traffic across independent agents paying each other round-robin, asserting each
payee was credited exactly its voucher total and checking contract solvency
across the whole population.

**Still not proven by any of this:** that the binary agrees with the *existing*
Argos chain. Joining it requires the generator's genesis, and
`genesis/testnet.json` produces a different chain (see below) — so a
sync-from-network soak needs the authoritative genesis file first. That remains
the last gate before a validator takes this build.

### One real defect the compile caught

`bin/reth/tests/it/main.rs` auto-merged cleanly and was still wrong. CreditChain
had renamed the test helper `reth_ok` → `creditchaind_ok`; upstream added a new
`--force` download test that calls `reth_ok`. Git combined our renamed definition
with upstream's new call site and reported no conflict, because neither side
touched the same lines.

This is the failure mode a rename-based fork produces, and it is invisible to
`git status`, to the conflict list, and to any review that reads only the
conflicted files. Only the compiler found it. Nothing below the compile gate
should be treated as verified.

The build with all default features minus `jit` is the configuration that has
actually been proven here, and it is the one to ship until LLVM 22 is a
deliberate decision:

```bash
cargo +stable build --release -p cc-cli --no-default-features \
  --features "jemalloc,otlp,otlp-logs,reth-revm/portable,js-tracer,\
keccak-cache-global,asm-keccak,gmp,min-trace-logs"
```

## Not merged

`upstream/main` carries 183 commits beyond `v2.5.2`. They are deliberately out of
scope: unreleased work does not belong under a chain preparing for mainnet. Revisit
at the next upstream release.
