# Launch announcement — CreditChain Ecosystem v2.2.0 "Agent Finance"

Channels: X/Twitter, Farcaster, LinkedIn, the project blog. Tone: confident,
concrete, every claim links to something runnable. Disclosures visible.

---

## X / Farcaster thread

**1/**
CreditChain Ecosystem **v2.2.0 — "Agent Finance"** is live on public testnet.

The pitch in one line: an AI agent can now spend **autonomously**, while the
**blockchain enforces every limit you set**. Not a slide — a chain you can use
right now. 🧵

**2/**
The primitive: **AgentSpendVault** (now an open standard, **ERC-AGM**).

You fund a vault and grant an agent a bounded, revocable mandate — budget cap,
per-tx max, rate limit, recipient allowlist, expiry. The agent transacts; the
chain holds the line; you keep an instant kill switch.

**3/**
It's not a promise — it's tested and proven:
• 19 Foundry tests incl. **4 invariants over ~12,800 randomized sequences**
  (the vault is always solvent; never over-budget)
• **source-verified** on the public explorer
• a live demo of an agent paying its own bills, every rail enforced on-chain

**4/**
The wallet ships the message: iWallet's new **Agent Mandate Console** —
grant, monitor, and revoke mandates from your phone. *Grant a mandate, not
your keys.*

Plus unlock-once security: Face ID once, sign all session, no per-tx prompts.

**5/**
For builders: **self-serve contract verification** is live —
`POST /v1/contracts/:address/verify` matches your bytecode against the chain
and publishes your source. Etherscan-style, on CreditChain.

**6/**
Try it now — free, testnet, two minutes:
• RPC https://testnet.creditchain.org
• Faucet https://faucet.creditchain.org
• Explorer https://scan.creditchain.org
• Demo: `RPC_URL=https://testnet.creditchain.org ./agent_commerce_demo.sh`
Releases: github.com/openibank/creditchain/releases/tag/v2.2.0-agent-finance

**7/**
Honest status: this is the **public testnet**. CCC has no monetary value.
Mainnet is **not** launched — it ships only after external audit, a stable PoS
testnet, and a hardware key ceremony. We market what the chain *does*, never a
price. Not investment advice.

**8/**
The AI economy needs rails where humans set the rules and the chain enforces
them. We just shipped the first ones. Come build. 🟢

---

## Single post (LinkedIn / blog lede)

**CreditChain Ecosystem v2.2.0 "Agent Finance" is live on public testnet.**

AI agents can now spend autonomously within bounded, revocable mandates the
chain itself enforces — budget caps, rate limits, allowlists, instant revoke.
It ships as an open standard (ERC-AGM) with an invariant-tested reference
implementation, a live agent-commerce demo, self-serve contract verification,
and a wallet that lets you grant/monitor/revoke mandates from your phone.

Try it: https://scan.creditchain.org · https://testnet.creditchain.org
Releases: https://github.com/openibank/creditchain/releases/tag/v2.2.0-agent-finance

(Public testnet. CCC has no monetary value. Mainnet not launched. Not
investment advice.)

## Asset checklist
- 20s screen capture of `agent_commerce_demo.sh` (green ✓ pays, red ✗ rejects).
- Screenshot: iWallet → Settings → Agent Mandates (grant + revoke).
- Screenshot: a verified AgentSpendVault on https://scan.creditchain.org.
- Pin the thread; cross-post to the CreditChain blog with the release notes.
