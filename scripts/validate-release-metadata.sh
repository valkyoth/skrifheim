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
docs/memory-secrecy.md
docs/security-controls.md
docs/threat-model.md
scripts/validate-release-readiness.sh
release-notes/RELEASE_NOTES_0.1.0.md
release-notes/RELEASE_NOTES_0.2.0.md
release-notes/RELEASE_NOTES_0.3.0.md
release-notes/RELEASE_NOTES_0.4.0.md
release-notes/RELEASE_NOTES_0.5.0.md
release-notes/RELEASE_NOTES_0.6.0.md
release-notes/RELEASE_NOTES_0.7.0.md
release-notes/RELEASE_NOTES_0.8.0.md
release-notes/RELEASE_NOTES_0.9.0.md
release-notes/RELEASE_NOTES_0.10.0.md
release-notes/RELEASE_NOTES_0.11.0.md
release-notes/RELEASE_NOTES_0.12.0.md
release-notes/RELEASE_NOTES_0.13.0.md
release-notes/RELEASE_NOTES_0.14.0.md
release-notes/RELEASE_NOTES_0.15.0.md
release-notes/RELEASE_NOTES_0.16.0.md
release-notes/RELEASE_NOTES_0.17.0.md
release-notes/RELEASE_NOTES_0.18.0.md
release-notes/RELEASE_NOTES_0.18.1.md
release-notes/RELEASE_NOTES_0.18.2.md
security/pentest/v0.1.0.md
security/pentest/v0.2.0.md
security/pentest/v0.3.0.md
security/pentest/v0.4.0.md
security/pentest/v0.5.0.md
security/pentest/v0.6.0.md
security/pentest/v0.7.0.md
security/pentest/v0.8.0.md
security/pentest/v0.9.0.md
security/pentest/v0.10.0.md
security/pentest/v0.11.0.md
security/pentest/v0.12.0.md
security/pentest/v0.13.0.md
security/pentest/v0.14.0.md
security/pentest/v0.15.0.md
security/pentest/v0.16.0.md
security/pentest/v0.17.0.md
security/pentest/v0.18.0.md
security/pentest/v0.18.1.md
security/pentest/v0.18.2.md
"

for path in $required; do
    if [ ! -f "$path" ]; then
        echo "missing required release metadata: $path" >&2
        exit 1
    fi
done

if ! grep -q 'channel = "1.97.1"' rust-toolchain.toml; then
    echo "rust-toolchain.toml must pin Rust stable 1.97.1" >&2
    exit 1
fi

awk '
    /^## v/ {
        if ($0 !~ /^## v[0-9]+\.[0-9]+\.[0-9]+( -|$)/) {
            print "VERSION_PLAN.md heading is not SemVer: " $0 > "/dev/stderr";
            exit 1;
        }

        version = $2;
        sub(/^v/, "", version);
        split(version, parts, ".");

        major = parts[1] + 0;
        minor = parts[2] + 0;
        patch = parts[3] + 0;

        not_increasing = major < last_major
        not_increasing = not_increasing || (major == last_major && minor < last_minor)
        not_increasing = not_increasing || (major == last_major && minor == last_minor && patch <= last_patch)

        if (seen && not_increasing) {
            print "VERSION_PLAN.md headings must be strictly increasing SemVer: " $0 > "/dev/stderr";
            exit 1;
        }

        seen = 1;
        last_major = major;
        last_minor = minor;
        last_patch = patch;
    }
' docs/VERSION_PLAN.md
