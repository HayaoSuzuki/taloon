use proptest::prelude::*;
use rstest::rstest;
use taloon_decisions::status::Status;

#[rstest]
#[case::compliant_compliant(Status::Compliant, Status::Compliant, Status::Compliant)]
#[case::compliant_violation(Status::Compliant, Status::Violation, Status::Violation)]
#[case::compliant_incomplete(Status::Compliant, Status::Incomplete, Status::Incomplete)]
#[case::violation_compliant(Status::Violation, Status::Compliant, Status::Violation)]
#[case::violation_violation(Status::Violation, Status::Violation, Status::Violation)]
#[case::violation_incomplete(Status::Violation, Status::Incomplete, Status::Incomplete)]
#[case::incomplete_compliant(Status::Incomplete, Status::Compliant, Status::Incomplete)]
#[case::incomplete_violation(Status::Incomplete, Status::Violation, Status::Incomplete)]
#[case::incomplete_incomplete(Status::Incomplete, Status::Incomplete, Status::Incomplete)]
fn combined_status_preserves_precedence(
    #[case] left: Status,
    #[case] right: Status,
    #[case] expected: Status,
) {
    // Arrange: the two statuses are independent observations.
    // Act
    let result = left.combine(right);
    // Assert
    assert_eq!(result, expected);
}

#[rstest]
#[case::allowed(Status::Compliant, false, false, 0)]
#[case::warnings_allowed(Status::Compliant, false, true, 0)]
#[case::deny_without_warnings(Status::Compliant, true, false, 0)]
#[case::denied_warning(Status::Compliant, true, true, 1)]
#[case::violation(Status::Violation, false, false, 1)]
#[case::violation_warning(Status::Violation, false, true, 1)]
#[case::violation_deny(Status::Violation, true, false, 1)]
#[case::violation_deny_warning(Status::Violation, true, true, 1)]
#[case::incomplete(Status::Incomplete, false, false, 2)]
#[case::incomplete_warning(Status::Incomplete, false, true, 2)]
#[case::incomplete_deny(Status::Incomplete, true, false, 2)]
#[case::incomplete_deny_warning(Status::Incomplete, true, true, 2)]
fn exit_status_obeys_warning_policy(
    #[case] status: Status,
    #[case] deny: bool,
    #[case] warnings: bool,
    #[case] expected: u8,
) {
    // Arrange: status and warning policy are supplied independently.
    // Act
    let result = status.exit_code(deny, warnings);
    // Assert
    assert_eq!(result, expected);
}

proptest! {
    #[test]
    fn status_fold_matches_independent_maximum(ranks in prop::collection::vec(0_u8..=2, 0..100), deny in any::<bool>(), warnings in any::<bool>()) {
        // Arrange
        let statuses: Vec<_> = ranks.iter().map(|rank| match rank {
            0 => Status::Compliant,
            1 => Status::Violation,
            _ => Status::Incomplete,
        }).collect();
        let expected = ranks.into_iter().chain([u8::from(deny && warnings)]).max().unwrap();
        // Act
        let actual = statuses.into_iter().fold(Status::Compliant, Status::combine).exit_code(deny, warnings);
        // Assert
        prop_assert_eq!(actual, expected);
    }
}
