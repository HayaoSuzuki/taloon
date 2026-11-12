# SPDX-License-Identifier: Elastic-2.0
"""Failures that the documentation gate must catch before changes merge."""

import os
import subprocess
import sys
from pathlib import Path

import pytest

from tools.check_docs import check_repository, main, validate_okf


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


def test_reserved_log_without_metadata_is_valid(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    (root / "log.md").write_text("# History\n")
    # Act
    errors = validate_okf(root)
    # Assert
    assert errors == []


def test_root_index_rejects_extra_metadata(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    (root / "index.md").write_text('---\nokf_version: "0.2"\ntype: Index\n---\n')
    # Act
    errors = validate_okf(root)
    # Assert
    assert len(errors) == 1
    assert "root index must declare only" in errors[0]


@pytest.mark.parametrize("broken", [False, True])
def test_documentation_command_reports_status(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    *,
    broken: bool,
) -> None:
    # Arrange
    bundle(tmp_path)
    monkeypatch.chdir(tmp_path)
    if broken:
        (tmp_path / "README.md").write_text("[missing](absent.md)")
    # Act
    status = main()
    output = capsys.readouterr()
    # Assert
    assert status == int(broken)
    assert bool(output.err) == broken
    assert bool(output.out) != broken


def test_documentation_module_reports_invalid_repository(tmp_path: Path) -> None:
    # Arrange
    bundle(tmp_path)
    (tmp_path / "README.md").write_text("[missing](absent.md)")
    # Act
    result = subprocess.run(
        [sys.executable, "-m", "tools.check_docs"],
        cwd=tmp_path,
        env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1])},
        capture_output=True,
        text=True,
        check=False,
    )
    # Assert
    assert result.returncode == 1
    assert "missing link target absent.md" in result.stderr
    assert result.stdout == ""


def test_reports_every_missing_bundle_reference(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    (root / "rule.md").write_text(
        (root / "rule.md").read_text()
        + "\n[First](first-missing.md)\n[Second](second-missing.md)\n"
    )
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert len(errors) == 2
    assert any("first-missing.md" in error for error in errors)
    assert any("second-missing.md" in error for error in errors)


def test_reports_every_unreachable_concept(tmp_path: Path) -> None:
    # Arrange
    root = bundle(tmp_path)
    for name in ("first-orphan.md", "second-orphan.md"):
        (root / name).write_text("---\ntype: Reference\n---\n# Unreachable\n")
    # Act
    errors = check_repository(tmp_path)
    # Assert
    assert len(errors) == 2
    assert any("first-orphan.md" in error for error in errors)
    assert any("second-orphan.md" in error for error in errors)


def test_command_prints_every_diagnostic(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    bundle(tmp_path)
    (tmp_path / "README.md").write_text("[First](first.md)\n[Second](second.md)")
    monkeypatch.chdir(tmp_path)
    # Act
    status = main()
    observed = capsys.readouterr()
    # Assert
    assert status == 1
    assert len(observed.err.splitlines()) == 2
    assert "first.md" in observed.err
    assert "second.md" in observed.err
    assert observed.out == ""
