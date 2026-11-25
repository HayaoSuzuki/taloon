module AggregationModel {
  const MUTANT: bool

  pure val compliant = 0
  pure val violation = 1
  pure val incomplete = 2
  pure val inputError = 3
  pure val events = Set(compliant, violation, incomplete, inputError)

  var seen: Set[int]
  var result: int

  action init = all {
    seen' = Set(),
    result' = compliant,
  }

  action step = {
    nondet event = events.oneOf()
    val rank = if (event == inputError) incomplete else event
    all {
      seen' = seen.union(Set(event)),
      result' = if (MUTANT and event == violation) violation
                else if (rank > result) rank else result,
    }
  }

  val Precedence =
    (result == incomplete) == (seen.contains(inputError) or seen.contains(incomplete))
    and (result == violation) ==
      (seen.contains(violation) and not(seen.contains(inputError) or seen.contains(incomplete)))
    and (result == compliant) ==
      (not(seen.contains(violation) or seen.contains(inputError) or seen.contains(incomplete)))
}

module Aggregation {
  import AggregationModel(MUTANT = false).*
}

module AggregationMutant {
  import AggregationModel(MUTANT = true).*
}
