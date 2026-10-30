---
type: Decision
title: Documentation ownership
description: Separate user guidance from development knowledge and keep one current
  owner per topic.
status: draft
sources:
- id: agents
  resource: ../../../AGENTS.md
  working_tree: untracked
  sha256: afcfe774b5f3fe1abe4fd5cddc8286b73a596ef8cd78e692c4a96234a2bcabf7
- id: procedure
  resource: ../documentation.md
  working_tree: untracked
  sha256: 24c068334e098883af15221b80bc745b8473767cb4714f51ea95a51bd8a9ab0e
- id: site
  resource: ../../../mkdocs.yml
  working_tree: untracked
  sha256: f8e0d43041de74c191b6d9ee1f50e386bdafba429d55ba89202b75b73a2ab5b0
- id: checker
  resource: ../../../tools/check_docs.py
  working_tree: untracked
  sha256: 725c44b4cceed6394f317d88a42c45818e4839743868983a1bce15aaf6152dee
---
# Decision and scope

Keep user instructions in `docs/user/`. Keep contributor procedures and the
AI development knowledge catalog in `docs/development/`. Start development
through `AGENTS.md` and the task-oriented OKF index. The user site reads only
the user tree.[^agents][^site]

The repository adopts the documentation separation used in taloon's reference,
[kiryl PR #3](https://github.com/HayaoSuzuki/kiryl/pull/3). Use one current owner per topic: guides
own procedures, and OKF concepts own the reasons, conditions, and evidence for
decisions and contracts. Follow the [maintenance procedure](../documentation.md)
instead of repeating its commands here.[^procedure]

# Alternatives and consequences

A shared tree with separate navigation would require a publication allowlist
and allow development context to enter search when the allowlist drifts.
Separate trees give the site a direct content boundary. This adds a second
entry point, so the root README links to both audiences.[^site][^procedure]

A separate ADR collection beside an OKF summary would create two current
explanations of each decision. Store the decision in OKF directly and cite
necessary original evidence. Dated work records remain supporting material;
developers read the current concept first.[^agents][^procedure]

# Limits and reconsideration

This draft records the chosen repository organization, not a reviewed product
architecture. The checks detect selected structural and navigation errors;
developers still review factual claims and user instructions. Revisit the
ownership boundaries if one topic acquires duplicate current explanations or
readers need historical reports to complete ordinary tasks.[^procedure][^checker]

[^agents]: [Agent entry point and ownership rules](../../../AGENTS.md).
[^procedure]: [Documentation maintenance](../documentation.md).
[^site]: [User-site build configuration](../../../mkdocs.yml).
[^checker]: [Documentation validator](../../../tools/check_docs.py).
