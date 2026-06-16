use alloc::{string::String, vec::Vec};
use skrifheim_core::{
    Classification, PolicyTokenSet, Result, SecurityLabel, SkrifheimError, canonical_policy_set,
};

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

#[derive(Clone, Debug)]
pub struct QueryResultInput {
    label: SecurityLabel,
    sovereignty: PolicyTokenSet,
    pii: PiiMarker,
    ai_processing: AiProcessingEligibility,
    confidence_threshold: Option<ConfidenceThreshold>,
}

impl QueryResultInput {
    #[must_use]
    pub fn label_only(label: SecurityLabel) -> Self {
        Self {
            label,
            sovereignty: PolicyTokenSet::empty(),
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
            sovereignty: canonical_policy_set(sovereignty)?,
            pii,
            ai_processing,
            confidence_threshold,
        })
    }

    #[must_use]
    pub const fn label(&self) -> &SecurityLabel {
        &self.label
    }

    #[must_use]
    pub const fn sovereignty(&self) -> &PolicyTokenSet {
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
}

#[derive(Clone, Debug)]
pub struct ResultClassification {
    output_classification: Classification,
    sovereignty: PolicyTokenSet,
    pii: PiiMarker,
    ai_processing: AiProcessingEligibility,
    confidence_threshold: Option<ConfidenceThreshold>,
}

impl ResultClassification {
    #[must_use]
    pub fn public() -> Self {
        Self {
            output_classification: Classification::Public,
            sovereignty: PolicyTokenSet::empty(),
            pii: PiiMarker::NoPii,
            ai_processing: AiProcessingEligibility::Eligible,
            confidence_threshold: None,
        }
    }

    #[must_use]
    pub(crate) fn classification_only(output_classification: Classification) -> Self {
        Self {
            output_classification,
            sovereignty: PolicyTokenSet::empty(),
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
    pub const fn sovereignty(&self) -> &PolicyTokenSet {
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
        self.sovereignty = self.sovereignty.union(input.sovereignty())?;
        self.pii = self.pii.join(input.pii());
        self.ai_processing = self.ai_processing.join(input.ai_processing());
        self.confidence_threshold =
            join_thresholds(self.confidence_threshold, input.confidence_threshold());
        Ok(())
    }
}

impl PartialEq for ResultClassification {
    fn eq(&self, other: &Self) -> bool {
        self.output_classification == other.output_classification
            && self.sovereignty.structurally_equal(&other.sovereignty)
            && self.pii == other.pii
            && self.ai_processing == other.ai_processing
            && self.confidence_threshold == other.confidence_threshold
    }
}

impl Eq for ResultClassification {}

pub(crate) fn derive_result_classification(
    inputs: &[QueryResultInput],
) -> Result<ResultClassification> {
    if inputs.is_empty() {
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
