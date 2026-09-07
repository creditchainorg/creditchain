#!/usr/bin/env bash
# How far is CreditChain behind upstream Reth, and what is in the gap?
#
# Written because the fork drifted four months without anyone noticing, and the
# first attempt to measure the drift got it wrong by 15x. Two traps, both of
# which this script avoids:
#
#   1. `git merge-base HEAD upstream/<some-old-branch>` does NOT give the fork
#      point. It gives the common ancestor with that branch, which sweeps
#      upstream's own progress into what looks like our delta. Use
#      `--not --remotes=upstream` to get commits that exist nowhere upstream.
#
#   2. Upstream release tags are NOT on `main`. They are cut on release
#      branches. Syncing to `main` pulls unreleased work; a chain heading for
#      mainnet should track the newest release tag instead.
#
# Read-only: fetches, measures, and reports. It never merges.
#
#   tools/upstream-report.sh            # against the newest upstream release
#   TARGET=v2.6.0 tools/upstream-report.sh
#   SKIP_FETCH=1 tools/upstream-report.sh
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

REMOTE="${REMOTE:-upstream}"
b()   { printf '\n\033[1m%s\033[0m\n' "$*"; }
dim() { printf '\033[2m%s\033[0m\n' "$*"; }

git remote get-url "$REMOTE" >/dev/null 2>&1 || {
  echo "no '$REMOTE' remote. Add it:" >&2
  echo "  git remote add upstream https://github.com/paradigmxyz/reth.git" >&2
  exit 1; }

if [ -z "${SKIP_FETCH:-}" ]; then
  dim "fetching $REMOTE (set SKIP_FETCH=1 to skip)…"
  git fetch "$REMOTE" --tags --prune --quiet
fi

# Newest upstream release by tag date. Release tags are vN.N.N; ignore our own.
if [ -z "${TARGET:-}" ]; then
  # Newest tag that is NOT an ancestor of main — i.e. cut on a release branch.
  # `|| true` throughout: `head` closing the pipe early would otherwise SIGPIPE
  # the producer and, under `set -euo pipefail`, kill the script silently.
  ALL_TAGS=$(git for-each-ref --sort=-creatordate --format='%(refname:short)' 'refs/tags/v[0-9]*' || true)
  TARGET=$(printf '%s\n' "$ALL_TAGS" | while read -r t; do
     [ -n "$t" ] || continue
     git merge-base --is-ancestor "$t" "$REMOTE/main" 2>/dev/null || printf '%s\n' "$t"
   done | head -1 || true)
fi
[ -n "$TARGET" ] || { echo "could not determine an upstream release tag" >&2; exit 1; }

# Commits that exist nowhere upstream — the true CreditChain delta.
OURS=$(git rev-list --count HEAD --not --remotes="$REMOTE")
# Where our history last touched upstream.
BASE=$(git merge-base HEAD "$REMOTE/main")

b "CreditChain vs upstream Reth"
dim "  our head          $(git log -1 --format='%h %s' HEAD | cut -c1-70)"
dim "  upstream base     $(git log -1 --format='%h %ad' --date=short "$BASE")"
dim "  newest release    $TARGET  ($(git log -1 --format='%ad' --date=short "$TARGET"))"
echo "  CreditChain-only commits : $OURS"
echo "  behind $TARGET$(printf '%*s' $((14-${#TARGET})) '') : $(git rev-list --count --no-merges "$BASE".."$TARGET")"
echo "  behind $REMOTE/main       : $(git rev-list --count --no-merges "$BASE".."$REMOTE/main")  (includes unreleased work)"

b "Gap by commit type"
git log --no-merges --format='%s' "$BASE".."$TARGET" \
  | sed -E 's/^([a-z]+)(\(.*\))?!?:.*/\1/' | grep -E '^[a-z]+$' \
  | sort | uniq -c | sort -rn | sed 's/^/  /' | head -8 || true

b "Breaking changes (conventional '!' marker)"
BREAK=$(git log --no-merges --format='%s' "$BASE".."$TARGET" | grep -E '^[a-z]+(\(.*\))?!:' || true)
[ -n "$BREAK" ] && echo "$BREAK" | sed 's/^/  /' || echo "  none — additive sync"

b "Consensus-critical fixes (evm / consensus / engine / trie / revm)"
git log --no-merges --format='  %s' "$BASE".."$TARGET" \
  -- crates/evm crates/consensus crates/engine crates/trie crates/revm crates/primitives 2>/dev/null \
  | grep -E '^\s+(fix|revert)' | head -20 || true
echo "  …$(git log --no-merges --format='%s' "$BASE".."$TARGET" -- crates/evm crates/consensus crates/engine crates/trie crates/revm crates/primitives 2>/dev/null | grep -cE '^(fix|revert)') total"

b "Networking fixes (this fork runs a custom chain id behind NAT — read these)"
git log --no-merges --format='  %s' "$BASE".."$TARGET" -- crates/net 2>/dev/null \
  | grep -E '^\s+fix' | head -12 || true

b "Conflict forecast"
CONFLICTS=$(git merge-tree --write-tree --name-only HEAD "$TARGET" 2>/dev/null \
  | awk 'NR>1{if($0=="")exit; print}' | wc -l | tr -d ' ')
if [ "$CONFLICTS" = "0" ]; then
  echo "  clean merge"
else
  echo "  $CONFLICTS conflicted file(s):"
  git merge-tree --write-tree --name-only HEAD "$TARGET" 2>/dev/null \
    | awk 'NR>1{if($0=="")exit; print}' | sed 's/^/    /' | head -25 || true
fi

b "To sync"
dim "  git checkout -b sync/upstream-$TARGET"
dim "  git merge $TARGET"
dim "  # policy: upstream's logic, CreditChain's identity."
dim "  # re-apply branding afterwards, then verify:"
dim "  #   grep -rl 'target: \"reth::cli\"' crates/ bin/ --include='*.rs'"
dim "  #   grep -rl 'github.com/paradigmxyz/reth' crates/ bin/ --include='*.rs'"
dim "  cargo +stable check --workspace --all-targets   # MSRV is ahead of many default toolchains"
dim "  make update-book-cli                            # the 'book' CI job enforces this"
echo
dim "  See docs/UPSTREAM-SYNC-2026-09-07.md for the last sync's decisions."
