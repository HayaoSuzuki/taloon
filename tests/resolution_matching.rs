//! Indexed matching must preserve locked ambiguity and annotated path semantics.
use proptest::prelude::*;
use rstest::rstest;
use std::fmt::Write as _;
use taloon::{model::Code, resolution::Resolution};

const FORKS: [(&str, &str); 3] = [("1.0", "one"), ("2.0", "one"), ("1.0", "two")];

fn lock(selector: &str, unrelated: usize) -> String {
    let mut text = format!(
        "version=1\nrevision=5\n[[package]]\nname='root'\nsource={{virtual='.'}}\ndependencies=[{{name='DEP_pkg'{selector}}}]\n"
    );
    append_forks(&mut text);
    for n in 0..unrelated {
        writeln!(text, "[[package]]\nname='unrelated-{n}'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}").unwrap();
    }
    text
}

fn append_forks(text: &mut String) {
    for (version, host) in FORKS {
        writeln!(text, "[[package]]\nname='dep-pkg'\nversion='{version}'\nsource={{registry='https://{host}.test/simple'}}").unwrap();
    }
}
#[rstest]
#[case::version(",version='2.0'", "2.0", "one")]
#[case::source(",source={registry='https://two.test/simple'}", "1.0", "two")]
#[case::exact(
    ",version='1.0',source={registry='https://one.test/simple'}",
    "1.0",
    "one"
)]
#[case::specifier(",specifier='>=2'", "2.0", "one")]
fn selectors_preserve_the_unique_locked_identity(
    #[case] selector: &str,
    #[case] version: &str,
    #[case] host: &str,
) {
    // Arrange
    let text = lock(selector, 32);
    let source = format!("https://{host}.test/simple");
    // Act
    let resolution = Resolution::from_uv_lock(&text).unwrap();
    // Assert: origin exists only for the selected identity; all other targets remain.
    assert_eq!(resolution.targets().len(), 35);
    let reached: Vec<_> = resolution
        .targets()
        .iter()
        .filter(|t| !resolution.paths(t.package()).is_empty())
        .collect();
    assert_eq!(reached.len(), 1);
    assert_eq!(reached[0].package().version(), version);
    assert_eq!(
        reached[0]
            .package()
            .source()
            .location()
            .trim_end_matches('/'),
        source
    );
}
#[rstest]
#[case::ambiguous("", Code::LockFormat)]
#[case::no_match(",version='3.0'", Code::LockFormat)]
#[case::invalid_version(",version='invalid'", Code::Configuration)]
#[case::invalid_constraint(",specifier='invalid'", Code::LockFormat)]
#[case::unmatched_invalid_constraint(",version='3.0',specifier='invalid'", Code::LockFormat)]
fn invalid_selectors_preserve_failure_codes(#[case] selector: &str, #[case] expected: Code) {
    // Arrange
    let text = lock(selector, 16);
    // Act
    let result = Resolution::from_uv_lock(&text);
    // Assert
    assert_eq!(result.err().unwrap().code, expected);
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn unrelated_records_cannot_change_exact_origin_selection(unrelated in 0_usize..128, choose_second in any::<bool>()) {
        // Arrange: source choice has one known version independently of resolution.
        let (version, host) = if choose_second { ("1.0", "two") } else { ("2.0", "one") };
        let selector = format!(",version='{version}',source={{registry='https://{host}.test/simple'}}");
        let text = lock(&selector, unrelated);
        // Act
        let resolution = Resolution::from_uv_lock(&text).unwrap();
        // Assert
        let reached: Vec<_> = resolution.targets().iter().filter(|t| !resolution.paths(t.package()).is_empty()).collect();
        prop_assert_eq!(resolution.targets().len(), unrelated + 3);
        prop_assert_eq!(reached.len(), 1);
        prop_assert_eq!(reached[0].package().version(), version);
        let expected_host = format!("{host}.test");
        prop_assert!(reached[0].package().source().location().contains(&expected_host));
    }
}

