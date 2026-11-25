module UnusedPolicyModel {
  const MUTANT: bool
  type Scenario = {
    outcome: int, deny: bool, listed: bool, betaScope: bool, alphaScope: bool, clarify: bool
  }
  type Decision = { exit: int, strictExit: int, warnings: List[str] }

  var caseId: int
  var scenario: Scenario
  var decision: Decision

  pure def input(number: int): Scenario = {
    outcome: number / 32,
    deny: (number / 16) % 2 == 1,
    listed: (number / 8) % 2 == 1,
    betaScope: (number / 4) % 2 == 1,
    alphaScope: (number / 2) % 2 == 1,
    clarify: number % 2 == 1,
  }

  // Alpha declares Apache-2.0 OR MIT OR BSD-3-Clause. Beta's ISC
  // declaration is replaced by BSD-3-Clause only for a matching hash.
  pure def collectWarnings(input: Scenario): List[str] = {
    val alpha = if (MUTANT) Set("Apache-2.0")
                else Set("Apache-2.0", "MIT", "BSD-3-Clause")
    val beta = if (input.clarify) "BSD-3-Clause" else "ISC"
    val used = alpha.union(Set(beta))
    val configured = if (input.listed) ["Apache-2.0", "ISC", "MIT"] else ["Apache-2.0", "MIT"]
    // Report diagnostics with this code are ordered by message.
    val first = if (input.clarify) [] else ["clarification"]
    val second = if (input.betaScope) first else first.append("beta exception")
    val scoped = if (input.alphaScope) second else second.append("alpha exception")
    configured.foldl(scoped, (warnings, unit) =>
      if (used.contains(unit)) warnings else warnings.append(unit))
  }

  pure def evaluate(input: Scenario): Decision = {
    if (input.outcome != 0) { exit: 2, strictExit: 2, warnings: [] }
    else {
      val warnings = collectWarnings(input)
      val betaAllowed = if (input.clarify) input.deny or input.betaScope
                        else input.deny != input.listed
      {
        exit: if (betaAllowed) 0 else 1,
        strictExit: if (betaAllowed and warnings.length() == 0) 0 else 1,
        warnings: warnings,
      }
    }
  }

  action init = all {
    caseId' = 0,
    scenario' = input(0),
    decision' = evaluate(input(0)),
  }

  action step = {
    val index = if (caseId < 127) caseId + 1 else caseId
    val request = input(index)
    all {
      caseId' = index,
      scenario' = request,
      decision' = evaluate(request),
    }
  }

  val WarningContract = {
    val complete = scenario.outcome == 0
    val absent = (if (scenario.clarify) [] else ["clarification"])
      .concat(if (scenario.betaScope) [] else ["beta exception"])
      .concat(if (scenario.alphaScope) [] else ["alpha exception"])
      .concat(if (scenario.listed and scenario.clarify) ["ISC"] else [])
    all {
      decision.warnings == (if (complete) absent else []),
      (decision.exit == 2) == not(complete),
      (decision.exit == 0) == (complete and
        (if (scenario.clarify) scenario.deny or scenario.betaScope
         else scenario.deny != scenario.listed)),
      decision.strictExit == (if (decision.exit == 0 and absent.length() > 0) 1 else decision.exit),
    }
  }
}

module UnusedPolicy {
  import UnusedPolicyModel(MUTANT = false).*
}

module UnusedPolicyMutant {
  import UnusedPolicyModel(MUTANT = true).*
}
