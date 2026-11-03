mod common;
use common::{metadata, request, wheel};
use proptest::prelude::*;
use rstest::rstest;
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{ArtifactFormat, Code, ContentHash, EvidenceRequirement},
};
fn legacy(value: &str) -> Evidence {
    let meta = metadata(&format!("License: {}", value.replace('\n', "\n ")));
    let bytes = wheel(&meta);
    let request = request(&bytes, ArtifactFormat::Wheel);
    let hash = request.hash().unwrap();
    Evidence::from_sidecar(
        &request,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap()
}
#[rstest]
#[case::mit_title("MIT License", "MIT")]
#[case::apache_title("Apache License 2.0", "Apache-2.0")]
#[case::apache_punctuation("Apache License, Version 2.0", "Apache-2.0")]
#[case::apache_short("Apache 2.0", "Apache-2.0")]
#[case::bsd2("BSD 2-Clause License", "BSD-2-Clause")]
#[case::bsd3("BSD 3-Clause License", "BSD-3-Clause")]
#[case::bsd2_reverse("2-Clause BSD License", "BSD-2-Clause")]
#[case::bsd3_reverse("3-Clause BSD License", "BSD-3-Clause")]
#[case::mit_wrapped("The MIT License (MIT)", "MIT")]
#[case::case_insensitive("mit license", "MIT")]
#[case::mpl11_short("MPL 1.1", "MPL-1.1")]
#[case::mpl11_title("Mozilla Public License Version 1.1", "MPL-1.1")]
#[case::mpl20_short("MPL 2.0", "MPL-2.0")]
#[case::mpl20_title("Mozilla Public License, Version 2.0", "MPL-2.0")]
#[case::mpl_case_and_space(" \tMpL 2.0 \t", "MPL-2.0")]
#[case::mit_article("The MIT License", "MIT")]
#[case::apache_license_suffix("Apache 2.0 License", "Apache-2.0")]
#[case::apache_software_version("Apache Software License, Version 2.0", "Apache-2.0")]
#[case::apache_software("Apache Software License 2.0", "Apache-2.0")]
#[case::apache_software_article("The Apache Software License, Version 2.0", "Apache-2.0")]
#[case::apache_parenthesized("Apache License (v2.0)", "Apache-2.0")]
#[case::apache_v_prefix("Apache License v2.0", "Apache-2.0")]
#[case::apache_version("Apache License Version 2.0", "Apache-2.0")]
#[case::apache_dash_version("Apache Software License - Version 2.0", "Apache-2.0")]
#[case::apache_article("The Apache License, Version 2.0", "Apache-2.0")]
#[case::apache_identifier_suffix("Apache-2.0 License", "Apache-2.0")]
#[case::bsd2_spaces("BSD 2 Clause", "BSD-2-Clause")]
#[case::bsd2_article("The BSD 2-Clause License", "BSD-2-Clause")]
#[case::bsd2_article_reverse("The 2-Clause BSD License", "BSD-2-Clause")]
#[case::bsd3_spaces("BSD 3 Clause", "BSD-3-Clause")]
#[case::bsd3_article("The BSD 3-Clause License", "BSD-3-Clause")]
#[case::cddl10_short("CDDL 1.0", "CDDL-1.0")]
#[case::cddl10_v_prefix("Common Development and Distribution License (CDDL) v1.0", "CDDL-1.0")]
#[case::cddl11_short("CDDL 1.1", "CDDL-1.1")]
#[case::cddl11_v_prefix("Common Development and Distribution License (CDDL) v1.1", "CDDL-1.1")]
#[case::epl10_short("EPL 1.0", "EPL-1.0")]
#[case::epl10_parenthesized("Eclipse Public License (EPL) 1.0", "EPL-1.0")]
#[case::epl10_v_prefix("Eclipse Public License v1.0", "EPL-1.0")]
#[case::epl10_version("Eclipse Public License, Version 1.0", "EPL-1.0")]
#[case::epl20_short("EPL 2.0", "EPL-2.0")]
#[case::epl20_v_prefix("Eclipse Public License v2.0", "EPL-2.0")]
#[case::epl20_version("Eclipse Public License, Version 2.0", "EPL-2.0")]
#[case::cc0_version("CC0 1.0 Universal", "CC0-1.0")]
#[case::gpl2_explicit_later(
    "GNU General Public License (GPL) version 2, or any later version",
    "GPL-2.0-or-later"
)]
#[case::gpl20_explicit_later(
    "GNU General Public License (GPL) version 2.0, or any later version",
    "GPL-2.0-or-later"
)]
#[case::gpl2_v_later("GNU General Public License v2 or later", "GPL-2.0-or-later")]
#[case::lgpl21_explicit_later(
    "GNU Lesser General Public License (LGPL), version 2.1 or later",
    "LGPL-2.1-or-later"
)]
#[case::lgpl21_v_later("LGPL, v2.1 or later", "LGPL-2.1-or-later")]
#[case::lgpl3_explicit_later(
    "GNU Lesser General Public License (LGPL), version 3 or later",
    "LGPL-3.0-or-later"
)]
#[case::lgpl30_explicit_later(
    "GNU Lesser General Public License (LGPL), version 3.0 or later",
    "LGPL-3.0-or-later"
)]
fn precise_legacy_names_are_interpreted_without_archive_proof(
    #[case] value: &str,
    #[case] expected: &str,
) {
    // Arrange
    let evidence = legacy(value);
    // Act
    let expression = evidence.license().unwrap();
    // Assert
    assert_eq!(expression.text(), expected);
    assert!(evidence.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!evidence.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert!(!evidence.needs_license_files());
    assert_eq!(
        evidence.summary().original_license,
        vec![value.trim().to_owned()]
    );
    assert_eq!(
        evidence.summary().license_origin,
        Some(taloon::evidence::LicenseOrigin::LegacyDeclaration)
    );
}
#[rstest]
#[case::bsd("BSD")]
#[case::bsd_generic("BSD License")]
#[case::apache_unspecified("Apache Software License")]
#[case::added_terms("MIT License, non-commercial use only")]
#[case::missing_conjunction("MIT License Apache License 2.0")]
#[case::unknown("UNKNOWN")]
#[case::locator("https://example.test/MIT-License")]
#[case::mpl_unspecified("MPL")]
#[case::mpl_unknown_version("MPL 9.0")]
#[case::mpl_conditions("MPL 2.0 with additional restrictions")]
#[case::mpl_conjunction("MPL 2.0 OR MIT")]
#[case::mpl_prefix("Modified MPL 2.0")]
#[case::mpl_inner_spacing("MPL  2.0")]
#[case::dual_unspecified("Dual License")]
#[case::gpl_version_only("GPL v2")]
#[case::lgpl_version_only("LGPL 2.1")]
#[case::cddl_unspecified("CDDL")]
#[case::epl_unspecified("EPL")]
fn ambiguous_or_modified_names_remain_unknown(#[case] value: &str) {
    // Arrange
    let evidence = legacy(value);
    // Act
    let error = evidence.license().unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnknownLicense);
}
proptest! {
    #[test]
    fn extra_words_cannot_be_discarded_as_name_decoration(extra in "[a-z]{1,24}") {
        // Arrange
        let evidence=legacy(&format!("MIT License {extra}"));
        // Act
        let result=evidence.license();
        // Assert
        prop_assert!(result.is_err());
    }
    #[test]
    fn official_name_case_does_not_change_identity(mask in any::<u32>()) {
        // Arrange
        let name: String="MIT License".chars().enumerate().map(|(i,c)|if mask & (1 << i)==0 {c.to_ascii_lowercase()}else{c.to_ascii_uppercase()}).collect();
        let evidence=legacy(&name);
        // Act
        let result=evidence.license().unwrap();
        // Assert
        prop_assert_eq!(result.text(),"MIT");
    }
}
#[rstest]
#[case::or("MIT OR Apache-2.0")]
#[case::and("BSD-2-Clause AND MIT")]
#[case::custom("LicenseRef-Reviewed")]
fn valid_legacy_expressions_keep_their_semantics(#[case] text: &str) {
    // Arrange
    let evidence = legacy(text);
    // Act
    let expression = evidence.license().unwrap();
    // Assert
    assert_eq!(expression.text(), text);
}

