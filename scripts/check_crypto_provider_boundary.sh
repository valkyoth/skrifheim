#!/usr/bin/env sh
set -eu

# These are source dependency guards; full graph evidence is also reviewed.
if rg -n '^(sha3|shake|chacha20poly1305|poly1305)[[:space:]]*=|\b(sha3|shake|chacha20poly1305|poly1305)::' \
    crates --glob '*.toml' --glob '*.rs' --glob '!crates/skrifheim-crypto-rustcrypto/**'; then
    echo "cryptographic implementations must remain inside the admitted provider" >&2
    exit 1
fi
if rg -n '^getrandom[[:space:]]*=|getrandom::' crates \
    --glob '*.toml' --glob '*.rs' --glob '!crates/skrifheim-entropy-host/**'; then
    echo "OS entropy must remain inside its admitted host boundary" >&2
    exit 1
fi

tree="$(cargo tree --locked -p skrifheim --no-default-features --edges normal --prefix none)"
if printf '%s\n' "$tree" | grep -E '^(skrifheim-crypto-rustcrypto|skrifheim-entropy-host|sha3|shake|getrandom|zeroize) '; then
    echo "optional providers leaked into the default core graph" >&2
    exit 1
fi

for crate in sha3 shake chacha20poly1305 poly1305; do
    if ! grep -E "^$crate = .*default-features = false, features = \[\"zeroize\"\]" \
        crates/skrifheim-crypto-rustcrypto/Cargo.toml >/dev/null; then
        echo "required upstream private-state cleanup missing: $crate" >&2
        exit 1
    fi
done
case "${RUSTFLAGS:-} ${CARGO_ENCODED_RUSTFLAGS:-}" in
    *getrandom_backend*)
        echo "alternate entropy backends need separate admission" >&2
        exit 1
        ;;
esac
