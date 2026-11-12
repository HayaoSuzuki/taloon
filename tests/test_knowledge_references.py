# SPDX-License-Identifier: Elastic-2.0
"""Small filesystem fixtures for the knowledge reference inventory."""

import json
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Literal

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

from tools.knowledge_references import inventory, local_target, main, markdown_targets


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


def test_fragment_only_target_resolves_to_current_page(tmp_path: Path) -> None:
    # Arrange
    page = tmp_path / "index.md"
    # Act
    observed = local_target(page, "#section", tmp_path)
    # Assert
    assert observed == page


def test_indented_footnote_continuation_contributes_navigation(tmp_path: Path) -> None:
    # Arrange
    (tmp_path / "index.md").write_text(
        "Read[^evidence]\n\n[^evidence]: Evidence\n\n    [detail](detail.md)\n"
    )
    (tmp_path / "detail.md").write_text("# Detail")
    # Act
    observed = inventory(tmp_path)
    # Assert
    assert observed["unreachable_pages"] == []
    assert observed["missing_markdown"] == []


@pytest.mark.parametrize("missing", [False, True])
def test_inventory_command_emits_json_and_exit_status(
    tmp_path: Path, *, missing: bool
) -> None:
    # Arrange
    (tmp_path / "index.md").write_text("[child](missing.md)" if missing else "# Index")
    # Act
    result = subprocess.run(  # noqa: S603 -- fixed interpreter/module and fixture path.
        [sys.executable, "-m", "tools.knowledge_references", "--root", str(tmp_path)],
        capture_output=True,
        text=True,
        check=False,
    )
    # Assert
    observed = json.loads(result.stdout)
    assert result.returncode == int(missing)
    assert observed["page_count"] == 1
    assert bool(observed["missing_markdown"]) == missing
    assert result.stderr == ""


def test_cycle_in_navigation_terminates_without_orphans(tmp_path: Path) -> None:
    # Arrange
    (tmp_path / "index.md").write_text("[child](child.md)")
    (tmp_path / "child.md").write_text("[root](index.md)")
    # Act
    observed = inventory(tmp_path)
    # Assert
    assert observed["page_count"] == 2
    assert observed["unreachable_pages"] == []


def test_dense_cyclic_navigation_visits_each_page_once(tmp_path: Path) -> None:
    # Arrange
    names = ["index.md", "a.md", "b.md", "c.md"]
    links = "\n".join(f"[page]({name})" for name in names)
    for name in names:
        (tmp_path / name).write_text(links)
    # Act
    observed = inventory(tmp_path)
    # Assert
    assert observed["page_count"] == 4
    assert len(observed["markdown"]) == 16
    assert observed["unreachable_pages"] == []
    assert observed["missing_markdown"] == []


@pytest.mark.parametrize(
    "target", ["mailto:maintainer@example.org", "//example.org/rule.md"]
)
def test_non_filesystem_references_are_not_resolved_locally(
    tmp_path: Path, target: str
) -> None:
    # Arrange
    page = tmp_path / "index.md"
    # Act
    observed = local_target(page, target, tmp_path)
    # Assert
    assert observed is None


@pytest.mark.parametrize(
    "fenced",
    [
        "```\nhidden\n```",
        "~~~\nhidden\n~~~",
        "```\nhidden\n~~~\nstill hidden\n```",
        "````\nhidden\n```\nstill hidden\n````",
    ],
)
def test_fence_scanner_resumes_only_after_matching_closure(fenced: str) -> None:
    # Arrange
    body = f"[before](before.md)\n{fenced}\n[after](after.md)"
    # Act
    observed = list(markdown_targets(body))
    # Assert
    assert observed == ["before.md", "after.md"]


def test_inline_link_does_not_hide_following_reference_navigation() -> None:
    # Arrange
    body = "[inline](inline.md) [used]\n[used]: reference.md\n"
    # Act
    observed = list(markdown_targets(body, navigation=True))
    # Assert
    assert observed == ["inline.md", "reference.md"]


@settings(max_examples=64)
@given(edges=st.lists(st.sets(st.integers(0, 4)), min_size=5, max_size=5))
def test_inventory_reachability_matches_transitive_closure(
    edges: list[set[int]],
) -> None:
    # Arrange: Floyd-Warshall is independent of the inventory's pending-stack traversal.
    names = ["index.md", "a.md", "b.md", "c.md", "d.md"]
    reachable = [
        [target in outgoing or source == target for target in range(5)]
        for source, outgoing in enumerate(edges)
    ]
    for via in range(5):
        for source in range(5):
            for target in range(5):
                reachable[source][target] |= (
                    reachable[source][via] and reachable[via][target]
                )
    expected = sorted(name for node, name in enumerate(names) if not reachable[0][node])
    with TemporaryDirectory() as directory:
        root = Path(directory)
        for source, outgoing in enumerate(edges):
            (root / names[source]).write_text(
                "\n".join(f"[node]({names[target]})" for target in sorted(outgoing))
            )
        # Act
        observed = inventory(root)
        # Assert
        assert observed["unreachable_pages"] == expected


def test_inventory_main_fails_for_missing_links_without_missing_sources(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    (tmp_path / "index.md").write_text("[missing](absent.md)")
    monkeypatch.setattr(sys, "argv", ["knowledge_references", "--root", str(tmp_path)])
    # Act
    observed = main()
    report = json.loads(capsys.readouterr().out)
    # Assert
    assert observed == 1
    assert report["missing_markdown"] == [
        {"page": "index.md", "target": "absent.md", "local": True, "exists": False}
    ]
    assert report["missing_sources"] == []
    assert report["unreachable_pages"] == []


def test_inventory_main_fails_for_orphans_without_missing_targets(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    (tmp_path / "index.md").write_text("# Index")
    (tmp_path / "orphan.md").write_text("# Orphan")
    monkeypatch.setattr(sys, "argv", ["knowledge_references", "--root", str(tmp_path)])
    # Act
    observed = main()
    report = json.loads(capsys.readouterr().out)
    # Assert
    assert observed == 1
    assert report["missing_markdown"] == []
    assert report["missing_sources"] == []
    assert report["unreachable_pages"] == ["orphan.md"]


def test_unused_blank_footnote_does_not_make_its_continuation_navigable() -> None:
    # Arrange: even a malformed label recognized by the scanner remains a footnote,
    # so its indented body cannot hide an otherwise unreachable concept.
    body = "[^ ]: Unused evidence\n    [hidden](hidden.md)\n"
    # Act
    observed = list(markdown_targets(body, navigation=True))
    # Assert
    assert observed == []
