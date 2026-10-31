use proptest::prelude::*;
use std::fmt::Write as _;
use taloon::{model::SourceKind, resolution::Resolution};

fn wheel() -> String {
    format!(
        "{{url='https://files.example.test/a.whl',hash='sha256:{}'}}",
        "11".repeat(32)
    )
}
fn sdist() -> String {
    format!(
        "{{url='https://files.example.test/a.tar.gz',hash='sha256:{}'}}",
        "22".repeat(32)
    )
}
fn fixture() -> String {
    format!(
        r#"version=1
revision=5
[[package]]
name='root'
version='0.1.0'
source={{virtual='.'}}
dependencies=[{{name='dep',marker="sys_platform == 'linux'"}},{{name='branch',version='1.0',marker="sys_platform == 'linux'"}}]
[package.dev-dependencies]
dev=[{{name='branch',version='2.0'}},{{name='dup',source={{registry='https://one.test/simple'}}}},{{name='externaldir'}},{{name='gitpkg'}}]
[package.optional-dependencies]
feature=[{{name='dup',source={{registry='https://two.test/simple'}}}}]
[[package]]
name='dep'
version='1.0'
source={{registry='https://pypi.org/simple'}}
dependencies=[{{name='leaf'}}]
wheels=[{wheel}]
sdist={sdist}
[[package]]
name='leaf'
version='1.0'
source={{registry='https://pypi.org/simple'}}
dependencies=[{{name='dep'}}]
wheels=[{wheel}]
[[package]]
name='branch'
version='1.0'
source={{registry='https://pypi.org/simple'}}
dependencies=[{{name='leaf'}}]
wheels=[{wheel}]
sdist={sdist}
[[package]]
name='branch'
version='2.0'
source={{registry='https://pypi.org/simple'}}
dependencies=[{{name='leaf'}}]
wheels=[{wheel}]
sdist={sdist}
[[package]]
name='dup'
version='1.0'
source={{registry='https://one.test/simple'}}
dependencies=[{{name='leaf'}}]
wheels=[{wheel}]
[[package]]
name='dup'
version='1.0'
source={{registry='https://two.test/simple'}}
dependencies=[{{name='leaf'}}]
wheels=[{wheel}]
[[package]]
name='externaldir'
version='1.0'
source={{directory='../external'}}
[[package]]
name='gitpkg'
version='1.0'
source={{git='https://example.test/source#abcdef'}}
"#,
        wheel = wheel(),
        sdist = sdist()
    )
}

#[test]
fn all_locked_branches_and_artifacts_are_targets() {
    let r = Resolution::from_uv_lock(&fixture()).unwrap();
    assert_eq!(r.targets().len(), 8);
    assert_eq!(
        r.targets()
            .iter()
            .map(|t| t.requests().len())
            .sum::<usize>(),
        9
    );
    assert_eq!(
        r.targets()
            .iter()
            .filter(|t| !t.issues().is_empty())
            .count(),
        2
    );
    assert!(!r.targets().iter().any(|t| t.package().name() == "root"));
}

#[test]
fn paths_preserve_entrypoint_and_conditions() {
    let r = Resolution::from_uv_lock(&fixture()).unwrap();
    let leaf = r
        .targets()
        .iter()
        .find(|t| t.package().name() == "leaf")
        .unwrap();
    let paths = r.paths(leaf.package());
    assert_eq!(paths.len(), 3);
    assert!(paths.iter().all(|p| p.steps.len() == 2));
    assert!(paths.iter().any(|p| p.entry.contains("group:dev")));
    assert!(paths.iter().any(|p| p.entry.contains("extra:feature")));
    assert!(paths.iter().any(|p| {
        p.steps
            .iter()
            .any(|s| s.marker.as_ref().is_some_and(|m| m.contains("linux")))
    }));
}

#[test]
fn explicit_members_do_not_exclude_registry_packages_sharing_a_name() {
    let mut text = fixture().replace("source={virtual='.'}", "source={editable='packages/root'}");
    text = text.replacen(
        "[[package]]",
        "[manifest]\nmembers=['root']\n[[package]]",
        1,
    );
    write!(text,"\n[[package]]\nname='root'\nversion='2.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{}]\n",wheel()).unwrap();
    let r = Resolution::from_uv_lock(&text).unwrap();
    assert_eq!(r.targets().len(), 9);
    assert!(r.targets().iter().any(
        |t| t.package().name() == "root" && t.package().source().kind() == SourceKind::Registry
    ));
}

#[test]
fn invalid_or_ambiguous_edges_cannot_invent_associations() {
    assert!(
        Resolution::from_uv_lock(&fixture().replacen(
            "dependencies=[{name='leaf'}]",
            "dependencies=[{name='missing'}]",
            1
        ))
        .is_err()
    );
    assert!(
        Resolution::from_uv_lock(
            &fixture().replace("name='branch',version='1.0'", "name='branch'")
        )
        .is_err()
    );
    assert!(Resolution::from_uv_lock(&fixture().replace("revision=5", "revision=6")).is_err());
    assert!(Resolution::from_uv_lock(&fixture().replace("version=1\n", "version=2\n")).is_err());
}

