use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt;
use skrifheim_core::{
    Classification, POLICY_TOKEN_SET_MAX_ITEMS, PolicyTokenSet, Result, SecurityLabel,
    SkrifheimError, canonical_policy_set, canonical_policy_token,
};

pub const RESULT_CLASSIFICATION_INPUT_MAX_ITEMS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PiiMarker {
    NoPii,
    ContainsPii,
}

impl PiiMarker {
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::ContainsPii, _) | (_, Self::ContainsPii) => Self::ContainsPii,
            (Self::NoPii, Self::NoPii) => Self::NoPii,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiProcessingEligibility {
    Eligible,
    NotEligible,
}

impl AiProcessingEligibility {
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::NotEligible, _) | (_, Self::NotEligible) => Self::NotEligible,
            (Self::Eligible, Self::Eligible) => Self::Eligible,
        }
    }
}

#[derive(Clone)]
pub struct SovereigntyScope {
    kind: SovereigntyScopeKind,
}

#[derive(Clone)]
enum SovereigntyScopeKind {
    Exact(Box<PolicyTokenSet>),
    MultiJurisdiction,
}

impl SovereigntyScope {
    #[must_use]
    pub fn public() -> Self {
        Self {
            kind: SovereigntyScopeKind::Exact(Box::new(PolicyTokenSet::empty())),
        }
    }

    pub fn from_tokens(values: Vec<String>) -> Result<Self> {
        let mut exact = Vec::new();
        let mut saturated = false;
        for value in values {
            let value = canonical_policy_token(value)?;
            if exact.iter().any(|existing| existing == &value) {
                continue;
            }
            if exact.len() == POLICY_TOKEN_SET_MAX_ITEMS {
                saturated = true;
            } else if !saturated {
                exact.push(value);
            }
        }
        if saturated {
            Ok(Self::multi_jurisdiction())
        } else {
            Ok(Self {
                kind: SovereigntyScopeKind::Exact(Box::new(canonical_policy_set(exact)?)),
            })
        }
    }

    #[must_use]
    pub const fn multi_jurisdiction() -> Self {
        Self {
            kind: SovereigntyScopeKind::MultiJurisdiction,
        }
    }

    #[must_use]
    pub fn is_multi_jurisdiction(&self) -> bool {
        matches!(self.kind, SovereigntyScopeKind::MultiJurisdiction)
    }

    #[must_use]
    pub fn is_exact(&self) -> bool {
        matches!(self.kind, SovereigntyScopeKind::Exact(_))
    }

