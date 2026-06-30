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
    check_no_sensitive_derive crates/skrifheim-crypto/src/projection.rs ProjectionSurface "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/domain.rs EncryptionDomain "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/projection.rs ProjectionEncryptionPolicy "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/lib.rs SignatureEnvelope "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/lib.rs SignatureSet "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/lib.rs AuditIdentity "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/lib.rs AttestationEvidenceRef "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/lib.rs DeviceAuditContext "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/lib.rs WorkloadAuditContext "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/lib.rs AuditEvent "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/protection.rs AuditLogProtection "$derive_name"
    check_no_sensitive_derive crates/skrifheim-audit/src/protection.rs AuditRecord "$derive_name"
done

for derive_name in PartialEq Eq; do
    check_no_sensitive_derive crates/skrifheim-crypto/src/digest.rs DigestValue "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/digest.rs WorldIdentityDigest "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/digest.rs ContentDigest "$derive_name"
    check_no_sensitive_derive crates/skrifheim-crypto/src/digest.rs ManifestDigest "$derive_name"
done

check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentHeader Debug
check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentHeaderInput Debug
check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentFooter Debug
check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentFooterInput Debug
check_no_sensitive_derive crates/skrifheim-storage/src/wal.rs WalFrameHeader Debug
check_no_sensitive_derive crates/skrifheim-storage/src/wal.rs WalFrameHeaderInput Debug
check_no_sensitive_derive crates/skrifheim-world/src/lib.rs WorldMetadata Debug
check_no_sensitive_derive crates/skrifheim-world/src/lib.rs World Debug

for derive_name in Debug Clone PartialEq Eq; do
    check_no_sensitive_derive crates/skrifheim-crypto/src/secret.rs SecretBytes "$derive_name"
done

for derive_name in PartialEq Eq; do
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PlannerDecision "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/decision.rs PolicyProof "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs QueryResultInput "$derive_name"
    check_no_sensitive_derive crates/skrifheim-policy/src/result.rs ResultClassification "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryRequest "$derive_name"
    check_no_sensitive_derive crates/skrifheim-query/src/lib.rs QueryPlan "$derive_name"
    check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentHeader "$derive_name"
    check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentHeaderInput "$derive_name"
    check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentFooter "$derive_name"
    check_no_sensitive_derive crates/skrifheim-storage/src/segment.rs SegmentFooterInput "$derive_name"
    check_no_sensitive_derive crates/skrifheim-storage/src/wal.rs WalFrameHeader "$derive_name"
    check_no_sensitive_derive crates/skrifheim-storage/src/wal.rs WalFrameHeaderInput "$derive_name"
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

if grep -E "^[[:space:]]*pub[[:space:]]+struct[[:space:]]+Timestamp[[:space:]]*\\([[:space:]]*pub[[:space:]]+" crates/skrifheim-core/src/lib.rs >/dev/null; then
    echo "Timestamp must not expose a public tuple field" >&2
    exit 1
fi

if grep -n "debug_assert!" crates/skrifheim-policy/src/decision.rs >/dev/null; then
    echo "policy decision invariants must not rely on debug_assert" >&2
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

check_no_public_impl_method \
    crates/skrifheim-crypto/src/secret.rs \
    SecretBytes \
    "as_slice|as_mut_slice|to_vec|into_vec|bytes|secret_bytes|expose" \
    "SecretBytes must not expose raw secret bytes through public accessors"

if grep -E "\\.field\\(\"intent\",[[:space:]]*&self\\.intent\\)" crates/skrifheim-query/src/lib.rs >/dev/null; then
    echo "QueryRequest and QueryPlan Debug must redact query intent" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(classification|compartment_count|releasable_to_count)\",[[:space:]]*&self\\.(classification|compartments\\.len\\(\\)|releasable_to\\.len\\(\\))\\)" crates/skrifheim-core/src/lib.rs >/dev/null; then
    echo "SecurityLabel Debug must redact classification and token counts" >&2
    exit 1
fi

