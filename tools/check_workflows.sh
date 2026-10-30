#!/usr/bin/env bash
# Keep the two binary wrappers separate; actionlint's subprocess integration can stall.
set -euo pipefail
shopt -s nullglob
taloon_workflows=(.github/workflows/*.yml .github/workflows/*.yaml)
uvx --from actionlint-py==1.7.12.25 actionlint -shellcheck= -pyflakes= "${taloon_workflows[@]}"
taloon_workflow_tmp=$(mktemp -d)
trap 'rm -rf "$taloon_workflow_tmp"' EXIT
.venv/bin/python - "$taloon_workflow_tmp" <<'PY'
from pathlib import Path
import sys
import yaml

output = Path(sys.argv[1])
count = 0
for workflow in sorted([*Path('.github/workflows').glob('*.yml'), *Path('.github/workflows').glob('*.yaml')]):
    document = yaml.safe_load(workflow.read_text())
    default_shell = document.get('defaults', {}).get('run', {}).get('shell', 'bash')
    for job in document['jobs'].values():
        for step in job.get('steps', []):
            if 'run' not in step:
                continue
            shell = step.get('shell', job.get('defaults', {}).get('run', {}).get('shell', default_shell))
            if shell != 'bash':
                raise SystemExit(f'Unsupported workflow shell for linting: {shell}')
            (output / f'step-{count}.sh').write_text('#!/usr/bin/env bash\n' + step['run'] + '\n')
            count += 1
print(f'Checking {count} workflow run blocks with standalone ShellCheck')
PY
uvx --from shellcheck-py==0.11.0.1 shellcheck --shell=bash tools/check_workflows.sh "$taloon_workflow_tmp"/*.sh
