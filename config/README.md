# CreditChain Environment Config

Copyright (c) CreditChain Research Team.

This directory contains operator-facing environment manifests for CreditChain.
The manifests are intentionally small: they define stable names, chain IDs,
genesis files, default endpoints, and reset policy without baking deployment
secrets into the repository.

Use these files as release inputs for `creditchaind`, packaging, infra modules,
and smoke tests. The canonical source repository is
`https://github.com/openibank/creditchain`.
