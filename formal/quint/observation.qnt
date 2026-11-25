module ObservationModel {
  const MUTANT: bool
  pure val missing = 0
  pure val duplicate = 3
  pure val payloads = Set(1, 2)

  var slot: int
  var history: List[int]
  var correct: bool

  action init = all {
    slot' = missing,
    history' = [],
    correct' = true,
  }

  action record = {
    nondet payload = payloads.oneOf()
    all {
      slot' = if (slot == missing or MUTANT) payload else duplicate,
      // Two entries suffice to distinguish a singleton from any duplicate.
      history' = if (history.length() < 2) history.append(payload) else history,
      correct' = correct,
    }
  }

  action consume = all {
    slot' = missing,
    history' = [],
    correct' = slot == (if (history.length() == 0) missing
                       else if (history.length() == 1) history[0]
                       else duplicate),
  }

  action step = any { record, consume }
  val ObservationContract = correct
}

module Observation {
  import ObservationModel(MUTANT = false).*
}

module ObservationMutant {
  import ObservationModel(MUTANT = true).*
}
