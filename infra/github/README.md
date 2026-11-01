# GitHub main protection

This Pulumi YAML project owns the existing `main-protection` ruleset
[26296914](https://github.com/HayaoSuzuki/taloon/rules/26296914) for
`HayaoSuzuki/taloon`. The GitHub provider is pinned to 6.15.0; deployment was
verified with Pulumi CLI 3.273.0. The repository itself is not a managed resource.

The rule requires a PR, zero approving reviews, and the 13 checks declared in
`Pulumi.yaml`. Required checks are bound to GitHub Actions integration `15368`.
They include the release preparation/validation prerequisites and all three wheel
smoke jobs; publication-only jobs are excluded. Branch deletion and force pushes
are prohibited, with no bypass actors. Branches must be current with main before
merging (`strictRequiredStatusChecksPolicy: true`).

The reference [hoimin configuration](https://github.com/HayaoSuzuki/hoimin/tree/cf5c4e724e373ac9d702dda0169cfa337463c234/infra/github)
uses a merge queue. GitHub limits merge queues to organization-owned repositories;
this personal repository uses strict required checks instead. See the
[GitHub queue availability documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).
Review this choice if the repository moves to an organization.

## Apply and verify

Install the [Pulumi CLI](https://www.pulumi.com/docs/install/) and authenticate
GitHub CLI as an identity with repository Administration read/write. Start from
the latest committed configuration and state. Run from `infra/github`; the
backend `file://./state` is relative to that directory. Do not run `pulumi login`
or initialize another stack for the same ruleset.

```sh
export GITHUB_TOKEN="$(gh auth token)"
export GITHUB_OWNER=HayaoSuzuki
export PULUMI_CONFIG_PASSPHRASE=""

pulumi stack --stack production
pulumi preview --stack production --refresh --diff
pulumi up --stack production --refresh --diff

pulumi stack output rulesetId --stack production
gh api repos/HayaoSuzuki/taloon/rulesets/26296914
gh api repos/HayaoSuzuki/taloon/rules/branches/main
pulumi preview --stack production --refresh --expect-no-changes
```

Only one maintainer should update the stack at a time. Local backend locks do
not coordinate separate clones. After any update or refresh, inspect and commit
changes to `Pulumi.production.yaml` and canonical `state/.pulumi/` before another
maintainer applies settings. Never merge divergent checkpoint JSON textually or
apply an old state checkout. Transient locks, history, backups and file metadata
are ignored; canonical stack and backend metadata are tracked.

The stack uses an empty passphrase and manages public repository settings only.
Credentials must remain in the environment, not `pulumi config` or committed
state. Do not add secrets or credential resources to this project.

When PR check names or prerequisites change, update `requiredChecks` alongside
the workflow. Required jobs must actually execute for PRs; requiring a skipped
dependent alone is insufficient because GitHub accepts skipped checks. Publish,
release reservation and push-only checks must not become PR prerequisites.
If merge queues become available, first ensure every required workflow,
including wheel validation, runs on `merge_group` before enabling a queue.

## Existing resource and recovery

The initial setup imported `taloon:26296914` through the explicit `github`
provider, reviewed an in-place update, applied it with `gh api`, then refreshed
Pulumi. A final stack update reconciled only its output; the final preview showed
three unchanged resources. The import did not create or delete a ruleset.
The committed `production` stack already owns that identity: do not re-import it
into a new stack or create a duplicate rule.

Inspect unexpected drift before applying. Ruleset `protect: true` prevents an
accidental Pulumi deletion; intentional retirement requires changing that
protection and reviewing the removal separately. Repository admins can still
edit rulesets through GitHub; reflect authorized changes in the program/state.

The [deployment review](../../docs/development/work/github-protection/review.md)
records observed scope. A successful normal PR merge confirms operational checks;
preview/API readback alone does not demonstrate a rejected failing merge or an
actual deletion/force-push attempt. Those destructive probes are unnecessary.
