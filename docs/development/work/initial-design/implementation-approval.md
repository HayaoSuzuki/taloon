# Initial design approval and review requirements

On 2026-10-30 the repository owner approved the written
[inspection specification](../../knowledge/license-inspection.md) and
[pure-core design](../../knowledge/sans-io-design.md), and approved proceeding
to implementation planning. The owner added this requirement:

> 各段階ではセルフレビューを最低5回は実施して欲しいです。

Each implementation stage and the planning stage must receive at least five
separate self-review passes. Record the scope, finding, resulting change, and
verification or remaining limit for each pass. Different passes should examine
different failure modes; running one test five times does not satisfy this
requirement. Fix review findings before advancing to the next stage.

On 2026-10-31 the owner selected inline execution in this session and requested
committing the documentation on the current branch before implementation. The
owner also required formal proofs informed by `hoimin/formal`, active fuzzing
and property-based testing, and deliberate use of Rust types and enums informed
by hoimin's implementation experience.

The implementation plan defines stage boundaries and their checks. These
requirements extend verification and representation choices without reducing
the approved inspection scope. This record does not claim that future
implementation reviews or proofs have run.
