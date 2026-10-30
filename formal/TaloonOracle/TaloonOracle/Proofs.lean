import TaloonOracle.Model

namespace TaloonOracle

example : evaluate (.atom "MIT") (fun u => u == "MIT") = true := by decide
example : evaluate (.both (.atom "MIT") (.atom "Apache-2.0")) (fun u => u == "MIT") = false := by decide
example : validThrough 30 30 = true := by decide

theorem permission_monotone (expression : Expression) (allowed extended : String → Bool)
    (includes : ∀ unit, allowed unit = true → extended unit = true) :
    evaluate expression allowed = true → evaluate expression extended = true := by
  induction expression with
  | atom unit => exact includes unit
  | both left right leftIH rightIH =>
      simp only [evaluate, Bool.and_eq_true]
      rintro ⟨leftOk, rightOk⟩
      exact ⟨leftIH leftOk, rightIH rightOk⟩
  | either left right leftIH rightIH =>
      simp only [evaluate, Bool.or_eq_true]
      intro accepted
      rcases accepted with leftOk | rightOk
      · exact Or.inl (leftIH leftOk)
      · exact Or.inr (rightIH rightOk)

theorem expiry_is_inclusive (today : Nat) : validThrough today today = true := by
  simp [validThrough]

theorem past_expiry_rejected (today expiry : Nat) (past : expiry < today) :
    validThrough today expiry = false := by
  simp [validThrough, Nat.not_le_of_gt past]

theorem sidecar_insufficient : sufficient false true = false := by decide

theorem archive_sufficient (required : Bool) : sufficient true required = true := by
  simp [sufficient]

theorem missing_target_rejected (expected : List Nat) (observations : List Observation)
    (required : Bool) (id : Nat) (present : id ∈ expected)
    (missing : (observations.filter (fun o => o.request == id)).length = 0) :
    complete expected observations required = false := by
  apply Bool.eq_false_iff.mpr
  intro h
  simp only [complete, Bool.and_eq_true, List.all_eq_true] at h
  have slot := h.1 id present
  simp [missing] at slot

theorem incomplete_precedes_violation (violation warnings denyWarnings : Bool) :
    exitCode false violation warnings denyWarnings = 2 := by
  simp [exitCode]

end TaloonOracle
