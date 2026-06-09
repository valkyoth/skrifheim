#!/usr/bin/env sh
set -eu

limit=500
failed=0

for file in $(find crates tools -type f -name '*.rs'); do
    lines="$(wc -l < "$file" | tr -d ' ')"
    if [ "$lines" -gt "$limit" ]; then
        echo "$file has $lines lines; split before exceeding $limit lines" >&2
        failed=1
    fi
done

exit "$failed"
