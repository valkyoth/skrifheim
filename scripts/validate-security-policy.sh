#!/usr/bin/env sh
set -eu

if grep -R -E "unsafe[[:space:]]*(\\{|fn|trait|impl|extern)" crates tools --include '*.rs' >/dev/null; then
    echo "unsafe code is not allowed in the v0.1.0 scaffold" >&2
    exit 1
fi

if grep -R "Fluxheim\\|Aesynx\\|Mjolni\\|CMS_PLAN\\|IDEA.md\\|IDEA.MD\\|CMS_IDEA" README.md docs .github SECURITY.md CHANGELOG.md release-notes 2>/dev/null; then
    echo "repository documentation contains copied or retired project wording" >&2
    exit 1
fi