#[rstest]
#[case::mit("MIT")]
#[case::bsd2("BSD-2-Clause")]
#[case::bsd3("BSD-3-Clause")]
#[case::isc("ISC")]
#[case::apache("Apache-2.0")]
fn complete_legacy_text_is_interpreted_from_metadata(#[case] id: &str) {
    // Arrange
    let text = spdx::license_id(id).unwrap().text();
    let evidence = legacy(text);
    // Act
    let expression = evidence.license().unwrap();
    // Assert
    assert_eq!(expression.text(), id);
    assert!(evidence.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!evidence.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert_eq!(evidence.summary().original_expression, Vec::<String>::new());
    assert_ne!(evidence.summary().original_license[0], "");
}
#[rstest]
#[case::authoritative("License-Expression: MIT\nLicense: Apache 2.0", Some("MIT"))]
#[case::invalid_expression("License-Expression: INVALID\nLicense: MIT License", None)]
#[case::expression_is_not_a_name("License-Expression: MIT License\nLicense: MIT", None)]
#[case::repeated_legacy("License: MIT License\nLicense: MIT License", None)]
#[case::repeated_expression(
    "License-Expression: MIT\nLicense-Expression: MIT\nLicense: MIT License",
    None
)]
#[case::mpl_authoritative("License-Expression: MIT\nLicense: MPL 2.0", Some("MIT"))]
#[case::mpl_invalid_expression("License-Expression: INVALID\nLicense: MPL 2.0", None)]
#[case::mpl_expression_is_not_an_alias("License-Expression: MPL 2.0\nLicense: MPL-2.0", None)]
#[case::mpl_repeated_legacy("License: MPL 2.0\nLicense: MPL 2.0", None)]
#[case::mpl_repeated_expression(
    "License-Expression: MIT\nLicense-Expression: MIT\nLicense: MPL 2.0",
    None
)]
fn field_priority_and_cardinality_remain_strict(
    #[case] fields: &str,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let meta = metadata(fields);
    let bytes = wheel(&meta);
    let request = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Act
    let result = evidence.license();
    // Assert
    assert_eq!(
        result
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected
    );
}

