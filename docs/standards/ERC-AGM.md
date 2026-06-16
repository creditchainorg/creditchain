---
eip: <to be assigned>
title: Agent Spending Mandate (AGM)
description: A chain-enforced, bounded, revocable spending mandate an owner grants to an autonomous agent.
author: CreditChain Research Team
discussions-to: https://www.creditchain.org
status: Draft
type: Standards Track
category: ERC
created: 2026-06-16
requires: 165
---

## Abstract

This standard defines `IAgentSpendMandate`, an interface for delegating
**bounded, revocable spending authority** from an owner to an autonomous agent.
An owner funds a mandate and grants an `agent` address the right to spend from
it, while the contract enforces — at the chain level, not by trust — a total
budget cap, a per-transaction maximum, a rolling-window rate limit, an optional
recipient allowlist, and an expiry. The owner retains an instant kill switch
that refunds the unspent balance. No party other than a mandate's own owner may
move its funds.

## Motivation

AI agents increasingly need to transact at machine speed — paying per inference,
per dataset, per API call — thousands of times an hour with no human approving
each payment. The account model of existing chains offers only two bad options:
hand the agent a private key (unbounded, catastrophic if the agent misbehaves or
is compromised) or keep a human in the loop (defeating autonomy).

Neither is acceptable for production agent commerce. What is missing is a
*first-class primitive for bounded spending authority*: a way to say "this agent
may spend up to X total, at most Y per payment, no faster than Z per hour, only
to these recipients, until this date — and I can revoke and reclaim at any
time." This standard specifies that primitive so agents, wallets, and explorers
can interoperate around a single, auditable notion of an agent's "spending
permit."

## Specification

The key words "MUST", "MUST NOT", "SHOULD", and "MAY" are to be interpreted as
described in RFC 2119 and RFC 8174.

A compliant contract MUST implement `IAgentSpendMandate` and SHOULD implement
[ERC-165](./eip-165.md) returning `true` for the interface id.

```solidity
interface IAgentSpendMandate {
    struct Mandate {
        address owner; address agent;
        uint256 budget; uint256 spent; uint256 balance;
        uint256 perTxMax; uint256 windowLimit;
        uint64 windowSeconds; uint64 windowStart; uint256 windowSpent;
        uint64 expiry; bool allowlistEnabled; bool revoked;
    }

    event MandateCreated(uint256 indexed mandateId, address indexed owner, address indexed agent,
        uint256 budget, uint256 perTxMax, uint256 windowLimit, uint64 windowSeconds, uint64 expiry, bool allowlistEnabled);
    event MandateFunded(uint256 indexed mandateId, address indexed from, uint256 amount, uint256 newBalance);
    event RecipientAllowed(uint256 indexed mandateId, address indexed recipient, bool allowed);
    event AgentPayment(uint256 indexed mandateId, address indexed agent, address indexed recipient,
        uint256 amount, bytes32 taskRef, uint256 totalSpent);
    event MandateRevoked(uint256 indexed mandateId, address indexed owner, uint256 refunded);
    event Withdrawn(uint256 indexed mandateId, address indexed owner, uint256 amount);

    function createMandate(address agent, uint256 budget, uint256 perTxMax, uint256 windowLimit,
        uint64 windowSeconds, uint64 expiry, bool allowlistEnabled) external payable returns (uint256 mandateId);
    function fundMandate(uint256 mandateId) external payable;
    function allowRecipient(uint256 mandateId, address recipient, bool allowed) external;
    function spend(uint256 mandateId, address recipient, uint256 amount, bytes32 taskRef) external;
    function revoke(uint256 mandateId) external;
    function withdraw(uint256 mandateId, uint256 amount) external;
    function mandate(uint256 mandateId) external view returns (Mandate memory);
    function spendableNow(uint256 mandateId) external view returns (uint256);
    function mandateCount() external view returns (uint256);
    function allowlisted(uint256 mandateId, address recipient) external view returns (bool);
}
```

### Roles

- **owner** — creates, funds, configures, withdraws from, and revokes a mandate.
- **agent** — the *only* address permitted to `spend` from a mandate.

### `createMandate`

Creates a mandate owned by `msg.sender`, delegated to `agent`, with the given
rails. Any value sent becomes the initial balance. `agent` MUST NOT be the zero
address. MUST emit `MandateCreated`, and `MandateFunded` if `msg.value > 0`.
Returns a `mandateId` unique within the contract.

