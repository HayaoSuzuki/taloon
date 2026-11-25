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
node - "$traces" <<'JS'
const { readFileSync, writeFileSync } = require("node:fs");
const directory = process.argv[2];
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
    const { states } = JSON.parse(readFileSync(file, "utf8"),
      (_, value) => value?.["#bigint"] === undefined ? value : Number(value["#bigint"]));
    if (states.length !== model.length + 1) throw new Error(`Incomplete trace for ${file}`);
    traces.push(JSON.stringify(states.slice(1).map(state => model.project(state, model.name))));
  }
  writeFileSync(`${directory}/${model.file}.jsonl`, traces.join("\n") + "\n");
}
JS
for corpus in lifecycle shared-cache; do
  if "$generate"; then
    cp "$traces/$corpus.jsonl" "$fixture/$corpus.jsonl"
  else
    diff -u "$fixture/$corpus.jsonl" "$traces/$corpus.jsonl"
  fi
done
if ! "$generate"; then
  cargo test --locked --bin taloon quint_replays
fi
