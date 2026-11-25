module InspectionModel {
  const MUTANT: bool
  pure val outcomes = Set([], [true], [false], [true, true])
  var left: List[bool]
  var right: List[bool]
  var permitted: bool
  var hasWarnings: bool
  var denyWarnings: bool
  var exit: int

  action init = {
    nondet a = outcomes.oneOf()
    nondet b = outcomes.oneOf()
    nondet allow = Bool.oneOf()
    nondet warn = Bool.oneOf()
    nondet deny = Bool.oneOf()
    all {
      left' = a,
      right' = b,
      permitted' = allow,
      hasWarnings' = warn,
      denyWarnings' = deny,
      exit' = -1,
    }
  }

  pure def usable(results: List[bool]): bool =
    if (results.length() == 0) false
    else if (MUTANT or results.length() == 1) results[0]
    else false

  action step = all {
    left' = left,
    right' = right,
    permitted' = permitted,
    hasWarnings' = hasWarnings,
    denyWarnings' = denyWarnings,
    exit' = if (not(usable(left) and usable(right))) 2
            else if (not(permitted) or (hasWarnings and denyWarnings)) 1
            else 0,
  }

  val ExitContract = {
    val complete = left == [true] and right == [true]
    val clean = permitted and not(hasWarnings and denyWarnings)
    exit == -1 or all {
      (exit == 2) == not(complete),
      (exit == 1) == (complete and not(clean)),
      (exit == 0) == (complete and clean),
    }
  }
}

module Inspection {
  import InspectionModel(MUTANT = false).*
}

module InspectionMutant {
  import InspectionModel(MUTANT = true).*
}
