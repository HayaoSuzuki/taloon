# SPDX-License-Identifier: Elastic-2.0
"""Check repository documentation and the development OKF bundle, read-only."""

import re
import sys
from pathlib import Path

import yaml

from tools.knowledge_references import (
    inventory,
    local_target,
    markdown_targets,
    unfenced_lines,
)


def _validate_source(source: object, ids: set[str], claim_text: str, body: str) -> None:
    if not isinstance(source, dict):
        msg = "each source must be a mapping"
        raise TypeError(msg)
    resource = source.get("resource")
    if not isinstance(resource, str) or not resource.strip():
        msg = "source resource must be a nonempty string"
        raise ValueError(msg)
    source_id = source.get("id")
    if not isinstance(source_id, str) or not source_id.strip():
        msg = "repository sources require a nonempty id"
        raise ValueError(msg)
    if source_id in ids:
        msg = f"duplicate source id: {source_id}"
        raise ValueError(msg)
    ids.add(source_id)
    if f"[^{source_id}]" not in claim_text or not re.search(
        rf"^\[\^{re.escape(source_id)}\]:", body, re.MULTILINE
    ):
        msg = f"source {source_id} requires a matching footnote"
        raise ValueError(msg)


def _validate_metadata(page: Path, root: Path, text: str) -> None:
    header = text.startswith("---\n")
    if page.name in {"index.md", "log.md"} and page != root / "index.md":
        if header:
            msg = "reserved files must not have frontmatter"
            raise ValueError(msg)
        return
    if not header:
        msg = "missing frontmatter"
        raise ValueError(msg)
    front, body = text[4:].split("\n---\n", 1)
    meta = yaml.safe_load(front)
    if not isinstance(meta, dict):
        msg = "frontmatter must be a mapping"
        raise TypeError(msg)
    if page == root / "index.md":
        if meta != {"okf_version": "0.2"}:
            msg = 'root index must declare only okf_version: "0.2"'
            raise ValueError(msg)
        return
    concept_type = meta.get("type")
    if not isinstance(concept_type, str) or not concept_type.strip():
        msg = "type must be a nonempty string"
        raise ValueError(msg)
    sources = meta.get("sources", [])
    if not isinstance(sources, list):
        msg = "sources must be a list"
        raise TypeError(msg)
    ids: set[str] = set()
    claim_text = "\n".join(
        line
        for line in unfenced_lines(body)
        if not re.match(r"^\s{0,3}\[\^[^\]]+\]:", line)
    )
    for source in sources:
        _validate_source(source, ids, claim_text, body)


def validate_okf(root: Path) -> list[str]:
    """Check OKF structure plus the repository's source-citation conventions."""
    errors = []
    if not (root / "index.md").is_file():
        return [f"{root / 'index.md'}: missing OKF entry point"]
    for page in sorted(root.rglob("*.md")):
        text = page.read_text(encoding="utf-8")
        try:
            _validate_metadata(page, root, text)
        except (TypeError, ValueError, yaml.YAMLError) as error:
            errors.append(f"{page}: {error}")
    return errors


def check_repository(root: Path) -> list[str]:
    """Check bundle navigation and local links, including user and agent entries."""
    root = root.resolve()
    knowledge = root / "docs/development/knowledge"
    errors = validate_okf(knowledge)
    if errors:
        return errors
    references = inventory(knowledge)
    for kind in ("missing_markdown", "missing_sources"):
        for row in references[kind]:
            page = row["page"]
            if not isinstance(page, str):
                msg = "Inventory page must be a string"
                raise TypeError(msg)
            target = row.get("target", row.get("resource"))
            errors.append(f"{knowledge / page}: missing {target}")
    for page in references["unreachable_pages"]:
        errors.append(f"{knowledge / page}: unreachable from the OKF index")
    pages = sorted((root / "docs").rglob("*.md"))
    pages.extend(
        page for name in ("README.md", "AGENTS.md") if (page := root / name).is_file()
    )
    for page in pages:
        if page.is_relative_to(knowledge):
            continue  # The inventory owns bundle-relative paths.
        text = page.read_text(encoding="utf-8")
        for target in markdown_targets(text):
            resolved = local_target(page, target, root)
            if resolved is not None and not resolved.exists():
                errors.append(f"{page}: missing link target {target}")
    return errors


def main() -> int:
    """Report repository documentation errors with a failing exit status."""
    errors = check_repository(Path.cwd())
    for error in errors:
        print(error, file=sys.stderr)  # noqa: T201 -- command-line status/output contract.
    if errors:
        return 1
    print("Documentation links, OKF structure, citations, and navigation passed")  # noqa: T201 -- command-line status/output contract.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
