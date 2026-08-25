# Quantum Resistance on CreditChain

**Status: Phase 0 shipped and live on public testnet.**
Last verified 2026-08-25 against chain `2026042404` at height ~2,093,051.

---

## 1. The honest status, first

Marketing in this area is unusually dishonest. Chains routinely claim to be
"quantum resistant" when they run the same secp256k1 ECDSA as everyone else. So
before any claim, here is what CreditChain actually runs today, layer by layer,
with what a cryptographically-relevant quantum computer (CRQC) would do to each.

| Layer | Algorithm today | Under a CRQC | Verified how |
|---|---|---|---|
| Transaction signatures | secp256k1 ECDSA | **Broken.** Shor recovers the private key from the public key. | `Cargo.toml` → `k256`, `secp256k1` |
| Address derivation | `keccak256(pubkey)[12:]` | 160-bit; Grover → ~80-bit. Protects an address only until it first *sends*, after which the pubkey is public forever. | EVM semantics |
| State / receipts / trie | keccak256 | Grover → ~128-bit. **Fine.** No action needed. | EVM semantics |
| Consensus signatures (PoS staging) | BLS12-381 | **Broken.** Shor. Validator keys forgeable. | creditbeacon (Lighthouse fork) |
| P2P transport | RLPx over secp256k1 ECIES | Broken. Traffic is public anyway, but node identity becomes spoofable. | devp2p |
| Public RPC TLS | TLS 1.3, **X25519** key exchange | **Harvest-now-decrypt-later applies today.** `X25519MLKEM768` is *rejected* by the endpoint. | `openssl s_client -groups X25519MLKEM768` → handshake failure |
| **Break-glass authority** | **WOTS+ over keccak256** | **Resistant.** Grover only: ~128-bit. | live on testnet, §4 |

**So: CreditChain's base layer is not quantum resistant today, and neither is
any other production EVM chain.** Claiming otherwise would be false. What we
have shipped is the first layer where real quantum resistance can be delivered
*without* a hard fork — and it is live, not a whitepaper.

### Why this is not a reason to panic, and not a reason to wait

Two things are true at once:

- **No CRQC exists.** Public estimates for breaking secp256k1 cluster around
  millions of physical qubits; current devices are orders of magnitude away.
  Anyone giving you a confident date is guessing.
- **The migration is the slow part, not the cryptography.** Every wallet, every
  library, every hardware signer, every deployed contract assumes ECDSA. That
  ecosystem turn takes years. A chain that starts when the CRQC is announced has
  already lost, because *every address that ever transacted is retroactively
  exposed the moment the machine exists.*

Signatures are also a different risk shape from encryption. For encryption,
"harvest now, decrypt later" means today's traffic is already at risk — which is
exactly why the TLS row above matters now. For signatures, the risk is future
forgery against public keys that are already published. Both point the same way:
start early, ship incrementally, do not oversell.

---

## 2. The design principle

> A post-quantum key is worth nothing if it can only do what the classical key
> can already do. The attacker simply uses the one they broke.

This is the mistake most "add a PQ key" designs make. Bolting an ML-DSA key
alongside an ECDSA key, both with full authority, buys nothing: an attacker
holding a forged ECDSA key ignores the PQ key entirely.

Security only appears when the two authorities are **asymmetric** — when the
post-quantum key can do something the classical key *cannot*. CreditChain's
`QuantumGuard` is built entirely around that asymmetry:

| | controller (ECDSA) | guardian (WOTS+, post-quantum) |
|---|---|---|
| open / fund mandates | yes, **capped by a rolling outflow limit** | — |
| allowlist recipients | yes | — |
| withdraw funds | **no such function exists** | yes — sweeps everything |
| change the recovery address | **no** | — (it is fixed at arm time) |
| replace the controller | **no** | yes |
| revoke every mandate | no | yes |

An attacker who has fully forged the ECDSA controller key therefore **cannot
withdraw, cannot redirect recovery, and cannot exceed the outflow cap**. The
worst case degrades from *instant total loss* to *a bounded leak, then guaranteed
recovery*. That is a real, checkable security property — and it is the honest
limit of what the contract claims.

