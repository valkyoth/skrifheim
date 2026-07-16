#!/usr/bin/env sh
set -eu

cargo fmt --all --check
scripts/check_shell_syntax.sh
sh scripts/check_version_plan_semver_tests.sh
scripts/check_doc_links.sh
scripts/validate-release-metadata.sh
scripts/validate-engineering-policy.sh
scripts/validate-modularity-policy.sh
scripts/validate-security-policy.sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
