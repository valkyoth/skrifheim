use alloc::{string::String, vec::Vec};
use skrifheim_core::{
    Classification, PolicyTokenSet, Result, SecurityLabel, SkrifheimError, canonical_policy_set,
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

pub const RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES: usize =
    core::mem::size_of::<QueryResultInput>();

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
mod tests {
    use super::*;
    use alloc::{string::String, vec, vec::Vec};
    use skrifheim_core::POLICY_TOKEN_SET_MAX_ITEMS;

    fn input(
        classification: Classification,
        sovereignty: Vec<String>,
        pii: PiiMarker,
        ai_processing: AiProcessingEligibility,
        threshold: Option<u16>,
    ) -> Result<QueryResultInput> {
        QueryResultInput::new(
            SecurityLabel::new(classification, Vec::new(), Vec::new())?,
            sovereignty,
            pii,
            ai_processing,
            match threshold {
                Some(value) => Some(ConfidenceThreshold::new(value)?),
                None => None,
            },
        )
    }

    #[test]
    fn confidence_threshold_is_bounded() {
        assert!(matches!(
            ConfidenceThreshold::new(ConfidenceThreshold::MAX + 1),
            Err(SkrifheimError::InvalidConfidence)
        ));
    }

    #[test]
    fn result_classification_joins_all_metadata() -> Result<()> {
        let result = derive_result_classification(&[
            input(
                Classification::Restricted,
                vec![String::from("eu")],
                PiiMarker::NoPii,
                AiProcessingEligibility::Eligible,
                Some(500),
            )?,
            input(
                Classification::Secret,
                vec![String::from("se")],
                PiiMarker::ContainsPii,
                AiProcessingEligibility::NotEligible,
                Some(900),
            )?,
        ])?;

        assert_eq!(result.output_classification(), Classification::Secret);
        assert_eq!(result.sovereignty().len(), 2);
        assert!(result.sovereignty().contains("EU"));
        assert!(result.sovereignty().contains("SE"));
        assert_eq!(result.pii(), PiiMarker::ContainsPii);
        assert_eq!(result.ai_processing(), AiProcessingEligibility::NotEligible);
        assert_eq!(
            result.confidence_threshold(),
            Some(ConfidenceThreshold::new(900)?)
        );
        Ok(())
    }

    #[test]
    fn result_classification_rejects_empty_input_sets() {
        assert!(matches!(
            derive_result_classification(&[]),
            Err(SkrifheimError::InvalidQueryRequest)
        ));
    }

    #[test]
    fn result_classification_rejects_too_many_inputs() -> Result<()> {
        let mut inputs = Vec::new();
        for index in 0..=RESULT_CLASSIFICATION_INPUT_MAX_ITEMS {
            inputs.push(input(
                Classification::Public,
                vec![alloc::format!("JURISDICTION-{index}")],
                PiiMarker::NoPii,
                AiProcessingEligibility::Eligible,
                None,
            )?);
        }

        assert!(matches!(
            derive_result_classification(&inputs),
            Err(SkrifheimError::InvalidQueryRequest)
        ));
        Ok(())
    }

    #[test]
    fn result_classification_rejects_sovereignty_overflow() -> Result<()> {
        let mut first = Vec::new();
        let mut second = Vec::new();
        for index in 0..POLICY_TOKEN_SET_MAX_ITEMS {
            first.push(alloc::format!("FIRST-{index}"));
            second.push(alloc::format!("SECOND-{index}"));
        }
        let inputs = vec![
            input(
                Classification::Public,
                first,
                PiiMarker::NoPii,
                AiProcessingEligibility::Eligible,
                None,
            )?,
            input(
                Classification::Public,
                second,
                PiiMarker::NoPii,
                AiProcessingEligibility::Eligible,
                None,
            )?,
        ];

        assert!(matches!(
            derive_result_classification(&inputs),
            Err(SkrifheimError::InvalidSecurityToken)
        ));
        Ok(())
    }
}
