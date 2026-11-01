//! Request-bound result accounting shared by policy and description consumers.
use crate::{
    evidence::Evidence,
    model::{ArtifactRequest, Code, Error, RequestId},
};
use std::collections::{BTreeMap, btree_map::Entry};
use taloon_decisions::observation::{Observation, ObservationError};

/// One terminal acquisition observation, associated with an expected request ID.
/// The ID alone grants no proof: consumers revalidate it through the ledger.
pub struct AcquisitionResult {
    id: RequestId,
    result: Result<Evidence, Error>,
}
impl AcquisitionResult {
    /// Associate evidence without trusting the supplied ID as an identity check.
    #[must_use]
    pub fn observed(id: RequestId, evidence: Evidence) -> Self {
        Self {
            id,
            result: Ok(evidence),
        }
    }
    /// Preserve a failed request as a terminal outcome instead of omitting it.
    #[must_use]
    pub fn failed(id: RequestId, error: Error) -> Self {
        Self {
            id,
            result: Err(error),
        }
    }
}

/// Consume each expected association once, retaining at most one payload per ID.
/// Duplicate status is permanent regardless of success/failure or arrival order.
/// Proof rebinding belongs to Evidence; permission and strength decisions remain
/// with the command. Unclaimed IDs remain available for foreign-result reporting.
pub(crate) struct ResultLedger {
    received: BTreeMap<RequestId, Observation<Result<Evidence, Error>>>,
}
impl ResultLedger {
    pub(crate) fn new(results: Vec<AcquisitionResult>) -> Self {
        let mut received = BTreeMap::new();
        for result in results {
            match received.entry(result.id) {
                Entry::Vacant(entry) => {
                    entry.insert(Observation::new(result.result));
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().mark_duplicate();
                }
            }
        }
        Self { received }
    }

    /// Missing or repeated observations cannot supply evidence. A single success
    /// is rebound to the expected package/content/source before reaching callers.
    pub(crate) fn take(&mut self, request: &ArtifactRequest) -> Result<Evidence, Error> {
        let mut slot = self.received.remove(&request.id());
        match Observation::take(&mut slot) {
            Err(ObservationError::Missing) => Err(Error::new(
                Code::MissingResult,
                "no acquisition result for expected artifact",
            )),
            Err(ObservationError::Duplicate) => Err(Error::new(
                Code::InvalidResult,
                "duplicate results for expected artifact",
            )),
            Ok(result) => result?.rebind(request),
        }
    }

    pub(crate) fn unclaimed(&self) -> impl Iterator<Item = &RequestId> {
        self.received.keys()
    }
}
