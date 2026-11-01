/- Exact-key snapshot filtering, independent of HTTP and Rust synchronization. -/
namespace TaloonOracle.IndexLookup

structure Entry where
  url : String
  archive : Option Nat
  metadata : Option Nat
  deriving DecidableEq

inductive Outcome where
  | none
  | invalid
  | hashMismatch
  | associated : Nat → Outcome
  deriving DecidableEq, BEq

def select (entries : List Entry) (url : String) : List Entry :=
  entries.filter (fun entry => decide (entry.url = url))

def snapshot (scope : List String) (entries : List Entry) : List Entry :=
  entries.filter (fun entry => decide (entry.url ∈ scope))

def lookup (entries : List Entry) (url : String) (locked : Nat) : Outcome :=
  match select entries url with
  | [] => .none
  | [entry] =>
      match entry.metadata with
      | none => .none
      | some metadata =>
          match entry.archive with
          | none => .invalid
          | some archive => if archive = locked then .associated metadata else .hashMismatch
  | _ => .invalid

theorem selected_snapshot (scope : List String) (entries : List Entry) (url : String)
    (included : url ∈ scope) : select (snapshot scope entries) url = select entries url := by
  unfold select snapshot
  rw [List.filter_filter]
  apply List.filter_congr
  intro entry _
  by_cases same : entry.url = url
  · simp [same, included]
  · simp [same]

theorem snapshot_preserves_lookup (scope : List String) (entries : List Entry)
    (url : String) (locked : Nat) (included : url ∈ scope) :
    lookup (snapshot scope entries) url locked = lookup entries url locked := by
  unfold lookup
  rw [selected_snapshot scope entries url included]

theorem wrong_hash_rejected (url : String) (archive metadata locked : Nat)
    (different : archive ≠ locked) :
    lookup [⟨url, some archive, some metadata⟩] url locked = .hashMismatch := by
  simp [lookup, select, different]

theorem duplicate_rejected (url : String) (a b : Entry) (locked : Nat)
    (first : a.url = url) (second : b.url = url) :
    lookup [a, b] url locked = .invalid := by
  simp [lookup, select, first, second]

end TaloonOracle.IndexLookup
