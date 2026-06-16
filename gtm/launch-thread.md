# Launch thread — "the AI with a debit card the blockchain enforces"

Primary channel: X/Twitter thread. Tone: confident, concrete, honest. Every
claim is backed by a link anyone can check. Disclosures kept visible (testnet,
CCC has no monetary value, not investment advice).

---

**1/**
We gave an AI agent a debit card.

The spending limits aren't enforced by an app, a bank, or a polite prompt.
They're enforced by the blockchain itself — and it's spending **right now** on
a public testnet.

Watch. 🧵

**2/**
The problem nobody's solved: an AI agent has to pay at machine speed —
per API call, per inference, per dataset — thousands of times an hour.

You can't hand it your private key (one bad loop drains you).
You can't approve every payment by hand (that's the whole point of an agent).

Every chain forces that bad trade.

**3/**
CreditChain's answer is one primitive: **AgentSpendVault.**

You fund a vault and grant your agent a *bounded, revocable mandate*:
"spend up to 5 CCC, ≤2 per payment, only to these addresses, for 30 days."

Then you walk away. The agent transacts on its own. The chain holds the line.

**4/**
Every rail is enforced by the contract — not by trust:

• total budget cap
• per-transaction max
• rolling-window rate limit
• recipient allowlist
• expiry
• instant owner revoke → unspent refunds
• no admin escape hatch — nobody can touch your funds

**5/**
Live on the public testnet: an agent holds a 5 CCC mandate and pays a metered
API per task — autonomously, no human in the loop per payment.

Real value, real blocks. See the vault:
https://scan.creditchain.org/?net=testnet

**6/**
Now watch the chain say **no.**

When the agent tries to overspend, the contract rejects it on-chain, by name:
`PerTxExceeded` · `BudgetExceeded` · `RecipientNotAllowed`

Not a warning. A revert. The money never moves.

**7/**
The kill switch: the owner revokes the mandate, the unspent CCC refunds
instantly, and the agent can never spend again → `MandateInactive`.

You were never not in control. You just stopped having to click "confirm."

**8/**
Best part: it's fully EVM.

Your MetaMask, your Foundry, your Solidity — all work here on day one. We
didn't reinvent the wheel. We added the one primitive that makes AI commerce
safe, and built the wallet + explorer + agent stack around it.

**9/**
Try it yourself — free, testnet, two minutes:

• RPC: https://testnet.creditchain.org
• Faucet: https://faucet.creditchain.org
• Explorer: https://scan.creditchain.org
• Run the demo (one command):
  `RPC_URL=https://testnet.creditchain.org ./agent_commerce_demo.sh`

**10/**
This is the foundation of OpeniBank: AI agents that hold mandates, settle in
CCC, and deliver financial services under rules **humans set and the chain
enforces.**

The AI economy needs rails. We're shipping them.

(Public testnet. CCC has no monetary value. Nothing here is investment advice.)

---

## Single-post version (for LinkedIn / Farcaster / a pinned post)

We gave an AI agent a debit card — with spending limits enforced by the
blockchain, not an app.

Fund a vault, grant a bounded mandate (budget, per-tx cap, allowlist, expiry),
and the agent pays autonomously while the chain rejects anything out of bounds
and the owner keeps an instant kill switch.

It's live on CreditChain's public testnet right now. Fully EVM. Try it:
https://scan.creditchain.org · https://testnet.creditchain.org

(Testnet; CCC has no monetary value; not investment advice.)

## Visual / media suggestions
- A 20-second screen capture of `agent_commerce_demo.sh` running: the green
  ✓ payments, then the red ✗ chain-rejected rails. The terminal *is* the ad.
- Or a split card: left "grant a mandate" (rules), right "the chain enforces"
  (the four rejection errors). Caption: "grant a mandate, not your keys."