#[rstest]
#[case::allow_permitted("MPL 2.0", "allow=['MPL-2.0']", 0)]
#[case::allow_unlisted("MPL 2.0", "allow=['MIT']", 1)]
#[case::deny_forbidden("MPL 2.0", "deny=['MPL-2.0']", 1)]
#[case::deny_other("MPL 2.0", "deny=['MIT']", 0)]
#[case::later_allow_permitted(
    "GNU General Public License v2 or later",
    "allow=['GPL-2.0-or-later']",
    0
)]
#[case::later_allow_unlisted("GNU General Public License v2 or later", "allow=['MIT']", 1)]
#[case::later_deny_forbidden(
    "GNU General Public License v2 or later",
    "deny=['GPL-2.0-or-later']",
    1
)]
#[case::later_deny_other("GNU General Public License v2 or later", "deny=['MIT']", 0)]
fn resolved_alias_obeys_policy(#[case] value: &str, #[case] policy: &str, #[case] exit_code: u8) {
    // Arrange
    let bytes = wheel(&metadata(&format!("License: {value}")));
    let hash = ContentHash::digest(&bytes);
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{{url='https://files.example.test/example.whl',hash='{hash}'}}]"
    );
    let inspection = taloon::inspection::Inspection::from_documents(
        &lock,
        &format!("[tool.taloon]\nschema-version=1\n{policy}"),
        taloon::model::EvaluationDate::parse("2026-11-03").unwrap(),
    )
    .unwrap();
    let request = &inspection.requests()[0];
    let observations = vec![taloon::inspection::AcquisitionResult::observed(
        request.id(),
        Evidence::from_archive(request, &bytes).unwrap(),
    )];
    // Act
    let report = inspection.evaluate(observations);
    // Assert
    assert_eq!(report.exit_code(false), exit_code);
}

