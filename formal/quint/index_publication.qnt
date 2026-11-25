module IndexPublicationModel {
  const MUTANT: bool
  type Scenario = { testIndex: bool, inspect: bool, lag: int, damage: int }
  type Check = { result: str, attempts: int, waits: List[int], remaining: int, interval: int }

  var caseId: int
  var scenario: Scenario
  var check: Check

  pure def input(n: int): Scenario = {
    testIndex: n >= 56, inspect: (n / 28) % 2 == 1,
    lag: (n / 7) % 4, damage: n % 7,
  }

  pure val initial = { result: "", attempts: 0, waits: [], remaining: 600, interval: 10 }
  pure val damageResults = ["verified", "partial", "yanked", "digest", "bytes", "predicate", "publisher"]

  pure def response(request: Scenario, attempt: int): str =
    if (request.lag == 3 or (request.lag == 1 and attempt == 1)) "version404"
    else if (request.damage.in(Set(1, 2, 3, 4))) damageResults[request.damage]
    else if (request.lag == 2 and attempt == 1) "provenance404"
    else damageResults[request.damage]

  pure def advance(request: Scenario, current: Check): Check = {
    val updated = current.with("attempts", current.attempts + 1)
    val observed = response(request, updated.attempts)
    val missing = observed.in(Set("version404", "provenance404"))
    if (not(missing)) updated.with("result", observed)
    else if (request.inspect) updated.with("result",
      if (observed == "version404" or MUTANT) "absent" else "http404")
    else {
      val wait = if (updated.interval < updated.remaining) updated.interval else updated.remaining
      updated.with("waits", updated.waits.append(wait))
        .with("remaining", updated.remaining - wait)
        .with("interval", if (updated.interval < 30) updated.interval * 2 else 60)
        .with("result", if (wait == updated.remaining) "http404" else "")
    }
  }

  action init = all { caseId' = 0, scenario' = input(0), check' = initial }

  action step =
    if (check.result == "") all {
      caseId' = caseId, scenario' = scenario, check' = advance(scenario, check),
    } else if (caseId < 111) all {
      caseId' = caseId + 1, scenario' = input(caseId + 1), check' = initial,
    } else all {
      caseId' = caseId, scenario' = scenario, check' = check,
    }

  val IndexContract = {
    val late404 = scenario.lag == 2 and scenario.damage.in(Set(0, 5, 6))
    val retried = scenario.lag == 1 or late404
    val expected = if (scenario.inspect and scenario.lag.in(Set(1, 3))) "absent"
      else if (scenario.lag == 3 or (scenario.inspect and late404)) "http404"
      else damageResults[scenario.damage]
    val waits = if (scenario.inspect) []
      else if (scenario.lag == 3) [10, 20, 40, 60, 60, 60, 60, 60, 60, 60, 60, 50]
      else if (retried) [10] else []
    all {
      check.remaining >= 0,
      check.attempts <= 12,
      check.result == "" or all {
        check.result == expected,
        check.waits == waits,
        check.attempts == (if (scenario.inspect) 1 else if (scenario.lag == 3) 12
                          else if (retried) 2 else 1),
      },
    }
  }
}

module IndexPublication {
  import IndexPublicationModel(MUTANT = false).*
}

module IndexPublicationMutant {
  import IndexPublicationModel(MUTANT = true).*
}
