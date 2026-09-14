---
eip: <to be assigned>
title: Agent Clearing and Settlement (ACS)
description: Collateralised payment channels with cumulative vouchers, letting autonomous agents transact at zero marginal on-chain cost and settle net positions in one operation.
author: CreditChain Research Team
discussions-to: https://www.creditchain.org
status: Draft
type: Standards Track
category: ERC
created: 2026-09-06
requires: 712
---

## Abstract

This standard defines `AgentClearing`, four layers of financial plumbing —
**deposit, payment, clearing, settlement** — for machine-speed commerce between
autonomous agents.

A payer posts collateral once. It then issues off-chain EIP-712 vouchers, each
stating a **cumulative running total** rather than an increment. Because a newer
voucher supersedes every older one, a payee holding the latest voucher holds the
entire payment history in 65 bytes, and redeems whenever it suits them. Many
thousands of payments collapse into a single on-chain operation, and obligations
across many channels net into one balance write per account.

## Motivation

[ERC-AGM](./ERC-AGM.md) gives an agent bounded, revocable spending authority and
settles each payment on-chain. That is the right shape for a payment a human
would notice: a subscription, a purchase, a transfer.

It is the wrong shape for how agents actually transact. Agent commerce is
thousands of sub-cent payments to a handful of counterparties, at machine speed —
per inference call, per row retrieved, per second of compute. At roughly 42,000
gas per settled spend, **an agent paying per API call spends more on gas than on
the service.** No fee schedule fixes this; the primitive has to change.

The change is to stop settling *payments* and start settling *positions*.

This is not a new idea in payments — it is how correspondent banking, card
networks, and securities clearing have always worked, and why a card swipe does
not move central bank reserves. What is new is that the parties are agents: they
can sign a voucher every few milliseconds, they have no business hours to batch
around, and they can verify their own counterparty risk continuously instead of
trusting an intermediary to do it once a day.

## Specification

The key words MUST, MUST NOT, SHOULD, and MAY are to be interpreted as described
in RFC 2119.

### Layer 1 — Deposit

Collateral is held in two balances per account:

| Field | Meaning |
|---|---|
| `available(address)` | free collateral; withdrawable |
| `reserved(address)`  | committed to open channels; **not** withdrawable |

```solidity
function deposit() external payable;
function withdraw(uint256 amount) external;
```

`withdraw` MUST revert if `amount > available[msg.sender]`. An implementation
MUST NOT allow reserved collateral to be withdrawn while a channel can still be
redeemed against it. This is the property that makes an off-chain voucher worth
anything.

### Layer 2 — Payment

```solidity
struct Channel {
    address payer;
    address payee;
    uint256 committed;   // collateral reserved against this channel
    uint256 redeemed;    // cumulative already paid out
    uint256 cap;         // highest total the payer may ever owe here
    uint64  expiry;      // after this the payee can no longer redeem
    uint64  closesAt;    // set when the payer starts closing
    bool    open;
}

function openChannel(address payee, uint256 commit, uint256 cap, uint64 expiry)
    external returns (uint256 channelId);
function fundChannel(uint256 channelId, uint256 amount) external;
```

`cap` MUST be greater than or equal to `commit`. A channel to `msg.sender` MUST
revert.

A **voucher** is an EIP-712 typed-data signature by the payer over:

```solidity
struct Voucher {
    uint256 channelId;
    uint256 cumulative;   // running total owed on this channel, not a delta
    bytes   signature;
}
```

with type hash `Voucher(uint256 channelId,uint256 cumulative)` and domain

```
EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)
name = "CreditChain AgentClearing", version = "1"
```

Issuing a voucher costs one local signature. It MUST NOT require any chain
interaction, and implementations MUST expose `voucherDigest(channelId,
cumulative)` so an off-chain client can be checked against the chain's own view.

**The cumulative total is the nonce.** A voucher for a total at or below
`redeemed` is stale; redeeming it MUST be a no-op that moves zero value, not a
revert. Replay is therefore structurally meaningless rather than defended
against.

### Layer 3 — Clearing

```solidity
function redeem(uint256 channelId, uint256 cumulative, bytes calldata signature) external;
function settleBatch(Voucher[] calldata vouchers) external;
```

`settleBatch` MUST accumulate net credits in memory across all supplied vouchers
and perform **at most one balance write per unique account**. It MUST skip stale
vouchers rather than reverting: one already-settled channel MUST NOT fail a batch
that is otherwise valid.

`settleBatch` MUST NOT be able to create an obligation that a single `redeem`
could not — batching is an efficiency, never an authority.

### Layer 4 — Settlement

```solidity
function startClose(uint256 channelId) external;   // payer only
function closeChannel(uint256 channelId) external;
uint64 constant CLOSE_WINDOW = 1 hours;
```

A payer MUST NOT be able to reclaim committed collateral immediately. After
`startClose`, redemption remains open until `closesAt = block.timestamp +
CLOSE_WINDOW`, so a voucher already in flight is never stranded. `closeChannel`
MUST revert before that point.

