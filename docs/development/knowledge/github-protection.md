---
type: Decision
title: GitHub repository protection and security
description: Scoped main/release protection, branch cleanup, Dependabot and nonsecret managed state.
status: accepted
sources:
- id: program
  resource: ../../../infra/github/Pulumi.yaml
  working_tree: modified
  sha256: ee97975a967f8c032d39b1439f23dc11efaebc0ce94e88ef907ea0512589618a
- id: operations
  resource: ../../../infra/github/README.md
  working_tree: modified
  sha256: 0dec7a4be3ba3943509a752aac632597ac5857c55ac01fbf3c18da3870766efc
- id: deployment
  resource: ../work/github-protection/review.md
  working_tree: modified
  sha256: 1c3afe21e06b433c60af5ff12774bd7a66f764ebbbc9c44e4ececd82c7148de5
- id: queue-availability
  resource: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
---
# Decision

Manage GitHub protection/security through pinned Pulumi YAML and canonical
nonsecret local state, following hoimin. Main requires PRs, zero approving
reviews, resolved review threads and 13 actual PR validation jobs bound to
GitHub Actions integration 15368. Requiring prepare, all wheel matrix entries
and validate prevents skipped dependents from hiding a failed prerequisite.
Deletion/non-fast-forward protection and no bypass apply.[^program][^deployment]

GitHub queues require organization ownership; taloon is personal/public. Strict
checks require current main instead. Reconsider a queue after ownership changes
and every required workflow, including wheel validation, supports merge_group.
[^queue-availability][^operations]

# Release identities and security

Separate no-bypass scopes prohibit updates/deletions of refs/tags/v* while
allowing new version tags, and prohibit deletion/non-fast-forward updates of
refs/heads/taloon-release-state. Reservation advances that state directly with
an ancestry-checked CAS lease; PR/status/update-only restrictions on it would
obstruct automation. Recover publication by reusing reservations/artifacts,
not by deleting state or moving published tags.[^program][^operations]

The owner's explicit follow-up enabled merged-branch cleanup and Dependabot
security updates, including their previously disabled alert prerequisite.
Existing secret scanning/push protection stay enabled. A protected imported
repository resource owns only deleteBranchOnMerge; ignoreChanges covers all
other pinned-schema inputs. Alerts and security updates have dedicated resources
and ordered dependencies, avoiding overlapping ownership.[^program][^deployment]

# State and verification

GitHub provider 6.15.0 owns six protected GitHub resources plus provider/stack
metadata (eight checkpoint resources total). Repository/main identities were
imported rather than recreated. Empty-passphrase state never stores credentials;
auth stays in environment. Require current state and one coordinated writer
across clones. Commands, resource IDs, known imported-field deprecation warnings
and maintenance belong to the operations guide.[^operations]

Live API/effective-rule readback and eight-unchanged Pulumi preview establish
configured protection and zero drift. Local bare-Git tests establish forward
reservation compatibility under deletion/non-FF/tag guards; they do not prove
an unobserved production release or a deliberately rejected GitHub push. Branch
CI and final merge state are separately observed. Revisit workflow checks,
provider ignored-field coverage and release-ref policy when those inputs change.
[^deployment]

[^program]: [Pulumi declaration](../../../infra/github/Pulumi.yaml).
[^operations]: [Apply, verify and coordinate state](../../../infra/github/README.md).
[^deployment]: [Deployment and staged review](../work/github-protection/review.md).
[^queue-availability]: [GitHub merge queue availability](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).