### Why the recovery address is frozen at arm time

A break-glass signature is public the instant it hits the mempool, so it must be
assumed stolen. Because the destination was committed when the key was armed and
is bound into the signed digest, a stolen signature can only do the one thing the
owner already wanted: move funds to the owner's own address. **Anyone may relay
it; nobody can redirect it.** Front-running is neutralised by construction, not
by a mitigation that has to hold up under pressure.

---

## 3. Why WOTS+, and not Dilithium

NIST standardised three post-quantum signature schemes. For an EVM chain today:

| Scheme | On-chain verification | Why not / why yes |
|---|---|---|
| ML-DSA (FIPS 204, Dilithium) | needs a **precompile** — NTT-heavy, impractical in Solidity | Best long-term answer. Requires a hard fork. **Phase 2.** |
| SLH-DSA (FIPS 205, SPHINCS+) | ~7–30 KB signatures | Hash-based and conservative, but signature size is punishing on-chain. |
| **WOTS+** (SLH-DSA's own internal OTS) | **2,144 bytes, ~253k gas, keccak256 only** | Ships **today, with no consensus change**. |

WOTS+ is the component SLH-DSA is built from, so this is not an exotic choice —
it is the mature, standardised core of a NIST scheme, used at the layer where the
EVM can already afford it. keccak256 is a native opcode, so no new precompile and
no fork is required.

Its one real constraint is that **a WOTS+ key signs exactly once**. A second
signature leaks enough chain material to forge others. Rather than fight that,
the design leans on it: a break-glass key *should* be single-use. You use it once,
in the worst moment of the account's life, and it burns itself on the way out.

**Parameters:** n = 32 bytes, w = 16 → 64 message digits + 3 checksum digits =
67 hash chains. The checksum is what prevents forgery: chains only run forward,
so an attacker can always *raise* a message digit — but the checksum digits are
the complement, so raising any message digit *lowers* the checksum and forces a
chain backward, which is a hash preimage.

### Measured cost

| Operation | Gas |
|---|---|
| `WOTSPlus.verify` (2,144-byte signature) | **252,606** |
| `breakGlass` incl. revoke + sweep, 1 mandate | 359,564 (test) / **415,323 (live)** |

The first implementation cost 510,861 gas; hashing out of two fixed assembly
scratch buffers instead of allocating via `abi.encodePacked` removed ~3,000
allocations and cut it **51%**. The reference vectors below re-verified
byte-identically afterwards, which is what makes that rewrite trustworthy.

---

## 4. What is live right now

Deployed and exercised end-to-end on public testnet (`chainId 2026042404`).
Test CCC has **no monetary value**.

| | |
|---|---|
| `AgentSpendVault` | `0x446e7636a5Fa9af46c3718719e465B547248bF62` |
| `QuantumGuard` #1 | `0x505d59ffFd312983Cc0eD114d7F117B91520d742` |
| break-glass tx | `0x4b717793287abece001628aa4afc236dbe5af8eac779d6ae6aed60adf9238ccc` |

Observed on-chain, not asserted in a test:

```
guard balance      21 CCC  →  0 CCC
recovery balance    0 CCC  →  25 CCC   (21 swept + 4 pulled back from the revoked mandate)
controller                 →  rotated to the recovery address
frozen                     →  true
isArmed                    →  false          (one-time key burned)
mandate.revoked            →  true

replay same valid signature →  0x56248192  = GuardianConsumed()
controller over-spend       →  0x7afd1a0d  = OutflowExceeded(uint256,uint256)
controller re-arm           →  0x98a838c8  = AlreadyArmed()
```

Every selector above was decoded with `cast sig`, not assumed.

---

## 5. How the correctness is established

A verifier tested only against a signer written by the same author proves the two
agree — not that either implements WOTS+. So the chain of evidence is deliberately
broken into independent links:

1. **`reference/wotsplus.py`** — written from RFC 8391, independent of the
   Solidity. Includes a pure-Python Keccak so it depends on no wheel; validated
   against the known `keccak256("") = 0xc5d2460186f7233c…`.
2. **`WOTSPlus.t.sol`** — the Solidity verifier accepts the *Python signer's*
   signature bytes cold, plus bit-flip fuzzing (512 runs), wrong-key, wrong-seed,
   wrong-digest, and an explicit **forward-chain forgery attempt**.
3. **`WOTSPlusSigner.t.sol`** — pins the test-only Solidity signer to the Python
   reference: same seeds must produce a **byte-identical** 2,144-byte signature.
4. **`QuantumGuard.t.sol`** — asserts the *absence* of capabilities, not only their
   presence: no withdraw path in the ABI, no re-arm, outflow enforced, cross-guard
   and cross-chain replay rejected, one-time enforcement, relayer cannot capture value.

**85/85 tests pass**, including the pre-existing suite (no regressions).

> **Unaudited.** This code has had no external security review. The audit RFP in
> `deploy/audit/AUDIT-RFP.md` should be extended to cover `src/quantum/`.

---

## 6. The wallet: `ccq`

`tools/ccq/ccq.py` — stdlib-only, so it runs on an air-gapped machine with no
installs and no network.

```bash
ccq new     --label main                 # derive a guardian key from a passphrase
ccq arm     --label main --guard 0x… --recovery 0x…
ccq sign    --label main --digest 0x…    # OFFLINE. no sockets are opened.
ccq verify  --digest 0x… --sig 0x… …     # check before you broadcast
```

Design rules it follows, and why:

- **In passphrase mode nothing secret is written to disk.** The key is re-derived
  each run via PBKDF2-HMAC-SHA512 at 600,000 iterations (current OWASP guidance)
  with a domain-separated salt. The only copy lives in the owner's head or on paper.
- **`sign` opens no socket.** Carry the digest in, carry the signature out. The
  secret never touches a networked machine.
- **It never reads or stores a secp256k1 key.** This wallet cannot move funds,
  so a compromised copy of it cannot either.
- **It refuses to sign twice** under the same label. A second WOTS+ signature is a
  break, not an inconvenience.

---

## 7. Roadmap

**Phase 0 — application layer. SHIPPED.**
WOTS+ verifier, `QuantumGuard`, `ccq` wallet. No consensus change. Live on testnet.

**Phase 1 — XMSS: lift the one-time limit.**
A Merkle tree over many WOTS+ keys gives ~2^10–2^20 signatures per key with an
authentication path (~`h × 32` extra bytes). Still keccak-only, still no fork.
Turns break-glass into a general-purpose post-quantum account authority.

**Phase 2 — ML-DSA precompile.** *Requires a hard fork.*
FIPS 204 verification at a reserved precompile address, bringing PQ signature
checks to ~ordinary-transaction cost and letting contracts verify signatures from
standard PQ libraries. This is the real long-term answer.

**Phase 3 — native PQ transaction type.**
An EIP-2718 typed transaction carrying an ML-DSA signature, with accounts derived
from a PQ public key hash. Full base-layer resistance. Depends on Phase 2 and on
wallet/tooling support — the ecosystem work dwarfs the protocol work.

**Phase 4 — consensus and transport.**
Validator keys BLS12-381 → PQ; RLPx handshake; and the one item that needs no fork
at all and should not wait: **enable `X25519MLKEM768` on the public RPC endpoints**,
closing the harvest-now-decrypt-later gap measured in §1. This is an nginx/OpenSSL
configuration change and is the cheapest real win remaining.

### Sequencing rationale

Phases 0 and 1 ship value with **no fork and no ecosystem coordination**, which is
why they come first. Phase 4's TLS item is out of order on purpose — it is a config
change addressing a risk that is *live today*, so it should land alongside Phase 1
rather than waiting for the consensus work it is grouped with.

---

## 8. What we will not claim

- We will **not** say CreditChain is "quantum resistant" without qualification.
  The base layer is not, and §1 says so in the first table.
- We will **not** imply a CRQC is imminent. Nobody knows.
- We will **not** describe unaudited code as secure.
- Test CCC has no monetary value, and nothing here is investment advice.

The competitive claim we *will* make is narrower and defensible:

> **CreditChain is the first chain where an AI agent's spending authority has a
> post-quantum kill switch — live, measured, and reproducible from an independent
> reference implementation.**
