# creditchain-faucet

Lightweight, hardened public faucet for CreditChain test networks. Drips a
configurable amount of the network native gas token to any requested EVM
address under two enforced limits. CreditChain defaults to `CCC`; forks can
change the token display metadata with env vars.

- **Per-IP**: a sliding-window rate cap (default: 5 drips per hour).
- **Per-recipient**: a cooldown timer (default: 86 400 s = 1 drip per day per address).

## Build

```bash
cargo build --release
```

## Run

```bash
export FAUCET_RPC_URL=http://127.0.0.1:8545
export FAUCET_PUBLIC_RPC_URL=https://testnet.creditchain.org
export FAUCET_CHAIN_ID=2026042404
export FAUCET_PRIVATE_KEY_FILE=/run/secrets/faucet.key
export FAUCET_NATIVE_TOKEN_SYMBOL=CCC
./target/release/creditchain-faucet
```

The faucet validates the configured `FAUCET_CHAIN_ID` against the RPC's
`eth_chainId` at startup and refuses to launch on mismatch. Misconfigured
deploys fail loudly.

## Configuration

| Env var | Required | Default | Notes |
|---|---|---|---|
| `FAUCET_RPC_URL` | ✓ | — | JSON-RPC HTTP URL the faucet signs through (may be internal; never published) |
| `FAUCET_PUBLIC_RPC_URL` |   | — | RPC URL that `/info` shows users; unset means `/info` omits `rpc_url` |
| `FAUCET_CHAIN_ID` | ✓ | — | Decimal chain id; must match RPC |
| `FAUCET_PRIVATE_KEY_FILE` | ✓ | — | Path to a file containing the 32-byte hex private key (optional `0x` prefix) |
| `FAUCET_NATIVE_TOKEN_NAME` |   | `CreditChain Token` | Native gas token display name |
| `FAUCET_NATIVE_TOKEN_SYMBOL` |   | `CCC` | Native gas token symbol; use this for institutional forks |
| `FAUCET_NATIVE_TOKEN_DECIMALS` |   | `18` | Native gas token decimals |
| `FAUCET_DRIP_AMOUNT` |   | `1.0` | Decimal native token per drip |
| `FAUCET_DRIP_AMOUNT_CCC` |   | — | Backwards-compatible alias for older env files |
| `FAUCET_PER_IP_LIMIT` |   | `5` | Drips per IP per rolling hour |
| `FAUCET_PER_ADDRESS_COOLDOWN` |   | `86400` | Seconds between drips to the same recipient |
| `FAUCET_BIND` |   | `0.0.0.0:8080` | Listen address |
| `FAUCET_NETWORK_NAME` |   | `creditchain` | Display label used in `/info` + logs |
| `RUST_LOG` |   | `info,tower_http=info,creditchain_faucet=debug` | Standard tracing filter |

## HTTP API

### `GET /health`

Liveness + chain id + on-chain faucet balance.

```json
{
  "ok": true,
  "chain_id": 2026042404,
  "native_currency": { "name": "CreditChain Token", "symbol": "CCC", "decimals": 18 },
  "faucet_address": "0x…",
  "balance": "10000.0"
}
```

### `GET /info`

Human-readable info card. Useful for the public Browser to render
"Faucet: drips 1.0 CCC per recipient per day". It includes `rpc_url` only when
`FAUCET_PUBLIC_RPC_URL` is set; the URL the faucet signs through is never shown.

### `POST /drip`

```bash
curl -fsS -H 'content-type: application/json' \
  --data '{"address":"0x000…1234"}' \
  https://faucet.creditchain.org/testnet/drip
```

Success → `200`:
```json
{
  "network": "creditchain-testnet",
  "chain_id": 2026042404,
  "tx": "0xabc…",
  "to":  "0x000…1234",
  "amount":    "1.0 CCC",
  "amount_base_units": "1000000000000000000"
}
```

Rate-limited → `429` with `Retry-After`-style hint:
```json
{ "code": "faucet.ip_rate_limited", "message": "...", "retry_after_seconds": 3600 }
```

Bad input → `400`:
```json
{ "code": "faucet.bad_request", "message": "address is not a valid EVM address" }
```

## Security model

- Private key stays in memory; we never log it.
- Per-IP + per-recipient rate-limits are enforced atomically (a denied request
  does not consume the IP quota — see `rejected_drip_does_not_commit_ip_hit`).
- `RequestBodyLimit` = 1 MiB; `Timeout` = 30 s.
- Returns `chain_id`, `to`, and `amount_base_units` in every success response
  so callers can independently verify what was sent.
- Returns `native_currency` from `/health` and `/info`, so enterprise forks can
  present their own symbol without patching wallet or Browser clients.

## Docker

The bundled multi-stage `Dockerfile` produces a ~20 MB distroless image:

```bash
docker build -t creditchain-faucet:local .
docker run --rm \
  -e FAUCET_RPC_URL=http://node:8545 \
  -e FAUCET_PUBLIC_RPC_URL=https://testnet.creditchain.org \
  -e FAUCET_CHAIN_ID=2026042404 \
  -e FAUCET_PRIVATE_KEY_FILE=/secrets/faucet.key \
  -e FAUCET_NATIVE_TOKEN_SYMBOL=CCC \
  -v $PWD/secrets:/secrets:ro \
  -p 8080:8080 \
  creditchain-faucet:local
```

Run it next to a node the faucet can reach, with the key file mounted read-only and
never baked into the image.
