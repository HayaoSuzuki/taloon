# GitHub protection deployment review

2026-11-01. Owner request: follow the local hoimin infra, add GitHub-side settings,
and overwrite/delete existing ruleset 26296914 if useful. The clean main checkout
at `e6819274fabc682debbb6f38fd49e880a0b913aa` was branched to `infra/github-rules`.
No merge or publication is authorized by this administrative configuration task.

## Design: five passes

1. Read hoimin infra at public commit cf5c4e724e373ac9d702dda0169cfa337463c234.
   Reuse its pinned GitHub provider, explicit owner, protected ruleset, local
   backend and committed nonsecret canonical state.
2. Read existing rule and legacy protection endpoint. One active repository rule
   protected default main with PR/deletion/non-fast-forward; no required checks
   or bypass actors. Legacy protection reported branch not protected.
3. Verified current repo is public/user-owned and GitHub's official merge-queue
   availability source requires organization ownership. Select strict required
   checks rather than install an unusable queue. No workflow changes are needed.
4. Match exact check names/integration ID 15368 to current GitHub check runs and
   source workflows. Require all 13 PR validation jobs, including prepare and
   validate prerequisites; do not require push/publication-only jobs.
5. Preserve zero required approvals and allowed merge methods. Reuse existing ID
   in place, keep bypass empty and manage no repository/secret resource.

## Implementation: five passes

1. Download official Pulumi 3.273.0 darwin-arm64 archive through gh and verify its
   SHA-256 against the official release checksums. Pin provider 6.15.0.
2. Create one explicit provider, then import existing taloon:26296914 through that
   provider. No default provider duplication or rule deletion/creation occurred.
3. Preview showed only one in-place rule update: name/main ref and 13 required
   checks with strict tracking. No replacement/deletion was planned.
4. Apply reviewed payload through gh, preserving existing pull-request policy;
   refresh state. Only stack-output reconciliation remained, then Pulumi reported
   three unchanged resources and rulesetId 26296914.
5. Inspect inputs/state and active credential bytes before committing. Credentials
   stay in environment; state contains public settings, not the runtime token.
   Keep canonical backend/stack files; ignore local plugins/locks/history/backups.

## Verification and handoff: five passes

1. Read back active ruleset and effective main rules; compare context/app-ID set,
   strict policy, zero bypass, PR count, branch scope, deletion and force-push
   protections against Pulumi declaration.
2. Run preview with refresh/expect-no-changes; three unchanged resources. No
   template-only success is substituted for observed GitHub configuration.
3. Validate documentation/OKF, YAML syntax, exact check names and state ownership.
   No runtime/Python implementation changed, so no new runtime test is justified.
4. Fresh independent final review inspects source, state handling and readback.
   Reconcile actual findings before the authorized commit/push/PR handoff.
5. Record final CI for the configuration branch separately. API/preview establish
   active configuration, not that a deliberately failing PR/deletion/force push
   was attempted. No destructive validation or main merge is performed.

See [the canonical operations guide](../../../../infra/github/README.md) for
commands and state coordination. Source files, state and GitHub readback are the
owners; no duplicate current configuration is maintained in this report.


Fresh reviewer `github_rules_final_review` (separate context, 2026-11-01) found
no Critical/Important/Minor issue. It independently compared all 13 context/app-ID
pairs across declaration, checkpoint, API and effective-main snapshots; inspected
release prepare/skipped-reserve/wheel/validate prerequisites, explicit provider
and existing protected ID, state hygiene and source hashes. It did not rerun
live API/Pulumi, observe this branch's CI/merge, or perform destructive probes.
The executor separately read live settings and observed the final no-change
preview. No reviewer finding is deferred.

Credential-byte and credential-field checks passed on the canonical files;
plugin/lock/history/backup paths are ignored and the canonical metadata/stack
paths remain eligible for tracking. Documentation links/OKF and strict site build
passed. No behavior test was added for reversible declarative configuration;
Pulumi validation and actual API/effective-rule readback are the relevant checks.

The user's subsequent request initially asked for recommendations, then explicitly
authorized all four and requested protection for taloon-release-state. The next
stage records those additions; they were not silently included in the first
main-only deployment. Secret scanning/push protection were already enabled.


## Authorized additions: design, five passes

1. Resolve review threads before main merge while retaining zero approving reviews.
2. Protect v* tags against updates/deletions without restricting new tag creation.
3. Adopt the existing repository only for deleteBranchOnMerge. Ignore every other
   input in the pinned schema, including deprecated vulnerabilityAlerts; use its
   dedicated resource instead of sharing management with repository defaults.
4. Enable prerequisite vulnerability alerts (previously disabled), then Dependabot
   updates with explicit dependency ordering. Keep existing scanning flags intact.
5. Confirm release state's exact ref and reservation ancestry/CAS code. Prohibit
   deletion/non-fast-forward only; a PR/status/update restriction would obstruct
   the direct reservation job and is deliberately absent.

## Authorized additions: implementation, five passes

1. Preview the expanded program before mutation; validate tag and branch payload
   scopes. YAML flow-list interpolation required quotes, corrected before apply.
2. Apply main-thread resolution, branch cleanup, alerts and security updates through
   gh. Create release tag rule 26313565 and state rule 26313567 with no bypass.
3. Import existing repository/alert/security identities and new rules through the
   same explicit provider. No existing repository or main rule is recreated.
4. Refresh and require zero resource drift before reconciling local outputs.
   Final preview --refresh --expect-no-changes reports eight unchanged resources.
5. Compare repository before/after: descriptions, visibility, feature flags, merge
   methods, default branch and existing secret scanning/push protection unchanged.
   New managed fields and alert/security settings agree with live API.

## Authorized additions: verification, five passes

1. Read effective main and state-branch rules plus both new rule objects. Confirm
   exact context/app-ID set, resolved threads, zero approvals, tag creation not
   restricted, and state scope containing only deletion/non-fast-forward rules.
2. Run the actual reservation helper against a synthetic bare Git remote denying
   deletion/non-fast-forward and rejecting existing v* tag changes. Initial,
   next-version and repeated reservations succeed. This checks real Git lease/
   ancestry compatibility, not an unobserved GitHub production release.
3. Inspect scoped repository adoption/ignored fields and dedicated alert resource;
   record pinned provider's deprecated imported-field warnings without silently
   suppressing them. They do not represent configured ownership of those fields.
4. Revalidate credentials/state inclusion, docs/OKF and exact source hashes; all
   canonical resources remain protected. Fresh review covers expanded ownership.
5. Update the existing PR and observe checks for its final SHA. Changes are already
   live; no destructive production push/delete probe or main merge is required.


Fresh expanded-scope reviewer `github_extended_rules_review` (separate context,
2026-11-01) found no Critical/Important/Minor issue after inspecting all three
rulesets, repository ignore-list coverage, six protected GitHub resources,
security dependency ordering, canonical-state hygiene and updated source hashes.
The source/checkpoint/API/effective-rule snapshots agree. It read the eight-
unchanged preview and reservation implementation but did not independently run
API/Pulumi, repeat the local experiment or observe a production release/final
branch CI. No technical finding is deferred.
