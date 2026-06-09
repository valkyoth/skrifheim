#!/usr/bin/env sh
set -eu

missing=0
for doc in README.md docs/*.md security/pentest/*.md release-notes/*.md; do
    [ -f "$doc" ] || continue
    refs="$(sed -n 's/.*](\([^)#][^)]*\.md\)).*/\1/p' "$doc")"
    for ref in $refs; do
        case "$ref" in
            http://*|https://*) continue ;;
        esac
        base="$(dirname "$doc")"
        target="$base/$ref"
        if [ ! -f "$target" ]; then
            echo "missing doc link in $doc: $ref" >&2
            missing=1
        fi
    done
done
exit "$missing"
