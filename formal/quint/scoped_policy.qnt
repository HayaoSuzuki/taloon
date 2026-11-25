module ScopedPolicyModel {
  const MUTANT: bool
  type Scenario = {
    scope: int, clarification: int, deny: bool, listed: bool, archive: bool
  }
  type Decision = {
    strong: bool, exit: int, expression: str, overrides: List[str]
  }

  pure val standard = "MIT"
  pure val custom = "LicenseRef-Reviewed"

  var caseId: int
  var scenario: Scenario
  var decision: Decision

  pure def input(number: int): Scenario = {
    scope: number / 24,
    clarification: (number / 8) % 3,
    deny: (number / 4) % 2 == 1,
    listed: (number / 2) % 2 == 1,
    archive: number % 2 == 1,
  }

  pure def evaluate(input: Scenario): Decision = {
    // Scope 1 matches both identity and hash. Scope 2 has a different hash;
    // scope 3 has a different source. Clarification 1 matches the hash.
    val exception = input.scope == 1 or (MUTANT and input.scope == 2)
    val clarified = input.clarification == 1
    val strong = exception or clarified
    val global = if (input.listed) Set(standard) else Set()
    val scoped = if (exception) Set(standard, custom) else Set()
    val allowed = if (input.deny) Set(standard).union(scoped).exclude(global)
                  else global.union(scoped)
    val expression = if (clarified) standard else custom
    if (strong and not(input.archive)) {
      { strong: strong, exit: 2, expression: "", overrides: [] }
    } else {
      val audited = if (exception) ["exception"] else []
      {
        strong: strong,
        exit: if (allowed.contains(expression)) 0 else 1,
        expression: expression,
        overrides: if (clarified) audited.append("clarification") else audited,
      }
    }
  }

  action init = all {
    caseId' = 0,
    scenario' = input(0),
    decision' = evaluate(input(0)),
  }

  action step = {
    val index = if (caseId < 95) caseId + 1 else caseId
    val request = input(index)
    all {
      caseId' = index,
      scenario' = request,
      decision' = evaluate(request),
    }
  }

  val ScopeContract = {
    val matching = scenario.scope == 1
    val corrected = scenario.clarification == 1
    val examined = scenario.archive or not(matching or corrected)
    val permitted = if (corrected) {
      if (scenario.deny) not(scenario.listed) else scenario.listed or matching
    } else matching
    all {
      decision.strong == (matching or corrected),
      (decision.exit == 2) == not(examined),
      (decision.exit == 0) == (examined and permitted),
      (decision.exit == 1) == (examined and not(permitted)),
      decision.expression == (if (not(examined)) "" else if (corrected) standard else custom),
      decision.overrides == (if (not(examined)) []
        else if (matching and corrected) ["exception", "clarification"]
        else if (matching) ["exception"]
        else if (corrected) ["clarification"]
        else []),
    }
  }
}

module ScopedPolicy {
  import ScopedPolicyModel(MUTANT = false).*
}

module ScopedPolicyMutant {
  import ScopedPolicyModel(MUTANT = true).*
}