if awk '
    /^[[:space:]]*impl[[:space:]]+fmt::Debug[[:space:]]+for[[:space:]]+PolicyTokenSet[[:space:]]*\{/ {
        in_impl = 1
    }
    in_impl && /^[[:space:]]*impl[[:space:]]+/ && $0 !~ "PolicyTokenSet" {
        in_impl = 0
    }
    in_impl && /\.field\("len",[[:space:]]*&self\.len\)/ {
        found = 1
    }
    END {
        exit found ? 0 : 1
    }
' crates/skrifheim-core/src/policy_token.rs; then
    echo "PolicyTokenSet Debug must redact exact token counts" >&2
    exit 1
fi

if grep -E "\\.field\\(\"classification\",[[:space:]]*&self\\.label\\.classification\\(\\)\\)" crates/skrifheim-fact/src/lib.rs >/dev/null; then
    echo "Fact Debug must redact label classification" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(asserted_by|policy_id)\",[[:space:]]*&self\\.(asserted_by|policy_id)\\)" crates/skrifheim-fact/src/lib.rs >/dev/null; then
    echo "Fact Debug must redact asserted_by actor attribution and policy identifiers" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(id|world_id|subject|predicate|valid_time|committed_at|confidence|asserted_by|policy_id)\",[[:space:]]*&self\\." crates/skrifheim-fact/src/lib.rs >/dev/null; then
    echo "Fact Debug must redact structural identifiers, time, confidence, actor attribution, and policy identifiers" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(asserted_by|policy_id)\",[[:space:]]*&self\\.(asserted_by|policy_id)\\)" crates/skrifheim-fact/src/builder.rs >/dev/null; then
    echo "FactBuilder Debug must redact asserted_by actor attribution and policy identifiers" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(id|world_id|subject|predicate|valid_time|committed_at|confidence)\",[[:space:]]*&self\\." crates/skrifheim-fact/src/builder.rs >/dev/null; then
    echo "FactBuilder Debug must redact structural identifiers, time, and confidence" >&2
    exit 1
fi

if grep -E "SecurityLabel::classification|\\.classification\\(\\)" crates/skrifheim-fact/src/builder.rs >/dev/null; then
    echo "FactBuilder Debug must redact label classification" >&2
    exit 1
fi

if grep -E "\\.field\\(\"len\",[[:space:]]*&self\\.len\\)" crates/skrifheim-core/src/policy_token.rs >/dev/null; then
    echo "PolicyTokenSlot Debug must redact exact token length" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(algorithm|epoch|key_id|signature_bytes)\",[[:space:]]*&self\\.(algorithm|epoch|key_id|signature\\.len\\(\\))\\)" crates/skrifheim-crypto/src/lib.rs >/dev/null; then
    echo "SignatureEnvelope Debug must redact key, algorithm, epoch, and signature length" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(purpose|tenant_id|region_id|classification|compartment_id|world_id|segment_id)\",[[:space:]]*&self\\.(purpose|tenant_id|region_id|classification|compartment_id|world_id|segment_id)\\)" crates/skrifheim-crypto/src/domain.rs >/dev/null; then
    echo "EncryptionDomain Debug must redact domain metadata" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(surface|domain)\",[[:space:]]*&self\\.(surface|domain)\\)" crates/skrifheim-crypto/src/projection.rs >/dev/null; then
    echo "ProjectionEncryptionPolicy Debug must redact surface and domain" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(magic|tenant_id|tx_range|policy_id|encryption_key_id|crypto_epoch|encryption_domain|body_crc64|content_hash|content_digest)\",[[:space:]]*&self\\." crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "SegmentHeader and SegmentFooter Debug must redact identifiers, keys, epochs, domains, checksums, and hashes" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(magic|tenant_id|tx_id|encryption_key_id|crypto_epoch|encryption_domain|body_crc64)\",[[:space:]]*&self\\." crates/skrifheim-storage/src/wal.rs >/dev/null; then
    echo "WalFrameHeader Debug must redact identifiers, keys, domains, epochs, and checksums" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(id|tenant_id|name|kind|parent|metadata|added_facts|hidden_facts)\",[[:space:]]*&self\\." crates/skrifheim-world/src/lib.rs >/dev/null; then
    echo "World Debug output must redact operational metadata and fact lists" >&2
    exit 1
fi

if awk '
    /^[[:space:]]*pub[[:space:]]+struct[[:space:]]+SegmentHeader[[:space:]]*\{/ {
        in_struct = 1
        next
    }
    in_struct && /^[[:space:]]*\}/ {
        in_struct = 0
    }
    in_struct && /^[[:space:]]*pub[[:space:]]+[[:alnum:]_]+[[:space:]]*:/ {
        found = 1
    }
    END {
        exit found ? 0 : 1
    }
' crates/skrifheim-storage/src/segment.rs; then
    echo "SegmentHeader fields must stay private; use the validating constructor" >&2
    exit 1
fi

if awk '
    /^[[:space:]]*pub[[:space:]]+struct[[:space:]]+SegmentFooter[[:space:]]*\{/ {
        in_struct = 1
        next
    }
    in_struct && /^[[:space:]]*\}/ {
        in_struct = 0
    }
    in_struct && /^[[:space:]]*pub[[:space:]]+[[:alnum:]_]+[[:space:]]*:/ {
        found = 1
    }
    END {
        exit found ? 0 : 1
    }
' crates/skrifheim-storage/src/segment.rs; then
    echo "SegmentFooter fields must stay private; use the validating constructor" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(bytes|len|capacity|contents)\",[[:space:]]*&self\\.bytes" crates/skrifheim-crypto/src/secret.rs >/dev/null; then
    echo "SecretBytes Debug must redact secret contents and exact size" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(event_id|tenant_id|occurred_at|device|workload|targets|crypto_epoch|domain|signature_count)\",[[:space:]]*&self\\." crates/skrifheim-audit/src/lib.rs crates/skrifheim-audit/src/protection.rs >/dev/null; then
    echo "Audit Debug output must redact identifiers, targets, domains, signatures, and epochs" >&2
    exit 1
fi

if grep -E "\\.field\\(\"kind\",[[:space:]]*&self\\.kind\\)" crates/skrifheim-audit/src/lib.rs >/dev/null; then
    echo "AuditEvent Debug must redact event kind" >&2
    exit 1
fi

if grep -E "\\.field\\(\"kind\",[[:space:]]*&self\\.kind\\(\\)\\)" crates/skrifheim-audit/src/lib.rs >/dev/null; then
    echo "AuditIdentity Debug must redact identity kind" >&2
    exit 1
fi

if ! grep -q '"<redacted>"' crates/skrifheim-crypto/src/domain.rs; then
    echo "EncryptionDomain Debug must use redacted field values" >&2
    exit 1
fi

if ! grep -q '"<redacted>"' crates/skrifheim-crypto/src/projection.rs; then
    echo "ProjectionSurface and ProjectionEncryptionPolicy Debug must use redacted field values" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(world|result_input_count|result_inputs|proof)\",[[:space:]]*&self\\.(world|result_inputs|result_inputs\\.len\\(\\)|proof)" crates/skrifheim-query/src/lib.rs >/dev/null; then
    echo "QueryRequest and QueryPlan Debug must redact world, inputs, counts, and proof" >&2
    exit 1
fi

if grep -E "\\.field\\(\"(kind|decision)\",[[:space:]]*&self\\.(kind|decision|proof\\.decision\\(\\))\\)" crates/skrifheim-policy/src/decision.rs crates/skrifheim-query/src/lib.rs >/dev/null; then
    echo "Planner decision Debug output must redact decision state" >&2
    exit 1
fi

if grep -E "\\.any\\([|][^|]*[|][[:space:]]*[^)]*structurally_equal" crates/skrifheim-core/src/policy_token.rs >/dev/null; then
    echo "PolicyTokenSet deduplication must not use early-exit structural scans" >&2
    exit 1
fi

check_no_sensitive_derive crates/skrifheim-core/src/lib.rs Value Debug
check_no_sensitive_derive crates/skrifheim-fact/src/lib.rs Fact Debug
check_no_sensitive_derive crates/skrifheim-fact/src/builder.rs FactBuilder Debug
