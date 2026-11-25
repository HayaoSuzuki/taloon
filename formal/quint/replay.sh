#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.." || exit 1
generate=false
if [[ ${1:-} == --generate ]]; then
  generate=true
  shift
fi
quint=${1:-quint}
fixture=tests/fixtures/oracle
traces=$(mktemp -d)
trap 'rm -rf "$traces"' EXIT
for seed in {1..8}; do
  "$quint" run formal/quint/lifecycle.qnt --main Lifecycle --backend typescript \
    --invariants LifecycleContract --seed "$seed" --max-steps 24 --verbosity 0 \
    --out-itf "$traces/lifecycle-$seed.itf.json"
  "$quint" run formal/quint/shared_cache.qnt --main SharedCache --backend typescript \
    --invariants PublicationContract --seed "$seed" --max-steps 32 --verbosity 0 \
    --out-itf "$traces/shared-cache-$seed.itf.json"
done
"$quint" run formal/quint/extras.qnt --main Extras --backend typescript \
  --invariants ExtrasContract --seed 1 --max-steps 64 --verbosity 0 \
  --out-itf "$traces/extras.itf.json"
"$quint" run formal/quint/locked_forks.qnt --main LockedForks --backend typescript \
  --invariants UniqueIdentity --seed 1 --max-steps 191 --verbosity 0 \
  --out-itf "$traces/locked-forks.itf.json"
"$quint" run formal/quint/path_ties.qnt --main PathTies --backend typescript \
  --invariants CanonicalPath --seed 1 --max-steps 31 --verbosity 0 \
  --out-itf "$traces/path-ties.itf.json"
"$quint" run formal/quint/scoped_policy.qnt --main ScopedPolicy --backend typescript \
  --invariants ScopeContract --seed 1 --max-steps 95 --verbosity 0 \
  --out-itf "$traces/scoped-policy.itf.json"
"$quint" run formal/quint/unused_policy.qnt --main UnusedPolicy --backend typescript \
  --invariants WarningContract --seed 1 --max-steps 127 --verbosity 0 \
  --out-itf "$traces/unused-policy.itf.json"
"$quint" run formal/quint/description_evidence.qnt --main DescriptionEvidence --backend typescript \
  --invariants DescriptionContract --seed 1 --max-steps 323 --verbosity 0 \
  --out-itf "$traces/description-evidence.itf.json"
"$quint" run formal/quint/sidecar_fallback.qnt --main SidecarFallback --backend typescript \
  --invariants FallbackContract --seed 1 --max-steps 239 --verbosity 0 \
  --out-itf "$traces/sidecar-fallback.itf.json"
"$quint" run formal/quint/legacy_scan.qnt --main LegacyScan --backend typescript \
  --invariants ScanContract --seed 1 --max-steps 79 --verbosity 0 \
  --out-itf "$traces/legacy-scan.itf.json"
"$quint" run formal/quint/http_routing.qnt --main HttpRouting --backend typescript \
  --invariants RoutingContract --seed 1 --max-steps 700 --verbosity 0 \
  --out-itf "$traces/http-routing.itf.json"
"$quint" run formal/quint/release_reservation.qnt --main ReleaseReservation --backend typescript \
  --invariants ReservationContract --seed 1 --max-steps 400 --verbosity 0 \
  --out-itf "$traces/release-reservation.itf.json"
"$quint" run formal/quint/index_publication.qnt --main IndexPublication --backend typescript \
  --invariants IndexContract --seed 1 --max-steps 500 --verbosity 0 \
  --out-itf "$traces/index-publication.itf.json"
"$quint" run formal/quint/release_promotion.qnt --main ReleasePromotion --backend typescript \
  --invariants PromotionContract --seed 1 --max-steps 1300 --verbosity 0 \
  --out-itf "$traces/release-promotion.itf.json"
node - "$traces" <<'JS'
const { readFileSync, writeFileSync } = require("node:fs");
const directory = process.argv[2];
function readStates(file) {
  return JSON.parse(readFileSync(file, "utf8"),
    (_, value) => value?.["#bigint"] === undefined ? value : Number(value["#bigint"])).states;
}
const models = [
  {
    file: "lifecycle", name: "Lifecycle::LifecycleModel::", length: 24,
    project: (state, name) => ({
      operation: state[name + "operation"],
      report: state[name + "report"],
      cache: state[name + "cache"]["#set"].sort(),
    }),
  },
  {
    file: "shared-cache", name: "SharedCache::SharedCacheModel::", length: 32,
    project: (state, name) => ({
      event: state[name + "event"],
      visible: state[name + "visible"]["#set"].sort(),
    }),
  },
];
for (const model of models) {
  const traces = [];
  for (let seed = 1; seed <= 8; seed += 1) {
    const file = `${directory}/${model.file}-${seed}.itf.json`;
    const states = readStates(file);
    if (states.length !== model.length + 1) throw new Error(`Incomplete trace for ${file}`);
    traces.push(JSON.stringify(states.slice(1).map(state => model.project(state, model.name))));
  }
  writeFileSync(`${directory}/${model.file}.jsonl`, traces.join("\n") + "\n");
}
const prefix = "Extras::ExtrasModel::";
const cases = new Map();
for (const state of readStates(`${directory}/extras.itf.json`)) {
  if (state[prefix + "queue"].length === 0) {
    cases.set(state[prefix + "caseId"], {
      scenario: state[prefix + "scenario"],
      path: state[prefix + "path"],
    });
  }
}
if (cases.size !== 8) throw new Error("The extras trace must reach all eight cases");
writeFileSync(`${directory}/extras.jsonl`,
  Array.from(cases.values(), value => JSON.stringify(value)).join("\n") + "\n");
