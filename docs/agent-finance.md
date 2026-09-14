Copyright (c) CreditChain Research Team. All rights reserved.

# CreditChain Agent Finance

CreditChain Agent Finance is the first protocol layer that makes CreditChain
more than another EVM-compatible chain.

It introduces event-first financial primitives for the agentic economy:

- AgentID
- SpendPermit
- PaymentIntent
- TaskReceipt
- SettlementReceipt
- CreditObject

The first implementation lives in
[creditchainorg/contracts](https://github.com/creditchainorg/contracts):

```text
agent-finance/src/CreditAgentFinance.sol
```

## Design Principle

Keep EVM compatibility and add financial differentiation above it.

The MVP contract does not modify consensus, custody funds, or claim production
compliance. It emits canonical events that CreditChain Browser can index and
explain.

## First Demo

```text
AgentRegistered
  -> SpendPermitCreated
  -> PaymentIntentCreated
  -> TaskReceiptRecorded
  -> PaymentSettled
```

CreditChain Browser should answer:

```text
Was this agent payment authorized, proven, and settled?
```

## Future Native Extensions

Once the event-first model is proven, CreditChain can add:

- native precompile helpers;
- typed transaction wrappers;
- EIP-712 permit standards;
- credit-state indexer commitments;
- Ethereum anchoring of Credit Object roots;
- OpeniBank institutional workflows.
