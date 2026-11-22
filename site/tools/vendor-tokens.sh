#!/usr/bin/env bash
# Vendors the Mzizi token layer into site/tokens/tokens.css, never by hand.
#
#   site/tools/vendor-tokens.sh            fetch the pinned file and write it
#   site/tools/vendor-tokens.sh --check    fail if the committed file differs from the pinned one
#   site/tools/vendor-tokens.sh --update   re-pin to the head of mzizi-dev/mzizi-registry main, then write
#
# The source is styles/pattern/tokens.css of mzizi-dev/mzizi-registry, generated there from the
# palette and the floor (`pnpm pattern:generate`). The commit is pinned in site/tokens/PIN so a
# build never changes under us; CI runs --check. A value this site needs that the file lacks is
# proposed upstream, not typed into page CSS (see site/tokens/pending.tokens.css).
set -euo pipefail
cd "$(dirname "$0")/.."
REPO=mzizi-dev/mzizi-registry
FILE=styles/pattern/tokens.css
OUT=tokens/tokens.css

if [ "${1:-}" = "--update" ]; then
  sha=$(git ls-remote "https://github.com/$REPO.git" refs/heads/main | cut -f1)
  [ -n "$sha" ] || { echo "cannot read the head of $REPO main" >&2; exit 1; }
  echo "$sha" > tokens/PIN
fi
sha=$(tr -d '[:space:]' < tokens/PIN)
echo "$sha" | grep -Eq '^[0-9a-f]{40}$' || { echo "tokens/PIN is not a 40-hex commit" >&2; exit 1; }
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
curl -fsSL --retry 3 "https://raw.githubusercontent.com/$REPO/$sha/$FILE" -o "$tmp"
if [ "${1:-}" = "--check" ]; then
  if cmp -s "$tmp" "$OUT"; then
    echo "site/tokens/tokens.css matches $REPO@${sha:0:12}"
  else
    echo "site/tokens/tokens.css has drifted from $REPO@${sha:0:12}: run site/tools/vendor-tokens.sh" >&2
    exit 1
  fi
else
  cp "$tmp" "$OUT"
  echo "wrote site/tokens/tokens.css from $REPO@${sha:0:12}"
fi