A rail value of `0` disables that rail: `budget == 0` means no budget cap,
`perTxMax == 0` no per-transaction cap, `windowLimit == 0` no rate limit,
`expiry == 0` never expires.

### `spend`

Transfers `amount` of the native asset from the mandate to `recipient`. It:

1. MUST revert `NotAgent` unless `msg.sender` is the mandate's agent.
2. MUST revert `MandateInactive` if revoked or past `expiry`.
3. MUST revert `RecipientNotAllowed` if `allowlistEnabled` and `recipient` is
   not allowlisted.
4. MUST revert `PerTxExceeded` if `perTxMax != 0 && amount > perTxMax`.
5. MUST revert `BudgetExceeded` if `budget != 0 && spent + amount > budget`.
6. MUST revert `InsufficientBalance` if `amount > balance`.
7. MUST enforce the rolling window: if `windowLimit != 0`, reset the window when
   `block.timestamp >= windowStart + windowSeconds`, then revert `WindowExceeded`
   if `windowSpent + amount > windowLimit`.

On success it MUST update `spent`, `balance`, and window accounting **before**
transferring (checks-effects-interactions), MUST be reentrancy-safe, and MUST
emit `AgentPayment`. `taskRef` is an opaque caller-supplied reference (e.g. an
invoice or task hash) carried in the event for off-chain correlation.

On any revert, no value moves.

### `revoke` / `withdraw`

`revoke` MUST be owner-only, deactivate the mandate so the agent can never spend
again, refund the unspent balance to the owner, and emit `MandateRevoked`.
`withdraw` MUST be owner-only and let the owner reclaim part of the unspent
balance without revoking, emitting `Withdrawn`.

### `spendableNow`

MUST return the largest amount the agent could spend in a single call right now,
i.e. the minimum across the live balance, remaining budget, remaining window
allowance, and per-transaction cap; `0` if revoked or expired.

### No global escape hatch

A compliant implementation MUST NOT expose any function by which an address
other than a mandate's `owner` can move that mandate's funds — no admin, pause,
or upgrade path that can seize balances.

## Rationale

- **Native value, not just tokens.** Agent micropayments are dominated by the
  gas/settlement asset; the core interface targets the native asset. A token
  profile MAY be layered on top.
- **`spend` carries `taskRef`.** Agent payments are *for something*; an opaque
  reference makes every payment auditable against the off-chain task without
  prescribing a schema.
- **Rails are per-mandate, not per-agent.** One agent can hold many mandates
  with different limits from different owners; one owner can run many agents.
- **`spendableNow` is mandatory.** Agents and wallets need one honest "how much
  can I spend right now" number that already accounts for every rail.

## Backwards Compatibility

This is a new interface; there is nothing to break. It is EVM-native and works
on any chain with a standard EVM. It composes with existing tooling: the events
are ordinary logs any indexer can read, and the methods are ordinary calls any
wallet can make.

## Reference Implementation

`AgentSpendVault.sol` (CreditChain `contracts/agent-finance/`) implements this
interface. It is covered by 13 example tests, 4 invariant properties proven over
~12,800 randomized sequences each (solvency, budget, conservation), and a
conformance test that exercises the whole lifecycle *through* `IAgentSpendMandate`.
A live, source-verified deployment runs on the CreditChain public testnet.

## Security Considerations

- **Reentrancy.** `spend`, `revoke`, and `withdraw` transfer value and MUST
  follow checks-effects-interactions and be reentrancy-guarded; a malicious
  recipient MUST NOT be able to re-enter and exceed any rail.
- **Agent key compromise is bounded, not eliminated.** A compromised agent can
  still spend up to the rails the owner set. Owners SHOULD scope budget,
  per-tx, window, allowlist, and expiry to the smallest workable values and
  revoke on suspicion. The standard's value is that the blast radius is
  *bounded and revocable*, not zero.
- **Allowlist vs. open spend.** An open (allowlist-disabled) mandate trusts the
  agent's recipient choice within the value rails; an allowlisted mandate
  additionally constrains *where* funds can go. High-value mandates SHOULD use
  the allowlist.
- **Rolling window is approximate by design.** The window resets lazily on the
  first spend after it elapses, which is gas-efficient and sufficient for rate
  limiting; it is not a precise sliding window and SHOULD NOT be relied on as one.
- **Griefing via funding.** Implementations that let anyone `fundMandate`
  SHOULD note that funds added to a revoked mandate remain owner-recoverable via
  `withdraw`/`revoke` and are never spendable by the agent.

## Copyright

Copyright and related rights waived via [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