function writeOrderedCases(file, prefix, count, fields) {
  const rows = readStates(`${directory}/${file}.itf.json`).map((state, index) => {
    if (state[prefix + "caseId"] !== index) throw new Error(`Unexpected case in ${file} at ${index}`);
    return JSON.stringify(Object.fromEntries(fields.map(field => [field, state[prefix + field]])));
  });
  if (rows.length !== count) throw new Error(`Expected ${count} cases in ${file}`);
  writeFileSync(`${directory}/${file}.jsonl`, rows.join("\n") + "\n");
}
writeOrderedCases("locked-forks", "LockedForks::LockedForksModel::", 192, ["selector", "decision"]);
writeOrderedCases("path-ties", "PathTies::PathTiesModel::", 32, ["scenario", "path", "development"]);
writeOrderedCases("scoped-policy", "ScopedPolicy::ScopedPolicyModel::", 96, ["scenario", "decision"]);
writeOrderedCases("unused-policy", "UnusedPolicy::UnusedPolicyModel::", 128, ["scenario", "decision"]);
writeOrderedCases("description-evidence", "DescriptionEvidence::DescriptionEvidenceModel::", 324, ["scenario", "decision"]);
writeOrderedCases("sidecar-fallback", "SidecarFallback::SidecarFallbackModel::", 240, ["scenario", "decision"]);
writeOrderedCases("legacy-scan", "LegacyScan::LegacyScanModel::", 80, ["scenario", "decision"]);
const httpPrefix = "HttpRouting::HttpRoutingModel::";
const finished = new Map();
for (const state of readStates(`${directory}/http-routing.itf.json`)) {
  const { outcome, calls, bytes } = state[httpPrefix + "session"];
  if (outcome) finished.set(state[httpPrefix + "caseId"], {
    scenario: state[httpPrefix + "scenario"], outcome, calls, bytes,
  });
}
if (finished.size !== 96) throw new Error("The HTTP trace must finish all 96 cases");
writeFileSync(`${directory}/http-routing.jsonl`, Array.from(finished.values(), value =>
  JSON.stringify(value)).join("\n") + "\n");
const releasePrefix = "ReleaseReservation::ReleaseReservationModel::";
const reservations = new Map();
for (const state of readStates(`${directory}/release-reservation.itf.json`)) {
  const { phase, tags, head, attempts, result } = state[releasePrefix + "reservation"];
  if (phase === "done") reservations.set(state[releasePrefix + "caseId"], {
    scenario: state[releasePrefix + "scenario"], tags, head, attempts, result,
  });
}
if (reservations.size !== 32) throw new Error("The reservation trace must finish all 32 cases");
writeFileSync(`${directory}/release-reservation.jsonl`, Array.from(reservations.values(), value =>
  JSON.stringify(value)).join("\n") + "\n");
const indexPrefix = "IndexPublication::IndexPublicationModel::";
const publications = new Map();
for (const state of readStates(`${directory}/index-publication.itf.json`)) {
  const { result, attempts, waits } = state[indexPrefix + "check"];
  if (result) publications.set(state[indexPrefix + "caseId"], {
    scenario: state[indexPrefix + "scenario"], result, attempts, waits,
  });
}
if (publications.size !== 112) throw new Error("The publication trace must finish all 112 cases");
writeFileSync(`${directory}/index-publication.jsonl`, Array.from(publications.values(), value =>
  JSON.stringify(value)).join("\n") + "\n");
const promotionPrefix = "ReleasePromotion::ReleasePromotionModel::";
const promotions = new Map();
for (const state of readStates(`${directory}/release-promotion.itf.json`)) {
  const { phase, jobs, uploads, verified } = state[promotionPrefix + "promotion"];
  if (phase === 0) promotions.set(state[promotionPrefix + "caseId"], {
    scenario: state[promotionPrefix + "scenario"], jobs, uploads, verified,
  });
}
if (promotions.size !== 252) throw new Error("The promotion trace must finish all 252 cases");
writeFileSync(`${directory}/release-promotion.jsonl`, Array.from(promotions.values(), value =>
  JSON.stringify(value)).join("\n") + "\n");
JS
for corpus in lifecycle shared-cache extras locked-forks path-ties scoped-policy unused-policy description-evidence sidecar-fallback legacy-scan http-routing release-reservation index-publication release-promotion; do
  if "$generate"; then
    cp "$traces/$corpus.jsonl" "$fixture/$corpus.jsonl"
  else
    diff -u "$fixture/$corpus.jsonl" "$traces/$corpus.jsonl"
  fi
done
if ! "$generate"; then
  cargo test --locked --bin taloon quint_replays
  cargo test --locked --test resolution_paths quint_replays
  cargo test --locked --test resolution_matching quint_replays_locked_forks
  cargo test --locked --test inspection quint_replays_scoped_policy
  cargo test --locked --test policy_usage quint_replays_unused_policy
  cargo test --locked --test description quint_replays_description_evidence
fi
