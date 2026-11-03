//! Synthetic complete-text observations shared by public and acquisition tests.

#[derive(Clone, Copy, Debug)]
pub enum Variant {
    ApacheTerms,
    ApacheAppendix,
    MitNames,
    MitContact,
    MplLocator,
    BsdPlural,
    MitBare,
    MitMarkdown,
    MitYearless,
    MitDatedOwners,
    MitAttributionContacts,
}

impl Variant {
    pub fn text(self) -> String {
        match self {
            Self::MitBare => mit_preamble(""),
            Self::MitMarkdown => mit_preamble("# MIT License\nCopyright (c) 2026 Example Author"),
            Self::MitYearless => {
                mit_preamble("Copyright (c) Example Company, Inc. and affiliates.")
            }
            Self::MitDatedOwners => mit_preamble(
                "Copyright (c) 2020 Example Author, 2021 Other Writer, 2022 Third Person",
            ),
            Self::MitAttributionContacts => mit_preamble(
                "The MIT License (MIT)\nCopyright (c) 2020 - 2026 Example Author (Nickname) and contributors\nauthor.example\ncontact.example",
            ),
            Self::BsdPlural => spdx::license_id("BSD-3-Clause").unwrap().text().replace(
                "name of the copyright holder nor the names of its contributors",
                "name of the copyright holders, nor those of its contributors",
            ),
            Self::ApacheTerms => spdx::license_id("Apache-2.0")
                .unwrap()
                .text()
                .split_once("APPENDIX:")
                .unwrap()
                .0
                .trim_end()
                .to_owned(),
            Self::ApacheAppendix => spdx::license_id("Apache-2.0").unwrap().text().replace(
                "[yyyy] [name of copyright owner]",
                "[2026] [Example Author <author@example.org>]",
            ),
            Self::MitNames => spdx::license_id("MIT").unwrap().text().replace(
                "<year> <copyright holders>",
                "2026 Example Author, Other Writer.",
            ),
            Self::MitContact => spdx::license_id("MIT")
                .unwrap()
                .text()
                .replace("MIT License", "The MIT License (MIT)")
                .replace(
                    "<year> <copyright holders>",
                    "2026 Example Author <author@example.org> (https://author.example/)",
                ),
            Self::MplLocator => spdx::license_id("MPL-2.0").unwrap().text().replace(
                "https://mozilla.org/MPL/2.0/.",
                "http://mozilla.org/MPL/2.0/.",
            ),
        }
    }
}

fn mit_preamble(header: &str) -> String {
    let (_, body) = spdx::license_id("MIT")
        .unwrap()
        .text()
        .split_once("Permission is hereby granted")
        .unwrap();
    format!("{header}\n\nPermission is hereby granted{body}")
}
