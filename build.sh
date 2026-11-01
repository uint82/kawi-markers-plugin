#!/bin/sh
set -eu

dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
kawi_bin=${KAWI_BIN:-kawi}

if command -v "$kawi_bin" >/dev/null 2>&1; then
    exec "$kawi_bin" plugin build "$dir"
fi

if [ -n "${KAWI_MANIFEST:-}" ]; then
    exec cargo run --quiet --manifest-path "$KAWI_MANIFEST" -p kawi -- plugin build "$dir"
fi

echo "kawi not found; set KAWI_BIN or KAWI_MANIFEST" >&2
exit 1