#[derive(serde::Deserialize)]
struct ForkCase {
    selector: ForkSelector,
    decision: ForkDecision,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
struct ForkSelector {
    version: usize,
    source: usize,
    constraint: usize,
    manifest: bool,
    marked: bool,
}

#[derive(serde::Deserialize)]
struct ForkDecision {
    accepted: bool,
    selected: usize,
    marker: String,
}

fn selected_lock(selector: &ForkSelector) -> String {
    let version = ["", ",version='1.0'", ",version='2.0'", ",version='3.0'"][selector.version];
    let source = [
        "",
        ",source={registry='https://one.test/simple'}",
        ",source={registry='https://two.test/simple'}",
        ",source={registry='https://absent.test/simple'}",
    ][selector.source];
    let constraint = ["", ",specifier='>=2'", ",specifier='<2'"][selector.constraint];
    let marker = if selector.marked {
        ",marker=\"sys_platform == 'linux'\""
    } else {
        ""
    };
    let fields = format!("{version}{source}{constraint}{marker}");
    if selector.manifest {
        let mut text = format!(
            "version=1\nrevision=5\n[manifest.dependency-groups]\ndev=[{{name='DEP_pkg'{fields}}}]\n"
        );
        append_forks(&mut text);
        text
    } else {
        lock(&fields, 0)
    }
}

#[rstest]
fn quint_replays_locked_forks(
    #[values(0, 1, 2, 3)] version: usize,
    #[values(0, 1, 2, 3)] source: usize,
    #[values(0, 1, 2)] constraint: usize,
    #[values(false, true)] manifest: bool,
    #[values(false, true)] marked: bool,
) {
    // Arrange
    let index = (((version * 4 + source) * 3 + constraint) * 2 + usize::from(manifest)) * 2
        + usize::from(marked);
    let row = include_str!("fixtures/oracle/locked-forks.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: ForkCase = serde_json::from_str(row).unwrap();
    assert_eq!(
        case.selector,
        ForkSelector {
            version,
            source,
            constraint,
            manifest,
            marked
        }
    );
    let lock = selected_lock(&case.selector);

    // Act
    let result = Resolution::from_uv_lock(&lock);

    // Assert
    let expected = case.decision;
    if !expected.accepted {
        assert_eq!(result.err().unwrap().code, Code::LockFormat);
        return;
    }
    let resolution = result.unwrap();
    let mut actual_forks: Vec<_> = resolution
        .targets()
        .iter()
        .map(|target| {
            let package = target.package();
            (
                package.version(),
                package.source().location().trim_end_matches('/').to_owned(),
            )
        })
        .collect();
    actual_forks.sort_unstable();
    let mut expected_forks =
        FORKS.map(|(version, host)| (version.to_owned(), format!("https://{host}.test/simple")));
    expected_forks.sort_unstable();
    assert_eq!(actual_forks, expected_forks);
    let origins: Vec<_> = resolution
        .targets()
        .iter()
        .flat_map(|target| {
            resolution
                .paths(target.package())
                .into_iter()
                .map(|path| (target.package(), path))
        })
        .collect();
    assert_eq!(origins.len(), usize::from(expected.selected != 0));
    if let [(package, path)] = origins.as_slice() {
        let (version, host) = FORKS[expected.selected - 1];
        assert_eq!(package.version(), version);
        assert_eq!(
            package.source().location().trim_end_matches('/'),
            format!("https://{host}.test/simple")
        );
        let entry = if manifest {
            "workspace [group:dev]"
        } else {
            "root"
        };
        assert_eq!(path.entry, entry);
        assert_eq!(path.steps.len(), 1);
        let marker = Some(expected.marker.as_str()).filter(|value| !value.is_empty());
        assert_eq!(path.steps[0].marker.as_deref(), marker);
    }
}
