#!/usr/bin/env sh
set -eu

if grep -R -E "unsafe[[:space:]]*(\\{|fn|trait|impl|extern)" crates tools --include '*.rs' >/dev/null; then
    echo "unsafe code is not allowed in skrifheim core crates" >&2
    exit 1
fi

if grep -R "Fluxheim\\|Aesynx\\|Mjolni\\|CMS_PLAN\\|IDEA.md\\|IDEA.MD\\|CMS_IDEA" README.md docs .github SECURITY.md CHANGELOG.md release-notes 2>/dev/null; then
    echo "repository documentation contains copied or retired project wording" >&2
    exit 1
fi

check_no_sensitive_derive() {
    file="$1"
    type_name="$2"
    derive_name="$3"
    if awk -v type_name="$type_name" -v derive_name="$derive_name" '
        $0 ~ "#\\[derive\\(.*" derive_name ".*\\)\\]" {
            pending = 1
            next
        }
        pending && $0 ~ "^[[:space:]]*pub[[:space:]]+(struct|enum)[[:space:]]+" type_name "([^[:alnum:]_]|$)" {
            found = 1
        }
        pending && $0 !~ "^[[:space:]]*#\\[" && $0 !~ "^[[:space:]]*$" {
            pending = 0
        }
        END {
            exit found ? 0 : 1
        }
    ' "$file"; then
        echo "$file must not derive $derive_name for sensitive type $type_name" >&2
        exit 1
    fi
}

for derive_name in Debug PartialEq Eq; do
    check_no_sensitive_derive crates/skrifheim-core/src/lib.rs SecurityLabel "$derive_name"
    check_no_sensitive_derive crates/skrifheim-core/src/policy_token.rs PolicyTokenSet "$derive_name"
    check_no_sensitive_derive crates/skrifheim-core/src/policy_token.rs PolicyTokenSlot "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/lib.rs SignatureEnvelope "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/lib.rs SignatureSet "$derive_name"
done

check_no_sensitive_derive crates/skrifheim-core/src/lib.rs Value Debug
check_no_sensitive_derive crates/skrifheim-fact/src/lib.rs Fact Debug
check_no_sensitive_derive crates/skrifheim-fact/src/builder.rs FactBuilder Debug
