#!/usr/bin/env sh
set -eu

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cat > "$tmpdir/valid.md" <<'EOF'
# Version Plan

## v0.1.0 - First
## v0.1.1 - Patch
## v0.2.0 - Minor
## v1.0.0 - Stable
EOF

sh scripts/check_version_plan_semver.sh "$tmpdir/valid.md"

expect_reject() {
    name="$1"
    body="$2"
    path="$tmpdir/$name.md"
    printf '%s\n' "$body" > "$path"

    if sh scripts/check_version_plan_semver.sh "$path" >/dev/null 2>&1; then
        echo "expected SemVer validator to reject $name fixture" >&2
        exit 1
    fi
}

expect_reject "four-components" '# Version Plan

## v0.18.7.1 - Invalid'

expect_reject "leading-zero-major" '# Version Plan

## v01.0.0 - Invalid'

expect_reject "leading-zero-minor" '# Version Plan

## v0.01.0 - Invalid'

expect_reject "leading-zero-patch" '# Version Plan

## v0.1.01 - Invalid'

expect_reject "not-increasing" '# Version Plan

## v0.2.0 - Later
## v0.1.0 - Earlier'
