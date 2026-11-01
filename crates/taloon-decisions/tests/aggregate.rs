use proptest::prelude::*;
use rstest::rstest;
use taloon_decisions::status::{PackageOutcome, Status, aggregate};

type Package = PackageOutcome<(), (), (), (), (), ()>;

fn package(status: Status) -> Package {
    PackageOutcome {
        package: (),
        source: (),
        status,
        artifacts: (),
        diagnostics: (),
        paths: (),
        origin_note: (),
    }
}

#[rstest]
#[case::empty(vec![], false, Status::Compliant)]
#[case::empty_input_error(vec![], true, Status::Incomplete)]
#[case::compliant(vec![Status::Compliant], false, Status::Compliant)]
#[case::violation(vec![Status::Compliant, Status::Violation], false, Status::Violation)]
#[case::incomplete_first(vec![Status::Incomplete, Status::Violation], false, Status::Incomplete)]
#[case::incomplete_last(vec![Status::Violation, Status::Incomplete], false, Status::Incomplete)]
#[case::input_error(vec![Status::Violation], true, Status::Incomplete)]
fn aggregate_preserves_exact_precedence(
    #[case] statuses: Vec<Status>,
    #[case] input_error: bool,
    #[case] expected: Status,
) {
    // Arrange
    let packages: Vec<_> = statuses.into_iter().map(package).collect();
    // Act
    let actual = aggregate(&packages, input_error);
    // Assert
    assert_eq!(actual, expected);
}

proptest! {
    #[test]
    fn aggregate_matches_independent_maximum(
        ranks in prop::collection::vec(0_u8..=2, 0..100),
        input_error in any::<bool>(),
    ) {
        // Arrange
        let packages: Vec<_> = ranks.iter().map(|rank| package(match rank {
            0 => Status::Compliant,
            1 => Status::Violation,
            _ => Status::Incomplete,
        })).collect();
        let expected = ranks.into_iter().chain([2 * u8::from(input_error)]).max().unwrap();
        // Act
        let actual = aggregate(&packages, input_error).exit_code(false, false);
        // Assert
        prop_assert_eq!(actual, expected);
    }
}
