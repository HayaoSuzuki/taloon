/- A model of the atomic reservation decision, not Git's implementation.
   Commit ancestry and successful compare-and-swap are explicit premises. -/
namespace TaloonOracle.Release

inductive Decision where
  | reuse : Nat → Decision
  | fresh : Nat → Decision
  | stale : Decision
  deriving DecidableEq

def next (floor latest : Nat) : Nat := max floor (latest + 1)

def choose (existing : Option Nat) (follows : Bool) (floor latest : Nat) : Decision :=
  match existing with
  | some version => .reuse version
  | none => if follows then .fresh (next floor latest) else .stale

theorem existing_is_never_replaced (version floor latest : Nat) (follows : Bool) :
    choose (some version) follows floor latest = .reuse version := rfl

theorem stale_without_reservation_rejected (floor latest : Nat) :
    choose none false floor latest = .stale := rfl

theorem fresh_version_increases (floor latest : Nat) : latest < next floor latest := by
  exact Nat.lt_of_lt_of_le (Nat.lt_succ_self latest) (Nat.le_max_right floor (latest + 1))

theorem fresh_respects_floor (floor latest : Nat) : floor ≤ next floor latest :=
  Nat.le_max_left floor (latest + 1)

end TaloonOracle.Release
