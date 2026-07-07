#!/usr/bin/env sh
set -eu

failed=0

for lib in crates/*/src/lib.rs; do
    case "$lib" in
    crates/skrifheim-storage-host/src/lib.rs)
        continue
        ;;
    esac
    if ! grep -q '#!\[no_std\]' "$lib"; then
        echo "$lib must use #![no_std]" >&2
        failed=1
    fi
    if ! grep -q '#!\[forbid(unsafe_code)\]' "$lib"; then
        echo "$lib must use #![forbid(unsafe_code)]" >&2
        failed=1
    fi
done

if rg -n '\bstd::|extern crate std|use std' crates --glob '*.rs' --glob '!*/src/main.rs' --glob '!crates/skrifheim-storage-host/**'; then
    echo "core crates must not import std; use core/alloc or move host-only code into an explicit host-boundary crate" >&2
    failed=1
fi

if rg -n 'zeroize' Cargo.toml Cargo.lock crates; then
    echo "zeroize is not admitted; use sanitization only after dependency review" >&2
    failed=1
fi

if rg -n '(^|[^[:alnum:]_-])base64[[:space:]]*=|base64::' Cargo.toml Cargo.lock crates; then
    echo "the base64 crate is not admitted; use base64-ng only after dependency review" >&2
    failed=1
fi

exit "$failed"
