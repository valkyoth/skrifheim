#!/usr/bin/env sh
set -eu

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

# Exercise the real gate helper with isolated fixtures, never mutate source.
awk '
    /^check_no_sensitive_derive\(\) \{/ { copying = 1 }
    copying { print }
    copying && /^\}/ { exit }
' scripts/validate-security-policy.sh > "$tmpdir/helper.sh"
. "$tmpdir/helper.sh"

for type_name in KeyScope KeyLifecycleState KeyErasureReason KeyLifecycleEventSequence KeyErasureMetadata KeyRotationPreflight KeyMetadata WalFileFrame WorldDiff WorldConflict PromotionPreflight RollbackPreflight '$name'; do
    pattern="$type_name"
    if [ "$type_name" = '$name' ]; then
        pattern='[$]name'
    fi
    for kind in struct enum; do
        printf '#[derive(Clone)]\npub %s %s {}\n' "$kind" "$type_name" > "$tmpdir/fixture.rs"
        (check_no_sensitive_derive "$tmpdir/fixture.rs" "$pattern" Debug)
        for derive in '#[derive(Clone, Debug)]' '#[derive(core::fmt::Debug)]' '#[derive(
    Clone,
    Debug,
)]'; do
            printf '%s\npub %s %s {}\n' "$derive" "$kind" "$type_name" > "$tmpdir/fixture.rs"
            if (check_no_sensitive_derive "$tmpdir/fixture.rs" "$pattern" Debug) > "$tmpdir/error" 2>&1; then
                echo "security derive gate accepted Debug on $type_name" >&2
                exit 1
            fi
            grep -q 'must not derive Debug' "$tmpdir/error"
        done
    done
done
