/- Finite canonical graph selection; no lock parsing, I/O or metadata assumptions. -/
namespace TaloonOracle.Selection
inductive Kind where
  | normal | extra | development
  deriving BEq, DecidableEq
structure Edge where
  source : Nat
  target : Nat
  kind : Kind
  deriving BEq, DecidableEq

def expand (edges : List Edge) (seen : List Nat) : List Nat :=
  seen ++ (edges.filter (fun e => e.kind != .development && seen.contains e.source)).map Edge.target

def closure : Nat → List Edge → List Nat → List Nat
  | 0, _, roots => roots
  | n + 1, edges, roots => closure n edges (expand edges roots)

def choose (inventory workspace reachable : List Nat) (all : Bool) : List Nat :=
  inventory.filter (fun n => !workspace.contains n && (all || reachable.contains n))

def select (inventory workspace roots : List Nat) (edges : List Edge) : List Nat :=
  choose inventory workspace (closure inventory.length edges roots)
    (roots.any (fun n => workspace.contains n))

theorem roots_retained (fuel : Nat) (edges : List Edge) (roots : List Nat) (x : Nat)
    (included : x ∈ roots) : x ∈ closure fuel edges roots := by
  induction fuel generalizing roots with
  | zero => exact included
  | succ n ih =>
      apply ih
      exact List.mem_append_left _ included

theorem all_external (inventory workspace reachable : List Nat) :
    choose inventory workspace reachable true = inventory.filter (fun n => !workspace.contains n) := by
  simp [choose]

theorem selected_is_external (inventory workspace reachable : List Nat) (all : Bool)
    (x : Nat) (included : x ∈ choose inventory workspace reachable all) :
    x ∈ inventory ∧ x ∉ workspace := by
  have member := List.mem_filter.mp included
  constructor
  · exact member.left
  · simpa using (Bool.and_eq_true_iff.mp member.right).left


theorem reachable_retained (inventory workspace reachable : List Nat) (all : Bool)
    (x : Nat) (inInventory : x ∈ inventory) (outside : x ∉ workspace)
    (reached : x ∈ reachable) : x ∈ choose inventory workspace reachable all := by
  simp [choose, inInventory, outside, reached]

theorem external_root_selected (inventory workspace roots : List Nat) (edges : List Edge)
    (x : Nat) (inInventory : x ∈ inventory) (outside : x ∉ workspace)
    (root : x ∈ roots) : x ∈ select inventory workspace roots edges := by
  apply reachable_retained inventory workspace _ _ x inInventory outside
  exact roots_retained inventory.length edges roots x root

end TaloonOracle.Selection
