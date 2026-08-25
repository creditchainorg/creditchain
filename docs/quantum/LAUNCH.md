# Quantum Launch — go-to-market

## The strategic bet

Every chain that touches this topic overclaims. "Quantum-resistant blockchain"
is one of the most abused phrases in the industry, and the audience that matters
— cryptographers, security engineers, serious funds — has learned to discount it
on sight.

**So the differentiator is not the claim. It is the disclosure.**

Our headline says CreditChain's base layer is *not* quantum resistant. That single
sentence does more competitive work than any superlative, because:

1. **It is verifiable, and everything next to it inherits that credibility.** A
   reader who checks the first claim and finds it honest will believe the second.
   A reader who catches one overclaim discounts the entire page.
2. **It is a claim no competitor can copy** without admitting the same about
   themselves.
3. **It survives scrutiny.** Cryptographers will read this. The failure mode for
   a hype launch is a teardown thread; the failure mode for an honest launch is
   being called boring. Only one of those is recoverable.

The narrow claim we make instead is defensible and still first:

> The first chain where an AI agent's spending authority has a post-quantum kill
> switch — live, measured, and reproducible from an independent implementation.

---

## Primary post

> **We shipped post-quantum recovery on CreditChain. First, the part nobody says out loud:**
>
> **CreditChain is not quantum resistant.** It runs secp256k1 ECDSA — same as
> Ethereum, same as every production EVM chain. Shor's algorithm breaks it. Anyone
> telling you their EVM chain is quantum-safe today is selling you something.
>
> Here's what we actually built, and why it isn't theatre.
>
> **The mistake most PQ designs make:** bolting a post-quantum key next to an ECDSA
> key, both with full authority. That buys nothing. An attacker who forges your ECDSA
> key just… uses it. The PQ key is decoration.
>
> **Security only appears when the two are asymmetric.** So in `QuantumGuard`:
>
> - the ECDSA controller has **no withdraw function at all** — not restricted, *absent
>   from the ABI*
> - it **cannot** change the recovery address; that's frozen when the guardian is armed
> - it's **capped** by a rolling outflow limit
> - only the post-quantum key can sweep funds and rotate control
>
> An attacker with a fully forged ECDSA key cannot withdraw, cannot redirect recovery,
> and cannot exceed the cap. Worst case goes from *instant total loss* to *bounded leak,
> then guaranteed recovery*.
>
> **The crypto:** WOTS+ (RFC 8391) over keccak256 — the one-time signature inside NIST's
> SLH-DSA. Hash-based, so Shor doesn't apply and Grover only halves the margin. keccak256
> is already an EVM opcode, so it needs **no precompile and no hard fork**.
> 2,144-byte signatures. **252,606 gas.**
>
> **On correctness:** we wrote a second implementation in Python, straight from the RFC.
> The Solidity verifier accepts the Python signer's bytes cold; the Solidity signer
> reproduces the Python signature byte-for-byte. Testing a verifier against your own
> signer proves the two agree — not that either is right.
>
> **It's live.** Not a testnet demo we describe — a transaction you can fetch:
> 21 CCC swept, mandates revoked, control rotated, one-time key burned, replay rejected
> with `GuardianConsumed()`.
>
> It's **unaudited**. Test CCC has no monetary value. The per-layer status — including
> the layers that are still vulnerable, and the one config fix we haven't shipped yet —
> is in the repo.
>
> forge.creditchain.org

## Secondary angles

**For the crypto-security audience** — lead with the asymmetry argument and the
independent-reference methodology. This crowd has seen a hundred PQ announcements
and zero cross-implementation test vectors.

**For the AI-agent audience** — lead with the composition: an agent operates under a
chain-enforced mandate, and the mandate's kill switch outlives the cryptography that
protects the human's key.

**For builders** — lead with the gas number and the absence of a fork requirement.
"You can verify a post-quantum signature on an EVM chain today for 252k gas, no
precompile" is the fact people will repeat.

## The self-own that is also the strongest proof

Publish the measurement that embarrasses us: our own public RPC negotiates X25519 and
**rejects** `X25519MLKEM768`, so harvest-now-decrypt-later applies to our RPC traffic
right now. It's a config change, no fork needed, and we found it by testing rather than
assuming.

Shipping that finding alongside the launch is the single most credible thing on the
page. It proves the audit was real. Fix it, then publish the before/after.

## Distribution

1. Repo docs first — the post links to `QUANTUM-RESISTANCE.md`; the depth is the proof.
2. `forge.creditchain.org` quantum section — live, with the real break-glass transcript.
3. Post to the security-technical channels before the crypto-promotional ones. If
   cryptographers don't object, the rest follows. If they do, we learn something before
   it's amplified.
4. Submit WOTS+ gas numbers and the verifier as a standalone contribution. The library is
   useful to chains that will never touch CreditChain, and that is how it travels.

## Hard rules — do not break these for engagement

- Never say "quantum resistant" about CreditChain without the base-layer qualification.
- Never imply a CRQC is imminent or put a date on it. Nobody knows.
- Never call unaudited code secure.
- Never drop the test-CCC-has-no-value or not-investment-advice notes.
- If a number is quoted, it must be reproducible from the repo.

The moment we win an argument by overstating, we lose the only asset that makes this
launch work.
