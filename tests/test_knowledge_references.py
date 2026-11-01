# SPDX-License-Identifier: Elastic-2.0
"""Small filesystem fixtures for the knowledge reference inventory."""

from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from pathlib import Path

import pytest

from tools.knowledge_references import inventory


@pytest.mark.parametrize(
    ("field", "expected"),
    [
        ("missing_markdown", ["gone.rs", "gone.rs"]),
        ("missing_sources", ["absent.rs"]),
        ("unreachable_pages", ["orphan.md"]),
        ("sources", ["absent.rs", "https://example.org/source"]),
    ],
)
def test_inventory_keeps_diagnostic_categories_independent(
    tmp_path: Path,
    field: Literal[
        "missing_markdown", "missing_sources", "unreachable_pages", "sources"
    ],
    expected: list[str],
) -> None:
    # Arrange
    (tmp_path / "index.md").write_text(
        "[entry](entry.md#section)\n[remote](https://example.org/x)\n"
    )
    (tmp_path / "entry.md").write_text(
        "---\ntype: Reference\nsources:\n"
        "  - id: missing\n    resource: absent.rs\n"
        "  - id: external\n    resource: https://example.org/source\n---\n"
        "[bad](gone.rs)\n[again](gone.rs)\n[ref][next]\n"
        "[next]: nested/child.md\n```\n[fake](ignored.rs)\n```\n"
    )
    (tmp_path / "nested").mkdir()
    (tmp_path / "nested/child.md").write_text("[root](/index.md)\n")
    (tmp_path / "orphan.md").write_text("orphan\n")
    # Act
    result = inventory(tmp_path)
    # Assert
    if field == "unreachable_pages":
        assert result[field] == expected
    else:
        key = "target" if field == "missing_markdown" else "resource"
        assert [row[key] for row in result[field]] == expected


def test_source_resource_does_not_make_a_concept_navigable(tmp_path: Path) -> None:
    # Arrange
    (tmp_path / "index.md").write_text(
        '---\nokf_version: "0.2"\nsources:\n'
        "  - id: source\n    resource: hidden.md\n---\n"
    )
    (tmp_path / "hidden.md").write_text("hidden\n")
    # Act
    result = inventory(tmp_path)
    # Assert
    assert result["unreachable_pages"] == ["hidden.md"]
    assert result["missing_sources"] == []


def test_missing_entry_point_is_rejected(tmp_path: Path) -> None:
    # Arrange
    (tmp_path / "concept.md").write_text("concept\n")
    with pytest.raises(FileNotFoundError, match="Knowledge entry point missing"):
        inventory(tmp_path)


def test_unused_reference_definition_does_not_create_navigation(tmp_path: Path) -> None:
    # Arrange
    (tmp_path / "index.md").write_text("[unused]: hidden.md\n")
    (tmp_path / "hidden.md").write_text("hidden\n")
    # Act
    result = inventory(tmp_path)
    # Assert
    assert result["unreachable_pages"] == ["hidden.md"]
    assert [row["target"] for row in result["markdown"]] == ["hidden.md"]


def test_used_references_and_footnotes_create_navigation(tmp_path: Path) -> None:
    # Arrange
    (tmp_path / "index.md").write_text(
        "[full][CHILD] [collapsed][] [shortcut] [^used]\n"
        "[child]: child.md\n[collapsed]: collapsed.md\n[shortcut]: shortcut.md\n"
        "[^used]: [nested][footnote-source]\n"
        "[footnote-source]: evidence.md\n"
        "[^unused]: [hidden](hidden.md)\n"
    )
    for name in ("child", "collapsed", "shortcut", "evidence", "hidden"):
        (tmp_path / f"{name}.md").write_text(name)
    # Act
    result = inventory(tmp_path)
    # Assert
    assert result["unreachable_pages"] == ["hidden.md"]
    assert result["missing_markdown"] == []