#[test]
fn repository_documentation_lock_is_parseable() {
    let r = Resolution::from_uv_lock(include_str!("../uv.lock")).unwrap();
    assert!(!r.targets().is_empty());
    assert!(r.targets().iter().all(|t| !t.requests().is_empty()));
}

#[test]
fn workspace_only_groups_have_dependency_origins() {
    let lock = format!(
        "version=1\nrevision=5\n[manifest.dependency-groups]\ndev=[{{name='leaf',specifier='>=1.0',marker=\"sys_platform == 'linux'\"}}]\n[[package]]\nname='leaf'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{}]",
        wheel()
    );
    let r = Resolution::from_uv_lock(&lock).unwrap();
    let paths = r.paths(r.targets()[0].package());
    assert_eq!(paths.len(), 1);
    assert!(paths[0].entry.contains("group:dev"));
    assert!(paths[0].steps[0].marker.as_ref().unwrap().contains("linux"));
}

#[test]
fn package_resolution_markers_survive_path_reporting() {
    let lock = fixture().replace(
        "name='leaf'\nversion='1.0'",
        "name='leaf'\nversion='1.0'\nresolution-markers=[\"python_full_version >= '3.12'\"]",
    );
    let r = Resolution::from_uv_lock(&lock).unwrap();
    let target = r
        .targets()
        .iter()
        .find(|t| t.package().name() == "leaf")
        .unwrap();
    let paths = serde_json::to_value(r.paths(target.package())).unwrap();
    assert_eq!(
        paths[0]["steps"][1]["package_markers"][0],
        "python_full_version >= '3.12'"
    );
}

#[test]
fn local_archives_and_missing_artifacts_remain_explicit_targets() {
    let lock = "version=1\nrevision=5\n[[package]]\nname='local'\nversion='1.0'\nsource={path='dist/local-1.0.WHL'}\n[[package]]\nname='empty'\nversion='1.0'\nsource={registry='https://pypi.org/simple'}";
    let r = Resolution::from_uv_lock(lock).unwrap();
    assert_eq!(r.targets().len(), 2);
    assert_eq!(
        r.targets()[0].issues()[0].code,
        taloon::model::Code::MissingArtifact
    );
    assert_eq!(
        r.targets()[1].requests()[0].format(),
        taloon::model::ArtifactFormat::Wheel
    );
    assert_eq!(r.targets()[1].requests()[0].hash(), None);
}

#[test]
fn a_package_reached_with_an_extra_is_not_collapsed_into_its_plain_visit() {
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='root'\nversion='1.0'\nsource={{virtual='.'}}\ndependencies=[{{name='a'}},{{name='a',extra=['feature']}}]\n[[package]]\nname='a'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{}]\n[package.optional-dependencies]\nfeature=[{{name='b'}}]\n[[package]]\nname='b'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{}]",
        wheel(),
        wheel()
    );
    let r = Resolution::from_uv_lock(&lock).unwrap();
    let b = r
        .targets()
        .iter()
        .find(|t| t.package().name() == "b")
        .unwrap();
    let paths = r.paths(b.package());
    assert_eq!(paths.len(), 1);
    assert!(paths[0].steps[0].via.iter().any(|v| v == "extra:feature"));
}

#[test]
fn a_cycle_to_a_workspace_member_does_not_activate_its_development_group() {
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='root'\nversion='1.0'\nsource={{virtual='.'}}\ndependencies=[{{name='a'}}]\n[package.dev-dependencies]\ndev=[{{name='b'}}]\n[[package]]\nname='a'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\ndependencies=[{{name='root'}}]\nwheels=[{}]\n[[package]]\nname='b'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{}]",
        wheel(),
        wheel()
    );
    let r = Resolution::from_uv_lock(&lock).unwrap();
    let b = r
        .targets()
        .iter()
        .find(|t| t.package().name() == "b")
        .unwrap();
    let paths = r.paths(b.package());
    assert_eq!(paths.len(), 1);
    assert!(paths[0].entry.contains("group:dev"));
}

proptest! {
    #![proptest_config(ProptestConfig {cases:256,failure_persistence:Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct("tests/resolution.proptest-regressions"))),..ProptestConfig::default()})]
    #[test]
    fn package_reordering_preserves_complete_target_set(order in prop::collection::vec(any::<u16>(),1..12)) {
        let mut blocks=order.iter().enumerate().map(|(i,rank)|(*rank,format!("[[package]]\nname='pkg{i}'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{}]\n",wheel()))).collect::<Vec<_>>();
        blocks.sort_by_key(|(rank,_)|*rank);
        let lock=format!("version=1\nrevision=5\n{}",blocks.into_iter().map(|(_,s)|s).collect::<String>());
        let r=Resolution::from_uv_lock(&lock).unwrap();
        let mut expected=(0..order.len()).map(|i|format!("pkg{i}")).collect::<Vec<_>>();expected.sort();
        prop_assert_eq!(r.targets().iter().map(|t|t.package().name().to_owned()).collect::<Vec<_>>(),expected);
    }
}
