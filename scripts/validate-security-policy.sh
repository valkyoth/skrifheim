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
        function derive_contains_name(text) {
            return text ~ "(^|[^[:alnum:]_])(::)?([[:alnum:]_]+::)*" derive_name "([^[:alnum:]_]|$)"
        }
        $0 ~ "#\\[derive\\(" {
            in_derive = 1
            derive_text = $0
            if ($0 ~ "\\)\\]") {
                if (derive_contains_name(derive_text)) {
                    pending = 1
                }
                in_derive = 0
                derive_text = ""
            }
            next
        }
        in_derive {
            derive_text = derive_text " " $0
            if ($0 ~ "\\)\\]") {
                if (derive_contains_name(derive_text)) {
                    pending = 1
                }
                in_derive = 0
                derive_text = ""
            }
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
    if grep -E "^[[:space:]]*impl([[:space:]]*<[^>]*>)?[[:space:]]+(::)?([[:alnum:]_]+::)*$trait_name([[:space:]]*<[^>]*>)?[[:space:]]+for[[:space:]]+$type_name([^[:alnum:]_]|$)" "$file" >/dev/null; then
        echo "$file must not implement $trait_name for sensitive type $type_name" >&2
        exit 1
    fi
}

check_no_public_impl_method() {
    file="$1"
    type_name="$2"
    method_regex="$3"
    message="$4"
    if awk -v type_name="$type_name" -v method_regex="$method_regex" '
        function brace_delta(line, tmp, opens, closes) {
            tmp = line
            opens = gsub(/\{/, "", tmp)
            tmp = line
            closes = gsub(/\}/, "", tmp)
            return opens - closes
        }
        $0 ~ "^[[:space:]]*impl[[:space:]]+" type_name "[[:space:]]*\\{" {
            in_impl = 1
            depth = brace_delta($0)
            next
        }
        in_impl && $0 ~ "^[[:space:]]*pub[[:space:]]+(const[[:space:]]+)?fn[[:space:]]+(" method_regex ")[[:space:]]*\\(" {
            found = 1
        }
        in_impl {
            depth += brace_delta($0)
            if (depth <= 0) {
                in_impl = 0
            }
        }
        END {
            exit found ? 0 : 1
        }
    ' "$file"; then
        echo "$message" >&2
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
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs QueryResultInput "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs ResultClassification "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryRequest "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryPlan "$derive_name"
done

for derive_name in Debug; do
    check_no_sensitive_derive crates/skrifheim-policy/src/context.rs SubjectContext "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/context.rs DeviceContext "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/context.rs WorkloadContext "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/context.rs AuthorityContext "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PlannerDecision "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PolicyProof "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs QueryResultInput "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs ResultClassification "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryRequest "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryPlan "$derive_name"
done

for trait_name in PartialEq Eq; do
    check_no_sensitive_impl crates/skrifheim-policy/src/decision.rs PlannerDecision "$trait_name"
    check_no_sensitive_impl crates/skrifheim-policy/src/decision.rs PolicyProof "$trait_name"
    check_no_sensitive_impl crates/skrifheim-policy/src/result.rs QueryResultInput "$trait_name"
    check_no_sensitive_impl crates/skrifheim-policy/src/result.rs ResultClassification "$trait_name"
    check_no_sensitive_impl crates/skrifheim-query/src/lib.rs QueryRequest "$trait_name"
    check_no_sensitive_impl crates/skrifheim-query/src/lib.rs QueryPlan "$trait_name"
done

if grep -E "^[[:space:]]*pub[[:space:]]+enum[[:space:]]+PlannerDecision([^[:alnum:]_]|$)" crates/skrifheim-policy/src/decision.rs >/dev/null; then
    echo "PlannerDecision must not be a publicly constructible enum" >&2
    exit 1
fi

check_no_public_impl_method \
    crates/skrifheim-policy/src/decision.rs \
    PolicyProof \
    "new" \
    "PolicyProof::new must stay crate-private"

check_no_public_impl_method \
    crates/skrifheim-query/src/lib.rs \
    QueryRequest \
    "result_inputs" \
    "QueryRequest must not expose raw result inputs through a public accessor"

check_no_public_impl_method \
    crates/skrifheim-policy/src/result.rs \
    QueryResultInput \
    "label|sovereignty|pii|ai_processing|confidence_threshold" \
    "QueryResultInput must not expose raw metadata accessors publicly"

if grep -E "\\.field\\(\"intent\",[[:space:]]*&self\\.intent\\)" crates/skrifheim-query/src/lib.rs >/dev/null; then
    echo "QueryRequest and QueryPlan Debug must redact query intent" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(kind|decision)\",[[:space:]]*&self\\.(kind|decision|proof\\.decision\\(\\))\\)" crates/skrifheim-policy/src/decision.rs crates/skrifheim-query/src/lib.rs >/dev/null; then
    echo "Planner decision Debug output must redact decision state" >&2
    exit 1
fi

check_no_sensitive_derive crates/skrifheim-core/src/lib.rs Value Debug
check_no_sensitive_derive crates/skrifheim-fact/src/lib.rs Fact Debug
check_no_sensitive_derive crates/skrifheim-fact/src/builder.rs FactBuilder Debug
