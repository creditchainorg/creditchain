## CreditChain Monitoring Assets

This directory contains example Prometheus, Grafana, Loki, and Docker Compose
assets for running and observing `creditchaind`.

Run the local compose stack from this directory:

```sh
docker compose up
```

The compose file runs `creditchaind` with `genesis/local-single.json`, exposes
JSON-RPC on `127.0.0.1:8545`, and exposes metrics on port `9001`.

Operator documentation belongs in `docs/operators/` and public documentation
should point to `https://docs.creditchain.org`.
