---
type: Decision
title: GitHub main protection
description: Declarative main rules, required checks and nonsecret committed state.
status: accepted
sources:
- id: program
  resource: ../../../infra/github/Pulumi.yaml
  working_tree: untracked
  sha256: 632b0852824c034b146613b2fc2e847afc1fe972d59f6ab31f3b362c56c9dd3f
- id: operations
  resource: ../../../infra/github/README.md
  working_tree: untracked
  sha256: 9ab988d432ec3b9f5b1d61719fd9da8f941082dafaa809e7b5ec5a381d2fc29d
- id: deployment
  resource: ../work/github-protection/review.md
  working_tree: untracked
  sha256: 101fbce0d4e3739a3e3a827ee222719e81d6f7a957c355930b593c591f43a4c6
- id: queue-availability
  resource: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
---
# Decision

Manage the existing main ruleset through Pulumi YAML rather than create a second
rule or rely on undocumented UI configuration. PRs, all 13 current validation
checks, deletion/non-fast-forward protection and an empty bypass set apply to
main. Zero required approvals keeps solo-owner PRs usable. Check names bind to
GitHub Actions integration 15368, including prerequisite jobs whose failure
would otherwise skip dependents.[^program][^deployment]

GitHub merge queues require organization ownership; taloon is a personal public
repository. Strict required checks require current main instead. If ownership
changes, reconsider the queue only after every required workflow can report on
merge_group. Release wheel validation currently has no such trigger.
[^queue-availability][^deployment]

# State and verification

The provider is fixed at 6.15.0 and the only adopted GitHub resource is ruleset
26296914. A local backend and nonsecret canonical checkpoint are committed with
the program, following hoimin. The explicit provider avoids import-owner drift.
Empty-passphrase state must never contain credentials; auth stays in environment.
Updates require current state and one coordinated writer across clones.
Procedures belong to the operations guide.[^program][^operations]

The deployment record distinguishes live API/effective-rule readback and zero
Pulumi drift from destructive enforcement probes or an unobserved PR merge.
Reconsider required checks whenever workflow names/conditions/prerequisites
change; preserve the scope of publication-only jobs.[^deployment]

[^program]: [Pulumi declaration](../../../infra/github/Pulumi.yaml).
[^operations]: [Apply, verify and coordinate state](../../../infra/github/README.md).
[^deployment]: [Deployment and staged review](../work/github-protection/review.md).
[^queue-availability]: [GitHub merge queue availability](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).
