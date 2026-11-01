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

The user's subsequent request was to present other useful settings. Review-thread
resolution, immutable v* tags, merged-branch deletion and Dependabot security
updates are recommendations only in this change. Secret scanning/push protection
were already enabled. No extra setting is silently included beyond the declared
main ruleset. The canonical README/configuration remains the operations owner.
