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

check_no_sensitive_impl() {
    file="$1"
    type_name="$2"
    trait_name="$3"
    if grep -E "impl([[:space:]]+[^[:space:]]+)?[[:space:]]+$trait_name[[:space:]]+for[[:space:]]+$type_name([^[:alnum:]_]|$)" "$file" >/dev/null; then
        echo "$file must not implement $trait_name for sensitive type $type_name" >&2
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

for derive_name in PartialEq Eq; do
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PlannerDecision "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PolicyProof "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs ResultClassification "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryPlan "$derive_name"
done

for derive_name in Debug; do
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PlannerDecision "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PolicyProof "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs QueryResultInput "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs ResultClassification "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryRequest "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryPlan "$derive_name"
done

for trait_name in PartialEq Eq; do
    check_no_sensitive_impl crates/skrifheim-policy/src/result.rs ResultClassification "$trait_name"
done

if grep -E "^[[:space:]]*pub[[:space:]]+enum[[:space:]]+PlannerDecision([^[:alnum:]_]|$)" crates/skrifheim-policy/src/decision.rs >/dev/null; then
    echo "PlannerDecision must not be a publicly constructible enum" >&2
    exit 1
fi

if awk '
    /^[[:space:]]*impl[[:space:]]+PolicyProof[[:space:]]*\{/ {
        in_policy_proof = 1
        next
    }
    in_policy_proof && /^[[:space:]]*\}/ {
        in_policy_proof = 0
    }
    in_policy_proof && /^[[:space:]]*pub[[:space:]]+fn[[:space:]]+new[[:space:]]*\(/ {
        found = 1
    }
    END {
        exit found ? 0 : 1
    }
' crates/skrifheim-policy/src/decision.rs; then
    echo "PolicyProof::new must stay crate-private" >&2
    exit 1
fi

check_no_sensitive_derive crates/skrifheim-core/src/lib.rs Value Debug
check_no_sensitive_derive crates/skrifheim-fact/src/lib.rs Fact Debug
check_no_sensitive_derive crates/skrifheim-fact/src/builder.rs FactBuilder Debug
