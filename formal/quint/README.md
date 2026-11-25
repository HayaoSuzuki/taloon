# Behavioral models

Quint 0.33.1 translates these finite models to TLA+ for TLC. Use Java 21.
Quint downloads its backend on first use and starts a local translation
server. Run from the repository root:

```sh
npx --yes --package=@informalsystems/quint@0.33.1 quint verify formal/quint/aggregation.qnt --backend=tlc --main Aggregation --invariants Precedence
npx --yes --package=@informalsystems/quint@0.33.1 quint verify formal/quint/observation.qnt --backend=tlc --main Observation --invariants ObservationContract
npx --yes --package=@informalsystems/quint@0.33.1 quint verify formal/quint/acquisition.qnt --backend=tlc --main Acquisition --invariants AcquisitionContract
npx --yes --package=@informalsystems/quint@0.33.1 quint verify formal/quint/inspection.qnt --backend=tlc --main Inspection --invariants ExitContract
cargo test --locked --workspace --all-features
```

The [CI job](../../.github/workflows/ci.yml) type-checks and verifies each model
and requires an invariant violation from its negative control. To inspect the
counterexample locally, rerun a command with `Mutant` appended to the main
module name. The negative control requires TLC to report an invariant violation.

| Model | Finite domain and invariant | Rust correspondence |
| --- | --- | --- |
| [Aggregation](aggregation.qnt) | A subset of the three package outcomes and an input-error event. Exact precedence is specified independently of the rank fold. | [Aggregation tests](../../crates/taloon-decisions/tests/aggregate.rs) run one canonical package list for every event set. proptest varies list order and multiplicity. |
| [Observation](observation.qnt) | Two distinguishable payloads and repeated record/consume transitions. A bounded history distinguishes empty, singleton and duplicate segments and specifies the consume result. | [Observation tests](../../crates/taloon-decisions/tests/observation.rs) replay named traces through `Observation`. proptest compares arbitrary payload histories against a segmented-history oracle. |
| [Remote acquisition](acquisition.qnt) | Network mode, strength requirement, usable cache outcome and usable sidecar outcome. Offline blocks remote requests, strong requirements bypass sidecars, and incomplete online sidecars route to the archive. | [Acquisition fixtures](../../src/acquisition/http_tests.rs) test the same observable routes, including `offline_strong_policy_rejects_weak_only_cache`, `online_strong_policy_upgrades_weak_only_cache`, and `legacy_supplement_cached_sidecar_respects_network_mode`. |
| [Inspection exit](inspection.qnt) | Two required artifacts, each missing, successful, failed, or duplicated. Only exactly one successful observation per artifact allows permission and warning policy to determine the exit. | [Inspection tests](../../tests/inspection.rs) cover all 128 model inputs with real wheel bytes and parsed policy. PBT varies the order and multiplicity of results. The [CLI property](../../tests/cli.rs) checks the process and JSON exit codes. |

In the acquisition model, cache and sidecar outcomes enter after provenance
and strength validation. The routing decision reaches a terminal phase in at
most two model steps.
An archive phase denotes a fetch attempt. The Rust tests exercise HTTP
responses, cache writes, index parsing, concurrency and the final verdict.

Local TLC with Java 21 on macOS arm64 checked 16 aggregation, 7 observation,
108 acquisition, and 256 inspection states. The mutants yielded an incomplete
result overwritten by a violation, a duplicate returned as a singleton, an
offline cache miss reaching a network route, and an inspection with duplicate
evidence reaching the permission decision, respectively.

See the [verification strategy](../../docs/development/knowledge/verification-strategy.md#verification-across-taloon)
for how these models fit into the whole-project checks.
