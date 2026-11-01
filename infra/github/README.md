# GitHub repository protection

This Pulumi YAML project manages GitHub protection/security for
`HayaoSuzuki/taloon`. The provider is pinned to 6.15.0; deployment was verified
with Pulumi CLI 3.273.0. Every adopted resource is protected from Pulumi deletion.

| Resource | Active configuration |
| --- | --- |
| [Main ruleset 26296914](https://github.com/HayaoSuzuki/taloon/rules/26296914) | PR, zero approving reviews, resolved review threads, all 13 declared CI checks, current-main tracking, deletion/non-fast-forward protection, no bypass |
| [Release tags 26313565](https://github.com/HayaoSuzuki/taloon/rules/26313565) | `refs/tags/v*`: prohibit updates/deletions, allow new version tags, no bypass |
| [Release state 26313567](https://github.com/HayaoSuzuki/taloon/rules/26313567) | `refs/heads/taloon-release-state`: prohibit deletion/non-fast-forward updates, allow direct forward updates, no bypass |
| Repository settings | Automatically delete merged PR branches; every other repository input is ignored/preserved |
| Vulnerability alerts | Enabled through the dedicated alert resource |
| Dependabot security updates | Enabled, with an explicit dependency on vulnerability alerts |

The 13 required checks bind exact names to GitHub Actions integration `15368`.
They include prepare, every wheel smoke matrix entry and validate, so skipped
dependents cannot hide a failed prerequisite. Reservation/publication-only jobs
are outside the PR gate. Secret scanning/push protection were already enabled
and are preserved rather than adopted by this project.

The reference [hoimin infrastructure](https://github.com/HayaoSuzuki/hoimin/tree/cf5c4e724e373ac9d702dda0169cfa337463c234/infra/github)
uses a merge queue. GitHub restricts queues to organization-owned repositories;
this personal repository uses strict required checks instead. See
[queue availability](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).
Before enabling a future queue, every required workflow, including wheel
validation, must report on `merge_group`.

## Release compatibility

The release workflow atomically creates a new version tag and advances
`taloon-release-state`. It checks ancestry first and uses a ref-specific
`--force-with-lease` as a compare-and-swap lease. The actual state update is
fast-forward; the command flag does not require a non-fast-forward update.
Existing reservations are reused, never retagged. New tag creation remains
allowed. The state branch deliberately has no PR/check/update-only gate, because
reservation runs before the remaining release jobs and pushes directly.

Never delete the state branch to recover a publication failure. Use the
[release recovery procedure](../../docs/development/releases.md), which retains
reservations and original artifacts. Merged-branch cleanup must respect the
protected state reference.

## Apply and verify

Install the [Pulumi CLI](https://www.pulumi.com/docs/install/) and authenticate
GitHub CLI with repository Administration read/write. Use the latest committed
program/state and one coordinated writer. Run from `infra/github`; backend
`file://./state` is relative to that directory. Do not run `pulumi login` or
initialize/import another stack for these identities.

```sh
export GITHUB_TOKEN="$(gh auth token)"
export GITHUB_OWNER=HayaoSuzuki
export PULUMI_CONFIG_PASSPHRASE=""

pulumi stack --stack production
pulumi preview --stack production --refresh --diff
pulumi up --stack production --refresh --diff
pulumi stack output --stack production

gh api repos/HayaoSuzuki/taloon/rules/branches/main
gh api repos/HayaoSuzuki/taloon/rules/branches/taloon-release-state
gh api repos/HayaoSuzuki/taloon/rulesets/26313565
gh api repos/HayaoSuzuki/taloon --jq .delete_branch_on_merge
gh api repos/HayaoSuzuki/taloon/vulnerability-alerts --silent
gh api repos/HayaoSuzuki/taloon/automated-security-fixes
pulumi preview --stack production --refresh --expect-no-changes
```

Commit canonical `state/.pulumi/` and `Pulumi.production.yaml` after updates or
refreshes. Locks/history/backups/local metadata are ignored. Local backend locks
do not coordinate different clones: do not merge divergent checkpoints textually
or apply an old checkout. State uses an empty passphrase for public settings;
credentials belong only in environment, never config/state/secret resources.

The existing repository identity is imported solely for `deleteBranchOnMerge`.
`ignoreChanges` covers every other repository input supported by pinned 6.15.0,
including name, visibility, merge methods and security settings. Alerts use a
separate dedicated resource. The provider can warn about imported deprecated
`defaultBranch`/`vulnerabilityAlerts` fields; they are not configured here.
On provider upgrade, review the ignore list and defaults before applying.

Update required checks alongside workflow names/conditions/prerequisites.
Existing resources were applied through `gh api`, imported/refreshed and compared
with the program. Final preview reported eight unchanged resources. Repository
identity and existing main rule were not recreated; new tag/state rules are
separate scopes. Intentional retirement requires reviewing protection removal
before deletion, rather than deleting checkpoint entries.

The [deployment review](../../docs/development/work/github-protection/review.md)
distinguishes live configuration, local Git compatibility and branch CI from
production destructive probes or an actual release after policy changes.
