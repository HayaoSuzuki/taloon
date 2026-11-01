use proptest::prelude::*;
use rstest::rstest;
use taloon_decisions::evidence::{
    EvidenceRequirement, ProofBinding, content_matches, request_matches,
};

#[rstest]
#[case::sidecar_weak(false, false, true)]
#[case::sidecar_strong(false, true, false)]
#[case::archive_weak(true, false, true)]
#[case::archive_strong(true, true, true)]
fn strength_cannot_be_promoted(
    #[case] archive: bool,
    #[case] strong: bool,
    #[case] expected: bool,
) {
    // Arrange
    let id = [7; 32];
    let proof = if archive {
        ProofBinding::ArchiveVerified
    } else {
        ProofBinding::IndexAssociated {
            source: &id,
            request: &id,
        }
    };
    let need = if strong {
        EvidenceRequirement::ArchiveVerified
    } else {
        EvidenceRequirement::IndexAssociated
    };
    // Act
    let actual = proof.satisfies(need);
    // Assert
    assert_eq!(actual, expected);
}

proptest! {
    #[test]
    fn source_and_request_bindings_are_both_required(source in any::<[u8;32]>(), request in any::<[u8;32]>(), source_change in prop::option::of(0usize..32), request_change in prop::option::of(0usize..32)) {
        // Arrange
        let mut next_source = source;
        let mut next_request = request;
        if let Some(index) = source_change { next_source[index] ^= 1; }
        if let Some(index) = request_change { next_request[index] ^= 1; }
        let proof = ProofBinding::IndexAssociated { source:&source, request:&request };
        // Act
        let actual = proof.matches(&next_source, &next_request);
        // Assert
        prop_assert_eq!(actual, source_change.is_none() && request_change.is_none());
    }

    #[test]
    fn locked_content_rejects_any_changed_byte(content in any::<[u8;32]>(), changed in prop::option::of(0usize..32), pinned in any::<bool>()) {
        // Arrange
        let mut observed = content;
        if let Some(index) = changed { observed[index] ^= 1; }
        let locked = pinned.then_some(&content);
        // Act
        let actual = content_matches(locked, &observed);
        // Assert
        prop_assert_eq!(actual, !pinned || changed.is_none());
    }

    #[test]
    fn unpinned_reuse_requires_the_same_request(id in any::<[u8;32]>(), changed in prop::option::of(0usize..32), pinned in any::<bool>()) {
        // Arrange
        let mut next = id;
        if let Some(index) = changed { next[index] ^= 1; }
        let locked = pinned.then_some(&id);
        // Act
        let actual = request_matches(locked, &id, &next);
        // Assert
        prop_assert_eq!(actual, pinned || changed.is_none());
    }
}

#[rstest]
#[case::missing(false, false, false)]
#[case::missing_changed(false, true, false)]
#[case::locked(true, false, true)]
#[case::wrong_content(true, true, false)]
fn sidecars_require_a_locked_matching_artifact(
    #[case] pinned: bool,
    #[case] changed: bool,
    #[case] expected: bool,
) {
    // Arrange
    let content = [9; 32];
    let mut advertised = content;
    if changed {
        advertised[0] ^= 1;
    }
    // Act
    let actual =
        taloon_decisions::evidence::sidecar_matches(pinned.then_some(&content), &advertised);
    // Assert
    assert_eq!(actual, expected);
}
