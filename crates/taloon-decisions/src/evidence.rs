//! Evidence strength and exact content/source/request association decisions.
#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(
    not(creusot),
    derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)
)]
#[cfg_attr(
    all(feature = "serde", not(creusot)),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(all(feature = "serde", not(creusot)), serde(rename_all = "snake_case"))]
pub enum EvidenceRequirement {
    IndexAssociated,
    ArchiveVerified,
}

/// Borrowed identities of an observation; hashing and provenance acquisition stay outside.
pub enum ProofBinding<'a> {
    ArchiveVerified,
    IndexAssociated {
        source: &'a [u8; 32],
        request: &'a [u8; 32],
    },
}

#[cfg_attr(creusot, ensures(result == (left@ == right@)))]
fn same(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left == right
}

impl ProofBinding<'_> {
    /// Sidecars cannot meet an archive-verification requirement.
    #[must_use]
    #[cfg_attr(creusot, ensures(result == match self {
        ProofBinding::ArchiveVerified => true,
        ProofBinding::IndexAssociated { .. } => need == EvidenceRequirement::IndexAssociated,
    }))]
    pub fn satisfies(&self, need: EvidenceRequirement) -> bool {
        match (self, need) {
            (Self::ArchiveVerified, _) | (_, EvidenceRequirement::IndexAssociated) => true,
            (Self::IndexAssociated { .. }, EvidenceRequirement::ArchiveVerified) => false,
        }
    }

    /// Report the observed proof kind without strengthening it.
    #[must_use]
    #[cfg_attr(creusot, ensures(result == match self {
        ProofBinding::ArchiveVerified => EvidenceRequirement::ArchiveVerified,
        ProofBinding::IndexAssociated { .. } => EvidenceRequirement::IndexAssociated,
    }))]
    pub fn strength(&self) -> EvidenceRequirement {
        match self {
            Self::ArchiveVerified => EvidenceRequirement::ArchiveVerified,
            Self::IndexAssociated { .. } => EvidenceRequirement::IndexAssociated,
        }
    }

    /// Sidecar evidence remains bound to both the original source and request.
    #[must_use]
    #[cfg_attr(creusot, ensures(result == match self {
        ProofBinding::ArchiveVerified => true,
        ProofBinding::IndexAssociated {source, request} => source@ == expected_source@ && request@ == expected_request@,
    }))]
    pub fn matches(&self, expected_source: &[u8; 32], expected_request: &[u8; 32]) -> bool {
        match self {
            Self::ArchiveVerified => true,
            Self::IndexAssociated { source, request } => {
                same(source, expected_source) && same(request, expected_request)
            }
        }
    }
}

/// A lock digest, when present, must equal observed content exactly.
#[must_use]
#[cfg_attr(creusot, ensures(result == match locked { None => true, Some(id) => id@ == observed@ }))]
pub fn content_matches(locked: Option<&[u8; 32]>, observed: &[u8; 32]) -> bool {
    match locked {
        None => true,
        Some(id) => same(id, observed),
    }
}

/// Unpinned content cannot cross request identities, even after successful acquisition.
#[must_use]
#[cfg_attr(creusot, ensures(result == match locked { Some(_) => true, None => original@ == expected@ }))]
pub fn request_matches(
    locked: Option<&[u8; 32]>,
    original: &[u8; 32],
    expected: &[u8; 32],
) -> bool {
    match locked {
        Some(_) => true,
        None => same(original, expected),
    }
}

/// An index sidecar requires a present lock digest matching the advertised artifact.
#[must_use]
#[cfg_attr(creusot, ensures(result == match locked {None => false, Some(id) => id@ == advertised@}))]
pub fn sidecar_matches(locked: Option<&[u8; 32]>, advertised: &[u8; 32]) -> bool {
    match locked {
        None => false,
        Some(id) => same(id, advertised),
    }
}
