# Who controls CreditChain — testnet, then mainnet

Written 2026-08-31, answering two questions: is there a key ceremony for testnet,
and who actually governs these networks.

---

## 1. Is there an offline key ceremony for testnet?

**No — deliberately, and the distinction matters.**

The testnet role keys and the 60 validator keys were generated with
`cast wallet new` and `lcli mnemonic-validators` **on a networked machine**, and
the mnemonic is stored in a file on the operator's laptop. For testnet that is
correct: a ceremony exists to protect *value*, and test CCC has none.

Doing the same on mainnet would be catastrophic.

**But the procedure should still be rehearsed on testnet**, because a launch is a
bad time to perform a multi-party ritual for the first time. Rehearse the
sequence — offline machine, witnesses, address read back twice by two people,
written record — using throwaway keys. Practise the choreography, not the secrecy.

---

## 2. Who controls testnet today? You do. Completely.

All 60 validators run on five hosts under one operator:

| Host | Validators | Share |
|---|---|---|
| hostA3, hostA2, hostA1, hostB1, hostB2 | 12 each | 20% each |

No node holds ≥1/3, which is a real property — it means no *single machine* can
halt finality. But every machine has the same operator, so **one person can stop
the chain, rewrite it, or change any parameter.**

That is a fact about the current state, not a criticism. It is also true of
mainnet as it would launch today. It is worth stating plainly because the word
"decentralised" cannot honestly be used until it stops being true.

So: **testnet today is a staging environment, not a decentralisation rehearsal.**

---

## 3. The actual choice

**Option A — keep testnet operator-run.** Perfectly reasonable. It gives
developers a stable place to deploy, and it is what most projects do.
What it does *not* do is rehearse governance.

**Option B — open testnet validation to outsiders.** Slower and messier, and it
is the only way to rehearse the thing most likely to go wrong on mainnet:
governance under genuine disagreement, with people who do not report to you.

**Recommendation: B, and start small.** Not because decentralisation is a virtue
in itself, but because of a practical asymmetry — every other mainnet risk can be
rehearsed alone (consensus, allocation, upgrades, recovery). Governance under
disagreement cannot. If mainnet is the first time an external party votes against
you, you will be learning the mechanism and the politics simultaneously, with real
money watching.

Three or four independent validators is enough to learn from. They do not need to
be strangers — other teams, an infrastructure partner, a friendly project.

---

## 4. How governance actually works, mechanically

The contracts already exist (`contracts/treasury/`, 35 tests). They are not
theoretical:

| Layer | Contract | What it decides |
|---|---|---|
| **Protocol authority** | `Timelock` | Parameter changes, upgrades. Every action is queued on-chain and visible for the full delay before it can execute. |
| **Who may propose** | `Multisig` | M-of-N. Owner changes need the same quorum as spending, so the threshold cannot be quietly weakened. |
| **Who may validate** | `StakingReserve` | The governor — expected to be the Timelock — registers validators and their allocation. |

The delay is the part that matters. A multisig makes governance require
*agreement*; a timelock makes it require **agreement announced in advance**. Without
it, "governance" is an admin key with extra signatures — and users cannot tell the
difference until it is used against them.

### Testnet settings (permissive, for learning)

- Timelock delay: **1 hour** — long enough to observe, short enough to iterate.
- Multisig: 2-of-3 among the operator and early partners.
- Validator registration: **open on request**, stake funded by the faucet.
- Explicit expectation: **testnet may be reset.** Say so loudly, so nobody builds
  a business on it.

### Mainnet settings (conservative, for value)

- Timelock delay: **48 hours minimum** — time for an exchange or a large holder to
  react to a change they dislike.
- Multisig: 4-of-7 or wider, with signers who are not all in one jurisdiction or
  one company.
- Validator registration: **staking-gated**, with real economic weight and
  slashing.
- Emergency powers: as narrow as possible, and every use published afterwards.

---

## 5. The honest progression

**Phase 1 — now.** Operator runs everything. Governance contracts deployed on
testnet and exercised, but every signer is internal. Say "operated by CreditChain"
in public materials, not "decentralised".

**Phase 2 — before mainnet.** Invite 3–4 external testnet validators. Route real
decisions through the timelock in public: a parameter change, a validator
addition, a contract upgrade. **Deliberately lose one vote**, or at least run one
where the outcome is not pre-agreed. That is the rehearsal.

**Phase 3 — mainnet.** Launch with a governance structure that has *already been
used*, by people who have already disagreed with each other and resolved it. That
is what makes "governed" a description rather than a promise.

---

## 6. What this means for the mainnet key ceremony

The ceremony produces the addresses that go into genesis. Those addresses should
**not** be a set of EOAs held by one person — `build-genesis.py` already refuses
that for high-value roles. They should be:

- **governance** → a Timelock whose proposer multisig has signers from more than
  one organisation;
- **management** → a multisig, separate signers from governance, so an operational
  compromise cannot change protocol rules;
- **team** → a vesting contract, so the schedule is verifiable by outsiders;
- **staking** → the reserve contract, with the Timelock as governor.

Which means the ceremony is not one person on an offline laptop. **It is several
people, and it needs scheduling.** That is the practical reason to start Phase 2
now rather than after mainnet is otherwise ready — the people have to exist before
the ceremony can happen.

---

## 7. What I cannot do

I can build and verify all of the mechanism. I cannot choose your signers, decide
how much control to give up, or judge the legal shape of a governance body in your
jurisdictions. Those are yours, and the securities-law question in
`internal/MAINNET-LAUNCH-ADVISORY.md` §6 applies here too — a governance token and
a governance *process* are treated differently in different places.
