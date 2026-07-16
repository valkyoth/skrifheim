#!/usr/bin/env sh
set -eu

version_plan="${1:-docs/VERSION_PLAN.md}"

awk '
    /^## v/ {
        if ($0 !~ /^## v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)( -|$)/) {
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
' "$version_plan"
