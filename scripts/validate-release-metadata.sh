#!/usr/bin/env sh
set -eu

if [ -f PENTEST.md ]; then
    echo "PENTEST.md is temporary pentest input; resolve findings and remove it before release validation" >&2
    exit 1
fi

required="
README.md
LICENSE
SECURITY.md
CHANGELOG.md
docs/IMPLEMENTATION_PLAN.md
docs/VERSION_PLAN.md
docs/security-controls.md
docs/threat-model.md
release-notes/RELEASE_NOTES_0.1.0.md
release-notes/RELEASE_NOTES_0.2.0.md
release-notes/RELEASE_NOTES_0.3.0.md
release-notes/RELEASE_NOTES_0.4.0.md
release-notes/RELEASE_NOTES_0.5.0.md
release-notes/RELEASE_NOTES_0.6.0.md
release-notes/RELEASE_NOTES_0.7.0.md
release-notes/RELEASE_NOTES_0.8.0.md
security/pentest/v0.5.0.md
security/pentest/v0.6.0.md
"

for path in $required; do
    if [ ! -f "$path" ]; then
        echo "missing required release metadata: $path" >&2
        exit 1
    fi
done

if ! grep -q 'channel = "1.96.0"' rust-toolchain.toml; then
    echo "rust-toolchain.toml must pin Rust stable 1.96.0" >&2
    exit 1
fi
