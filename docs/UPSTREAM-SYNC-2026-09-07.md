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

**`deny.toml` — took upstream's advisory list.** Ours still ignored
`RUSTSEC-2026-0002` (lru) and `RUSTSEC-2026-0097` (rand unsoundness); upstream
has since resolved both by dependency upgrade and dropped the ignores. Keeping
ours would have suppressed advisories that no longer need suppressing.
`cargo deny` should be re-run to confirm.

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

**This sync adds a hard dependency on LLVM to build the node binary.**

`v2.5.2` introduces `revmc` — an EVM JIT compiler — and puts the `jit` feature in
the **default** feature set of the `creditchaind` binary
(`bin/reth/Cargo.toml`, package `cc-cli`). `revmc-llvm`'s build script shells out
to `llvm-config`, so the build now fails without LLVM present:

```
error: failed to run custom build command for `revmc-llvm v0.1.0`
  failed to run llvm-config: No such file or directory (os error 2)
  failed to run llvm-config-22: No such file or directory (os error 2)
  no llvm-config found
```

Pre-merge `Cargo.lock` contained **0** revmc entries; `v2.5.2` contains **37**.
The build script probes `llvm-config` then `llvm-config-22`, so it wants LLVM 22.

This affects every build surface and must be handled before this branch is used
to produce a node binary:

- **Docker images** (`Dockerfile`, `Dockerfile.reproducible`) need LLVM installed
- **CI** needs LLVM installed
- **Developer machines** need LLVM

Two options, and the choice is a real one:

1. **Install LLVM 22 and keep the JIT.** It is a genuine performance feature and
   upstream made it default deliberately. Costs a heavier build environment.
2. **Build with `--no-default-features` and re-add every default except `jit`.**
   Keeps the toolchain light; forgoes the JIT. The other defaults
   (`jemalloc`, `otlp`, `otlp-logs`, `reth-revm/portable`, `js-tracer`,
   `keccak-cache-global`, `asm-keccak`, `gmp`, `min-trace-logs`) must be listed
   explicitly or they are silently lost.

Option 2 is the conservative choice for a chain approaching mainnet: it changes
nothing about execution semantics, and the JIT can be adopted separately once
the build environment is deliberately upgraded and the JIT is soak-tested. **A
JIT compiles consensus-critical code paths at runtime; it deserves its own
evaluation rather than arriving as a side effect of a version bump.**

## Verification status

- [x] All 41 conflicts resolved; no conflict markers remain in tracked files
- [x] CreditChain chain specs, genesis wiring, contracts, deploy tree intact
- [x] Branding re-applied (0 remaining `reth::cli` targets or `paradigmxyz/reth` URLs)
- [x] `.github` workflow YAML parses
- [ ] `cargo check --workspace --all-targets` — **see below**
- [ ] `cargo +nightly fmt --all --check`
- [ ] `cargo clippy --workspace --all-features`
- [ ] `make update-book-cli` (CLI reference pages need regenerating — the `book`
      CI job enforces this and will fail until they are)
- [ ] Testnet soak on a non-validating node before any validator takes this build

**The workspace has not yet been proven to compile.** The declared MSRV is 1.95
and the machine's default toolchain is 1.92, so builds must use `cargo +stable`
(1.97.1). Nothing below the compile gate should be treated as done.

## Not merged

`upstream/main` carries 183 commits beyond `v2.5.2`. They are deliberately out of
scope: unreleased work does not belong under a chain preparing for mainnet. Revisit
at the next upstream release.