    #[must_use]
    pub fn exact_tokens(&self) -> Option<&PolicyTokenSet> {
        match &self.kind {
            SovereigntyScopeKind::Exact(tokens) => Some(tokens.as_ref()),
            SovereigntyScopeKind::MultiJurisdiction => None,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match &self.kind {
            SovereigntyScopeKind::Exact(tokens) => tokens.len(),
            SovereigntyScopeKind::MultiJurisdiction => POLICY_TOKEN_SET_MAX_ITEMS,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.kind {
            SovereigntyScopeKind::Exact(tokens) => tokens.is_empty(),
            SovereigntyScopeKind::MultiJurisdiction => false,
        }
    }

    #[must_use]
    pub fn contains(&self, needle: &str) -> bool {
        match &self.kind {
            SovereigntyScopeKind::Exact(tokens) => tokens.contains(needle),
            SovereigntyScopeKind::MultiJurisdiction => false,
        }
    }

    #[must_use]
    pub fn requires_restrictive_handling(&self) -> bool {
        self.is_multi_jurisdiction()
    }

    pub fn join(&self, other: &Self) -> Result<Self> {
        match (&self.kind, &other.kind) {
            (SovereigntyScopeKind::MultiJurisdiction, _)
            | (_, SovereigntyScopeKind::MultiJurisdiction) => Ok(Self::multi_jurisdiction()),
            (SovereigntyScopeKind::Exact(left), SovereigntyScopeKind::Exact(right)) => {
                match left.union(right) {
                    Ok(tokens) => Ok(Self {
                        kind: SovereigntyScopeKind::Exact(Box::new(tokens)),
                    }),
                    Err(SkrifheimError::InvalidSecurityToken) => Ok(Self::multi_jurisdiction()),
                    Err(error) => Err(error),
                }
            }
        }
    }
}

impl fmt::Debug for SovereigntyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SovereigntyScope")
            .field("kind", &"<redacted>")
            .field("tokens", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ConfidenceThreshold(u16);

impl ConfidenceThreshold {
    pub const MAX: u16 = 1000;

    pub const fn new(value: u16) -> Result<Self> {
        if value > Self::MAX {
            return Err(SkrifheimError::InvalidConfidence);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        if self.0 >= other.0 { self } else { other }
    }
}

#[derive(Clone)]
pub struct QueryResultInput {
    label: SecurityLabel,
    sovereignty: SovereigntyScope,
    pii: PiiMarker,
    ai_processing: AiProcessingEligibility,
    confidence_threshold: Option<ConfidenceThreshold>,
}

pub const RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES: usize =
    core::mem::size_of::<QueryResultInput>();

impl QueryResultInput {
    #[must_use]
    pub fn label_only(label: SecurityLabel) -> Self {
        Self {
            label,
            sovereignty: SovereigntyScope::public(),
            pii: PiiMarker::NoPii,
            ai_processing: AiProcessingEligibility::Eligible,
            confidence_threshold: None,
        }
    }

    pub fn new(
        label: SecurityLabel,
        sovereignty: Vec<String>,
        pii: PiiMarker,
        ai_processing: AiProcessingEligibility,
        confidence_threshold: Option<ConfidenceThreshold>,
    ) -> Result<Self> {
        Ok(Self {
            label,
            sovereignty: SovereigntyScope::from_tokens(sovereignty)?,
            pii,
            ai_processing,
            confidence_threshold,
        })
    }

    #[must_use]
    pub(crate) const fn label(&self) -> &SecurityLabel {
        &self.label
    }

    #[must_use]
    pub(crate) const fn pii(&self) -> PiiMarker {
        self.pii
    }

    #[must_use]
    pub(crate) const fn ai_processing(&self) -> AiProcessingEligibility {
        self.ai_processing
    }

    #[must_use]
    pub(crate) const fn confidence_threshold(&self) -> Option<ConfidenceThreshold> {
        self.confidence_threshold
    }
}

impl fmt::Debug for QueryResultInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryResultInput")
            .field("label", &"<redacted>")
            .field("sovereignty", &"<redacted>")
            .field("pii", &"<redacted>")
            .field("ai_processing", &"<redacted>")
            .field("confidence_threshold", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct ResultClassification {
    output_classification: Classification,
    sovereignty: SovereigntyScope,
    pii: PiiMarker,
    ai_processing: AiProcessingEligibility,
    confidence_threshold: Option<ConfidenceThreshold>,
}

impl ResultClassification {
    #[must_use]
    pub fn public() -> Self {
        Self {
            output_classification: Classification::Public,
            sovereignty: SovereigntyScope::public(),
            pii: PiiMarker::NoPii,
            ai_processing: AiProcessingEligibility::Eligible,
            confidence_threshold: None,
        }
    }

    #[must_use]
    pub(crate) fn classification_only(output_classification: Classification) -> Self {
        Self {
            output_classification,
            sovereignty: SovereigntyScope::public(),
            pii: PiiMarker::NoPii,
            ai_processing: AiProcessingEligibility::Eligible,
            confidence_threshold: None,
        }
    }

    #[must_use]
    pub const fn output_classification(&self) -> Classification {
        self.output_classification
    }

    #[must_use]
    pub const fn sovereignty(&self) -> &SovereigntyScope {
        &self.sovereignty
    }

    #[must_use]
    pub const fn pii(&self) -> PiiMarker {
        self.pii
    }

    #[must_use]
    pub const fn ai_processing(&self) -> AiProcessingEligibility {
        self.ai_processing
    }

    #[must_use]
    pub const fn confidence_threshold(&self) -> Option<ConfidenceThreshold> {
        self.confidence_threshold
    }

    fn absorb(&mut self, input: &QueryResultInput) -> Result<()> {
        if input.label.classification() > self.output_classification {
            self.output_classification = input.label.classification();
        }
        self.sovereignty = self.sovereignty.join(&input.sovereignty)?;
        self.pii = self.pii.join(input.pii());
        self.ai_processing = self.ai_processing.join(input.ai_processing());
        self.confidence_threshold =
            join_thresholds(self.confidence_threshold, input.confidence_threshold());
        Ok(())
    }
}

impl fmt::Debug for ResultClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResultClassification")
            .field("output_classification", &"<redacted>")
            .field("sovereignty", &"<redacted>")
            .field("pii", &"<redacted>")
            .field("ai_processing", &"<redacted>")
            .field("confidence_threshold", &"<redacted>")
            .finish()
    }
}

pub(crate) fn derive_result_classification(
    inputs: &[QueryResultInput],
) -> Result<ResultClassification> {
    if inputs.is_empty() || inputs.len() > RESULT_CLASSIFICATION_INPUT_MAX_ITEMS {
        return Err(SkrifheimError::InvalidQueryRequest);
    }
    let mut result = ResultClassification::public();
    for input in inputs {
        result.absorb(input)?;
    }
    Ok(result)
}

const fn join_thresholds(
    left: Option<ConfidenceThreshold>,
    right: Option<ConfidenceThreshold>,
) -> Option<ConfidenceThreshold> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.join(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

#[cfg(test)]
#[path = "result_tests.rs"]
mod tests;
