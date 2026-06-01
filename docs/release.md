# CreditChain Releases

CreditChain releases produce signed `creditchaind` artifacts, Docker images,
environment manifests, and operator notes.

Current release-prep notes:

- [v2.1.0 Enterprise Readiness](./releases/v2.1.0-enterprise-readiness.md)

## Release PR

- Create a branch such as `release/vx.y.z`.
- Ensure tests, lints, and smoke checks pass for the release commit.
- Update workspace versions and release manifests.
- Confirm `config/environments/` and `genesis/` match the intended network.
- Commit with `release: vx.y.z`.
- Require review from maintainers responsible for node, release, and operator
  readiness.

## Publish

- Tag the merge commit as `vx.y.z`.
- Push the tag to `openibank/creditchain`.
- Build and publish `creditchaind` artifacts.
- Publish Docker images under `ghcr.io/openibank/creditchain`.
- Publish operator notes on `https://docs.creditchain.org`.

Devnet may reset by release. Testnet must not reset casually. Mainnet releases
require an explicit operator notice and rollback plan.
