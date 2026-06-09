#!/usr/bin/env sh
set -eu

if command -v cargo-sbom >/dev/null 2>&1; then
    cargo sbom --output-format spdx_json_2_3 > sbom.spdx.json
else
    echo "cargo-sbom is not installed; install it before release tagging" >&2
    exit 1
fi
