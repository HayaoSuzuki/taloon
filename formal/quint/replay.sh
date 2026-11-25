#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.." || exit 1
generate=false
if [[ ${1:-} == --generate ]]; then
  generate=true
  shift
fi
quint=${1:-quint}
fixture=tests/fixtures/oracle/lifecycle.jsonl
traces=$(mktemp -d)
trap 'rm -rf "$traces"' EXIT
for seed in {1..8}; do
  "$quint" run formal/quint/lifecycle.qnt --main Lifecycle --backend typescript \
    --invariants LifecycleContract --seed "$seed" --max-steps 24 --verbosity 0 \
    --out-itf "$traces/$seed.itf.json"
done
node - "$traces" > "$traces/lifecycle.jsonl" <<'JS'
const { readFileSync } = require("node:fs");
const directory = process.argv[2];
const name = "Lifecycle::LifecycleModel::";
for (let seed = 1; seed <= 8; seed += 1) {
  const { states } = JSON.parse(readFileSync(`${directory}/${seed}.itf.json`, "utf8"),
    (_, value) => value?.["#bigint"] === undefined ? value : Number(value["#bigint"]));
  if (states.length !== 25) throw new Error(`Incomplete trace for seed ${seed}`);
  const steps = states.slice(1).map(state => ({
    operation: state[name + "operation"],
    report: state[name + "report"],
    cache: state[name + "cache"]["#set"].sort(),
  }));
  process.stdout.write(JSON.stringify(steps) + "\n");
}
JS
if "$generate"; then
  cp "$traces/lifecycle.jsonl" "$fixture"
else
  diff -u "$fixture" "$traces/lifecycle.jsonl"
  cargo test --locked --bin taloon quint_replays_acquisition_and_inspection_lifecycle
fi
