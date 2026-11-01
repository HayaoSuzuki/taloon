# SPDX-License-Identifier: Elastic-2.0
"""Failures that the documentation gate must catch before changes merge."""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

import pytest

from tools.check_docs import check_repository, validate_okf


def bundle(tmp_path: Path) -> Path:
    root = tmp_path / "docs/development/knowledge"
    root.mkdir(parents=True)
    (root / "index.md").write_text(
        '---\nokf_version: "0.2"\n---\n# Knowledge\n\n[Rule](rule.md)\n'
    )
    (root / "rule.md").write_text(
        "---\ntype: Decision\nstatus: draft\ncustom: keep-me\nsources:\n"
        "  - id: evidence\n    resource: ../evidence.md\n---\n"
        "# Rule\n\nRead the evidence.[^evidence]\n\n"
        "[^evidence]: [Evidence](../evidence.md).\n"
    )
    (root.parent / "evidence.md").write_text("# Evidence\n")
    return root


def test_accepts_valid_bundle_and_preserves_unknown_metadata(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    original = (root / "rule.md").read_bytes()
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert errors == []
    assert (root / "rule.md").read_bytes() == original


@pytest.mark.parametrize(
    "content",
    [
        "# Missing frontmatter\n",
        "---\n# type: Decision\n---\n# A comment is not metadata\n",
        "---\ntype: []\n---\n# Invalid type\n",
        "---\ntype: ' '\n---\n# Empty type\n",
        "---\n- Decision\n---\n# Not a mapping\n",
        "---\ntype: [\n---\n# Malformed YAML\n",
        "---\ntype: Decision\n# Unterminated header\n",
    ],
)
def test_rejects_malformed_concepts(tmp_path: Path, content: str) -> None:
    # Arrange
    root = bundle(tmp_path)
    (root / "rule.md").write_text(content)
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert errors
    assert any("rule.md" in error for error in errors)


@pytest.mark.parametrize("name", ["nested/index.md", "log.md"])
def test_reserved_files_cannot_hide_concepts(tmp_path: Path, name: str) -> None:
    # Arrange
    root = bundle(tmp_path)
    page = root / name
    page.parent.mkdir(exist_ok=True)
    page.write_text("---\ntype: Decision\n---\n# Reserved\n")
    # Act
    errors = validate_okf(root)
    # Assert
    assert any(name in error for error in errors)


def test_requires_explicit_bundle_version(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    (root / "index.md").write_text("# Knowledge\n[Rule](rule.md)\n")
    # Act
    errors = validate_okf(root)
    # Assert
    assert any("index.md" in error for error in errors)


def test_missing_entry_point_fails_with_a_diagnostic(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    (root / "index.md").unlink()
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert any("index.md" in error for error in errors)


@pytest.mark.parametrize("missing", ["source", "navigation"])
def test_missing_source_or_navigation_is_reported(tmp_path: Path, missing: str) -> None:
    # Arrange
    root = bundle(tmp_path)
    expected = "evidence.md" if missing == "source" else "orphan.md"
    if missing == "source":
        (root.parent / "evidence.md").unlink()
    else:
        (root / "orphan.md").write_text((root / "rule.md").read_text())
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert any(expected in error for error in errors)


@pytest.mark.parametrize("entry", ["docs/user/index.md", "AGENTS.md"])
def test_checks_user_docs_and_agent_entry_links(tmp_path: Path, entry: str) -> None:
    # Arrange
    bundle(tmp_path)
    page = tmp_path / entry
    page.parent.mkdir(parents=True, exist_ok=True)
    page.write_text("[Rules](missing-rules.md)\n")
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert any("missing-rules.md" in error for error in errors)


def test_ignores_external_urls_and_links_in_examples(tmp_path: Path) -> None:
    # Arrange
    bundle(tmp_path)
    (tmp_path / "README.md").write_text(
        "[External](https://example.org/unfetched)\n"
        "```markdown\n[Example](not-a-real-file.md)\n```\n"
    )
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert errors == []


def test_source_ids_require_matching_citations(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    page = root / "rule.md"
    page.write_text(page.read_text().replace("id: evidence", "id: uncited"))
    # Act
    errors = validate_okf(root)
    # Assert
    assert any("uncited" in error for error in errors)


def test_duplicate_source_ids_fail(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    page = root / "rule.md"
    page.write_text(
        page.read_text().replace(
            "    resource: ../evidence.md\n---",
            "    resource: ../evidence.md\n  - id: evidence\n"
            "    resource: ../evidence.md\n---",
        )
    )
    # Act
    errors = validate_okf(root)
    # Assert
    assert any("duplicate" in error for error in errors)


def test_definition_alone_does_not_attribute_a_claim(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    page = root / "rule.md"
    page.write_text(
        page.read_text().replace("Read the evidence.[^evidence]", "No attribution.")
    )
    # Act
    errors = validate_okf(root)
    # Assert
    assert any("evidence" in error for error in errors)


@pytest.mark.parametrize(
    "replacement",
    [
        "sources: invalid",
        "sources: [invalid]",
        "sources: [{id: evidence}]",
        "sources: [{id: evidence, resource: []}]",
        "sources: [{id: ' ', resource: ../evidence.md}]",
    ],
)
def test_malformed_sources_produce_diagnostics(
    tmp_path: Path, replacement: str
) -> None:
    # Arrange
    root = bundle(tmp_path)
    page = root / "rule.md"
    page.write_text(
        page.read_text().replace(
            "sources:\n  - id: evidence\n    resource: ../evidence.md", replacement
        )
    )
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert any("rule.md" in error for error in errors)
