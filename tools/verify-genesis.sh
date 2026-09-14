#!/usr/bin/env bash
# Does this genesis file actually produce the chain we think it does?
#
# A genesis file that is subtly wrong does not fail loudly. The node starts,
# syncs nothing, peers with nobody, and reports a healthy-looking block 0 of a
# chain that exists only on that machine. The only reliable check is to build
# the genesis with the real binary and compare the resulting hash to a chain
# that is already running.
#
# This exists because `genesis/testnet.json` does NOT produce Argos:
#
#     committed genesis/testnet.json -> 0xe984f667…c294
#     live Argos testnet             -> 0xcbb0f12e…42c9
#
# The live network was created by ethpandaops/ethereum-genesis-generator via
# deploy/testnet-pos/gen-genesis.sh, which injects the deposit contract and sets
# a different gasLimit, timestamp, nonce and extraData. The committed file is a
# hand-written approximation of it and cannot join the network.
#
#   tools/verify-genesis.sh genesis/argos-testnet.json https://testnet.creditchain.org
#   BIN=target/release/creditchaind tools/verify-genesis.sh <file> <rpc>
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

GENESIS="${1:?usage: verify-genesis.sh <genesis.json> <rpc-url>}"
RPC="${2:?usage: verify-genesis.sh <genesis.json> <rpc-url>}"
BIN="${BIN:-target/debug/creditchaind}"

b()   { printf '\n\033[1m%s\033[0m\n' "$*"; }
ok()  { printf '  \033[32m✓\033[0m %s\n' "$*"; }
bad() { printf '  \033[31m✗\033[0m %s\n' "$*"; }

[ -f "$GENESIS" ] || { bad "no such genesis file: $GENESIS"; exit 2; }
[ -x "$BIN" ] || { bad "no binary at $BIN — build it first, or set BIN="; exit 2; }

b "Verifying $GENESIS against $RPC"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# The binary is the only authority on what a genesis file hashes to. Parsing the
# JSON ourselves would just reimplement the bug we are looking for.
# The binary colourises its logs, so `hash` and `=` are separated by escape
# sequences; strip ANSI before matching or the grep silently finds nothing.
INIT_OUT="$("$BIN" init --chain "$GENESIS" --datadir "$TMP" 2>&1 || true)"
LOCAL_HASH="$(printf '%s' "$INIT_OUT" \
  | sed -E 's/\x1b\[[0-9;]*m//g' \
  | grep -oE '0x[0-9a-f]{64}' | tail -1 || true)"
if [ -z "$LOCAL_HASH" ]; then
  bad "binary did not report a genesis hash; init said:"
  printf '%s\n' "$INIT_OUT" | sed -E 's/\x1b\[[0-9;]*m//g' | tail -12 | sed 's/^/      /'
  exit 1
fi
LIVE_HASH="$(curl -fsS "$RPC" -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"eth_getBlockByNumber","params":["0x0",false]}' \
  | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["hash"])')"

echo "  this file : $LOCAL_HASH"
echo "  live chain: $LIVE_HASH"

if [ "$LOCAL_HASH" = "$LIVE_HASH" ]; then
  b "MATCH"
  ok "a node started with this file will join that network"
  exit 0
fi

b "MISMATCH — a node started with this file joins a DIFFERENT chain"

# Say *why*, so the failure is actionable rather than just alarming.
python3 - "$GENESIS" "$RPC" <<'PY'
import json, sys, urllib.request
path, rpc = sys.argv[1], sys.argv[2]
req = urllib.request.Request(rpc,
    data=json.dumps({"jsonrpc":"2.0","id":1,"method":"eth_getBlockByNumber",
                     "params":["0x0",False]}).encode(),
    headers={"content-type":"application/json"})
live = json.load(urllib.request.urlopen(req, timeout=25))["result"]
f = json.load(open(path))

def num(v):
    if v is None: return None
    if isinstance(v, str) and v.startswith("0x"):
        try: return int(v, 16)
        except ValueError: return v
    return v

print("  differing header fields:")
any_diff = False
for k in ("timestamp","gasLimit","difficulty","baseFeePerGas","nonce",
          "extraData","excessBlobGas","mixHash"):
    l, c = num(live.get(k)), num(f.get(k))
    if l != c:
        any_diff = True
        print(f"    {k:14} live={live.get(k)!s:<24} file={f.get(k)!s}")
if not any_diff:
    print("    none — headers agree")

coded = [a for a, v in (f.get("alloc") or {}).items() if v.get("code")]
print(f"  alloc: {len(f.get('alloc') or {})} accounts, {len(coded)} with code")
if not coded:
    print("    the live chain carries a deposit contract and the EIP system")
    print("    contracts; a file with no coded accounts cannot reproduce its")
    print("    state root even if every header field is corrected.")
PY

cat <<'EOF'

  The authoritative genesis for a PoS CreditChain network is the generator's
  output, not a hand-written file:

    deploy/testnet-pos/gen-genesis.sh  ->  ~/testnet-pos/gen/output/genesis.json

  deploy-node.sh mounts that file directly. Commit it, or point deployments at
  it — do not maintain a second copy by hand.
EOF
exit 1