Unredeemed collateral returns to the payer's `available` balance on close, or
after `expiry`.

### Views

```solidity
function channel(uint256 channelId) external view returns (Channel memory);
function redeemableNow(uint256 channelId) external view returns (uint256);
function backing(uint256 channelId) external view returns (uint256 bps);
function voucherDigest(uint256 channelId, uint256 cumulative) external view returns (bytes32);
```

`backing` returns collateralisation in basis points; `10000` means every unit the
channel may owe is fully collateralised.

## Rationale

**Cumulative totals, not increments.** An increment-based voucher needs a nonce,
and a nonce needs replay protection, ordering, and gap handling. A cumulative
total needs none of that: the newest voucher is simply the largest number, the
payee only ever has to keep one, and an out-of-order or duplicated voucher is
arithmetically inert. This also means a payee that loses its voucher history
loses nothing as long as it kept the last one.

**Four layers, one contract.** The layers are separated by section, not by
address. The deposit layer is what gives a voucher value and clearing settles
directly against it; split across contracts, every settlement becomes a
cross-contract value movement — more reentrancy surface, non-atomic accounting,
and two places to audit for one invariant.

**Credit is opt-in and visible.** A `cap` above `committed` lets a payee accept
vouchers beyond the posted collateral. This is what makes netting save
*liquidity* rather than merely gas: agents in a payment cycle need not each
pre-fund the gross. But it is unsecured lending, so it is explicit at open time,
capped, and continuously measurable through `backing()`.

**A close window rather than a dispute game.** Fraud proofs assume a
counterparty who might publish a false state. Here the payer cannot: every
voucher is signed by the payer, so the worst they can do is refuse to help. A
fixed window is enough, and is far cheaper to reason about.

## Security Considerations

**Signature malleability.** For every valid `(r, s)` there is a valid
`(r, N - s)`. Implementations MUST reject `s` above `secp256k1n / 2` and MUST
reject `v` outside `{27, 28}`. The reference implementation and the reference
off-chain client both do.

**Cross-channel and cross-chain replay.** `channelId` is inside the signed
struct, so a voucher cannot be moved between channels. The domain separator binds
`chainId` and `verifyingContract`. It is **recomputed on every use rather than
cached at construction**, so a contract that survives a chain split does not keep
honouring vouchers signed for the other side of it.

**Credit risk is not eliminated, only measured.** A payee accepting vouchers
beyond `committed` can lose the uncollateralised portion. `backing()` exists so
this is an open-eyed decision. Payout is still capped at collateral: an
overdrawn channel pays what it holds and no more — the contract never invents
value to honour a voucher.

**Reentrancy.** Value leaves only through `withdraw`, behind a guard, after
balances are written.

**Griefing.** A payee that never redeems locks the payer's collateral only until
`startClose` plus `CLOSE_WINDOW`, or until `expiry`.

**Expiry is a hard cliff.** A payee that has not redeemed by `expiry` loses the
claim. Payees SHOULD redeem well before it, and SHOULD treat a channel nearing
expiry as a reason to stop extending service.

## Reference Implementation

- Contract: [`agent-finance/src/AgentClearing.sol`](https://github.com/creditchainorg/contracts/blob/main/agent-finance/src/AgentClearing.sol)
- Tests: 19 example tests plus 4 invariants
  ([`test/AgentClearing.t.sol`](https://github.com/creditchainorg/contracts/blob/main/agent-finance/test/AgentClearing.t.sol),
  [`test/AgentClearing.invariants.t.sol`](https://github.com/creditchainorg/contracts/blob/main/agent-finance/test/AgentClearing.invariants.t.sol))
- Off-chain client, zero dependencies:
  [`demo/voucher_client.py`](https://github.com/creditchainorg/contracts/blob/main/agent-finance/demo/voucher_client.py)
- On-chain proof:
  [`demo/prove_clearing_against_deployed.sh`](https://github.com/creditchainorg/contracts/blob/main/agent-finance/demo/prove_clearing_against_deployed.sh)

The invariants held over 12,800 randomised calls each: the contract holds at
least what its books owe; value is conserved across deposits and withdrawals; no
channel pays out more than it committed; and reserved collateral always matches
the sum of open channels.

### Deployment

| Network | Chain ID | Address |
|---|---|---|
| CreditChain Testnet | 2026042404 | `0x741a5E7DA765069Fda4B415a73f21FfF204a5B75` (source-verified) |

Testnet only. Test CCC has no monetary value. There is no mainnet deployment.

### Measured on the testnet

1,000 payments issued off-chain in 4.3 s (4.3 ms each, zero transactions), then
redeemed in **one** transaction at 93,315 gas — about 93 gas per payment, falling
further the longer a channel runs. Settling the same run as individual transfers
would cost roughly 21,000,000 gas. Two channels then netted in one 102,322-gas
settlement, and the contract remained solvent at every step.

## Security Status

Neither this specification nor its reference implementation has been externally
audited. Do not deploy it to a network carrying real value on the strength of
this document.

## Copyright

Copyright and related rights waived via CC0.
