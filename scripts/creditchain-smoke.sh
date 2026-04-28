#!/usr/bin/env bash
set -euo pipefail

BIN="${CREDITCHAIND_BIN:-creditchaind}"
RPC_URL="${CREDITCHAIN_RPC_URL:-http://127.0.0.1:8545}"

echo "creditchaind version:"
"$BIN" --version

echo "web3_clientVersion:"
curl -fsS \
  -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"web3_clientVersion","params":[]}' \
  "$RPC_URL"
echo

echo "eth_chainId:"
curl -fsS \
  -H 'content-type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}' \
  "$RPC_URL"
echo