proptest! {
    #[test]
    fn reviewed_alias_case_and_outer_whitespace_preserve_identity(
        (name, expected) in prop::sample::select(vec![
            ("MPL 1.1", "MPL-1.1"),
            ("Mozilla Public License Version 1.1", "MPL-1.1"),
            ("MPL 2.0", "MPL-2.0"),
            ("Mozilla Public License, Version 2.0", "MPL-2.0"),
            ("The MIT License", "MIT"),
            ("Apache License (v2.0)", "Apache-2.0"),
            ("BSD 2 Clause", "BSD-2-Clause"),
            ("The BSD 3-Clause License", "BSD-3-Clause"),
            ("CDDL 1.0", "CDDL-1.0"),
            ("Common Development and Distribution License (CDDL) v1.1", "CDDL-1.1"),
            ("EPL 1.0", "EPL-1.0"),
            ("Eclipse Public License, Version 2.0", "EPL-2.0"),
            ("CC0 1.0 Universal", "CC0-1.0"),
            ("GNU General Public License (GPL) version 2.0, or any later version", "GPL-2.0-or-later"),
            ("LGPL, v2.1 or later", "LGPL-2.1-or-later"),
            ("GNU Lesser General Public License (LGPL), version 3.0 or later", "LGPL-3.0-or-later"),
        ]),
        mask in any::<u128>(), before in "[ \t]{0,8}", after in "[ \t]{0,8}",
    ) {
        // Arrange
        let varied: String = name.chars().enumerate().map(|(i, c)| {
            if mask & (1 << i) == 0 { c.to_ascii_lowercase() } else { c.to_ascii_uppercase() }
        }).collect();
        let evidence = legacy(&format!("{before}{varied}{after}"));
        // Act
        let expression = evidence.license().unwrap();
        // Assert
        prop_assert_eq!(expression.text(), expected);
    }

    #[test]
    fn reviewed_alias_extra_terms_remain_unknown(
        name in prop::sample::select(vec![
            "MPL 1.1", "Mozilla Public License Version 1.1", "MPL 2.0", "Mozilla Public License, Version 2.0",
            "The MIT License", "Apache License (v2.0)", "BSD 2 Clause", "The BSD 3-Clause License",
            "CDDL 1.0", "CDDL 1.1", "EPL 1.0", "EPL 2.0", "CC0 1.0 Universal",
            "GNU General Public License (GPL) version 2.0, or any later version",
            "LGPL, v2.1 or later", "GNU Lesser General Public License (LGPL), version 3.0 or later",
        ]),
        extra in "[a-z]{1,24}", prefix in any::<bool>(),
    ) {
        // Arrange
        let value = if prefix { format!("{extra} {name}") } else { format!("{name} {extra}") };
        let evidence = legacy(&value);
        // Act
        let result = evidence.license();
        // Assert
        prop_assert!(result.is_err());
    }
}
#[test]
fn cached_raw_legacy_fields_are_reinterpreted_without_new_observations() {
    // Arrange
    let evidence = legacy(spdx::license_id("MIT").unwrap().text());
    let meta = metadata(&format!(
        "License: {}",
        spdx::license_id("MIT").unwrap().text().replace('\n', "\n ")
    ));
    let bytes = wheel(&meta);
    let request = request(&bytes, ArtifactFormat::Wheel);
    let encoded = serde_json::to_vec(&evidence.to_record()).unwrap();
    let decoded = serde_json::from_slice(&encoded).unwrap();
    // Act
    let restored = Evidence::from_record(&request, decoded).unwrap();
    // Assert
    assert_eq!(restored.license().unwrap().text(), "MIT");
    assert!(!restored.needs_license_files());
}
proptest! {
    #[test]
    fn added_terms_cannot_be_dropped_from_complete_text(extra in "[a-z]{1,24}") {
        // Arrange
        let text=format!("{}\nAdditional terms: {extra}",spdx::license_id("MIT").unwrap().text());
        let evidence=legacy(&text);
        // Act
        let result=evidence.license();
        // Assert
        prop_assert!(result.is_err());
    }
}

#[rstest]
#[case::sidecar_lf_restriction(false, "\n", false)]
#[case::sidecar_crlf_restriction(false, "\r\n", false)]
#[case::archive_lf_restriction(true, "\n", false)]
#[case::archive_crlf_restriction(true, "\r\n", false)]
#[case::sidecar_lf_notices(false, "\n", true)]
#[case::sidecar_crlf_notices(false, "\r\n", true)]
#[case::archive_lf_notices(true, "\n", true)]
#[case::archive_crlf_notices(true, "\r\n", true)]
fn legacy_text_keeps_original_notice_boundaries(
    #[case] archive: bool,
    #[case] newline: &str,
    #[case] valid: bool,
) {
    // Arrange
    let canonical = spdx::license_id("MIT").unwrap().text();
    let (_, body) = canonical
        .split_once("Permission is hereby granted")
        .unwrap();
    let second = if valid {
        "Copyright (c) 2025 Sample"
    } else {
        "Redistribution Requires Consent."
    };
    let text =
        format!("Copyright (c) 2026 Example\n{second}\n\nPermission is hereby granted{body}");
    let meta = metadata(&format!("License: {}", text.replace('\n', "\n ")));
    let meta = String::from_utf8(meta)
        .unwrap()
        .replace('\n', newline)
        .into_bytes();
    let bytes = wheel(&meta);
    let request = request(&bytes, ArtifactFormat::Wheel);
    let evidence = if archive {
        Evidence::from_archive(&request, &bytes).unwrap()
    } else {
        let hash = request.hash().unwrap();
        Evidence::from_sidecar(
            &request,
            SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
            &meta,
        )
        .unwrap()
    };
    // Act
    let interpreted = evidence.license();
    // Assert
    assert_eq!(
        interpreted
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        if valid { Some("MIT") } else { None }
    );
}

proptest! {
    #[test]
    fn raw_legacy_spans_survive_unicode_headers_and_cache_roundtrips(prefix in "[éΩ漢]{0,40}") {
        // Arrange
        let text=spdx::license_id("MIT").unwrap().text();
        let meta=metadata(&format!("X-Note: {prefix}\nLicense: {}",text.replace('\n',"\n ")));
        let bytes=wheel(&meta);
        let request=request(&bytes,ArtifactFormat::Wheel);
        let original=Evidence::from_archive(&request,&bytes).unwrap();
        let encoded=serde_json::to_vec(&original.to_record()).unwrap();
        let decoded=serde_json::from_slice(&encoded).unwrap();
        // Act
        let restored=Evidence::from_record(&request,decoded).unwrap();
        let expression=restored.license().unwrap();
        // Assert
        prop_assert_eq!(expression.text(),"MIT");
    }
}
