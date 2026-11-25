module LegacyScanModel {
  const MUTANT: bool

  type Scenario = { bsd: bool, outcome: int, initiallyAvailable: bool, phase: int }
  type Decision = {
    cache: str, hits: int, indexRequests: int, metadataRequests: int, archiveRequests: int,
    exit: int, proof: str, expression: str, origin: str, issue: str
  }

  var caseId: int
  var scenario: Scenario
  var decision: Decision

  pure def input(number: int): Scenario = {
    bsd: number >= 40,
    outcome: (number / 10) % 4,
    initiallyAvailable: (number / 5) % 2 == 1,
    phase: number % 5,
  }

  pure def count(condition: bool): int = if (condition) 1 else 0

  pure def archiveExpression(input: Scenario): str =
    if (input.outcome == 0) {
      if (input.bsd) "BSD-3-Clause" else "MIT"
    } else if (input.outcome == 2) "ISC"
    else ""

  pure def acquire(input: Scenario, before: str): Decision = {
    val offline = input.phase.in(Set(1, 3))
    val cold = before == "empty"
    val license = archiveExpression(input)
    val revisit = MUTANT and before == "archive" and license == "" and not(offline)
    val reuse = not(revisit) and (before == "archive" or offline)
    val fetch = not(reuse)
    val available = input.phase == 2 or (input.phase == 0 and input.initiallyAvailable)
    val cache = if (fetch and available) "archive" else if (cold) "pending" else before
    val proof = if (reuse or available) (if (cache == "archive") "archive" else "index") else ""
    val interpreted = if (proof == "archive") license else ""
    {
      cache: cache,
      hits: count(reuse),
      indexRequests: count(cold),
      metadataRequests: count(cold),
      archiveRequests: count(fetch),
      exit: if (interpreted == "") 2 else 0,
      proof: proof,
      expression: interpreted,
      origin: if (interpreted == "") "" else if (input.outcome == 2) "license_expression"
              else "legacy_supplement",
      issue: if (proof == "") "acquisition"
             else if (interpreted == "") "unknown_license" else "",
    }
  }

  action init = all {
    caseId' = 0,
    scenario' = input(0),
    decision' = acquire(input(0), "empty"),
  }

  action step = {
    val index = if (caseId < 79) caseId + 1 else caseId
    val request = input(index)
    all {
      caseId' = index,
      scenario' = request,
      decision' = if (caseId == 79) decision
                  else acquire(request, if (request.phase == 0) "empty" else decision.cache),
    }
  }

  // Derive the expected report directly from schedule position and archive outcome.
  val ScanContract = {
    val failedCold = scenario.phase == 0 and not(scenario.initiallyAvailable)
    val pending = scenario.phase == 1 and not(scenario.initiallyAvailable)
    val conclusive = not(failedCold or pending)
    val recognized = conclusive and scenario.outcome.in(Set(0, 2))
    val terminalExpression = if (scenario.outcome == 2) "ISC"
      else if (scenario.bsd) "BSD-3-Clause" else "MIT"
    val requests = scenario.phase == 0 or (scenario.phase == 2 and not(scenario.initiallyAvailable))
    all {
      decision.indexRequests == count(scenario.phase == 0),
      decision.metadataRequests == decision.indexRequests,
      decision.archiveRequests == count(requests),
      decision.hits == count(not(requests)),
      decision.cache == (if (conclusive) "archive" else "pending"),
      decision.proof == (if (conclusive) "archive" else if (pending) "index" else ""),
      decision.exit == (if (recognized) 0 else 2),
      decision.expression == (if (recognized) terminalExpression else ""),
      decision.origin == (if (not(recognized)) "" else
        if (scenario.outcome == 2) "license_expression" else "legacy_supplement"),
      decision.issue == (if (failedCold) "acquisition" else if (recognized) "" else "unknown_license"),
    }
  }
}

module LegacyScan {
  import LegacyScanModel(MUTANT = false).*
}

module LegacyScanMutant {
  import LegacyScanModel(MUTANT = true).*
}
