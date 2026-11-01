# Ordered graph container feasibility

Full graph verification is deferred under the owner's explicit permission.
The current algorithms retain their efficient ordered-map/set and borrowed
predecessor/extra-state representation; no production traversal changes were made.

Pinned Creusot 0.14 experiments used actual BTreeMap insert/get, BTreeSet
insert/contains and a borrowed BTreeSet<(&u32, &[u32])> insertion. Generation
succeeded but warned that calling external functions without contracts yields
an impossible precondition. Generated Coma contains `{false} any`; each targeted
named proof then failed. These are unsupported-contract results, not semantic
counterexamples against taloon. Scalar identity proved as a positive control.

VecDeque is supported via upstream std/deque.rs. Real push/pop operations and a
two-element FIFO postcondition proved without cache. Consequently the obstacle
is not a blanket inability to verify queues or ownership. Whole reachability,
termination and shortest-path claims need map/set support plus graph invariants.

Five reviews: (1) probes match actual ordered APIs; (2) borrowed visited-state
shape is retained; (3) queue/scalar controls distinguish infrastructure failures;
(4) existing Lean selection, Floyd-Warshall and exhaustive DFS properties remain;
(5) no custom trusted map spec, duplicate graph, replacement algorithm or narrowed
inspection scope. Reconsider when upstream provides ordered-container contracts
or a separately justified representation improvement meets performance needs.
