module DescriptionEvidenceModel {
  const MUTANT: bool

  type Scenario = { left: int, right: int, foreign: bool, reverse: bool }
  type Observation = { slot: int, expression: str, issue: str }
  type Artifact = { expression: str, issue: str }
  type Group = { expression: str, members: List[int] }
  type Decision = {
    exit: int, unknown: int, artifacts: List[Artifact], groups: List[Group],
    foreign: bool
  }

  pure val expressions = ["BSD-3-Clause", "LicenseRef-Local", "MIT"]
  var caseId: int
  var scenario: Scenario
  var decision: Decision

  pure def input(number: int): Scenario = {
    left: number / 36,
    right: (number / 4) % 9,
    foreign: (number / 2) % 2 == 1,
    reverse: number % 2 == 1,
  }

  pure def observation(slot: int, expression: str, issue: str): Observation =
    { slot: slot, expression: expression, issue: issue }

  pure def events(slot: int, outcome: int): List[Observation] =
    if (outcome == 0) []
    else if (outcome.in(Set(1, 2))) [observation(slot, "MIT", "")]
    else if (outcome == 3) [observation(slot, "BSD-3-Clause", "")]
    else if (outcome == 4) [observation(slot, "LicenseRef-Local", "")]
    else if (outcome == 5) [observation(slot, "", "unknown_license")]
    else if (outcome == 6) [observation(slot, "MIT", ""), observation(slot, "", "acquisition")]
    else if (outcome == 7) [observation(slot, "", "acquisition")]
    else [observation(slot, "", "hash_mismatch")]

  pure def account(batch: List[Observation], slot: int): Artifact = {
    val matching = batch.foldl([], (found, entry) =>
      if (entry.slot == slot) found.append(entry) else found)
    if (matching.length() == 0) { expression: "", issue: "missing_result" }
    else if (matching.length() > 1 and not(MUTANT)) { expression: "", issue: "invalid_result" }
    else { expression: matching[0].expression, issue: matching[0].issue }
  }

  pure def evaluate(input: Scenario): Decision = {
    val selected = events(0, input.left).concat(events(1, input.right))
    val supplied = if (input.foreign) selected.append(observation(2, "", "acquisition")) else selected
    val batch = if (input.reverse)
                  supplied.foldl([], (items, entry) => [entry].concat(items))
                else supplied
    val artifacts = [account(batch, 0), account(batch, 1)]
    val unavailable = artifacts.foldl(0, (count, artifact) =>
      if (artifact.expression == "") count + 1 else count)
    val hasForeign = batch.foldl(false, (found, entry) => found or entry.slot == 2)
    val groups = expressions.foldl([], (groups, expression) => {
      val members = [0, 1].foldl([], (items, slot) =>
        if (artifacts[slot].expression == expression) items.append(slot) else items)
      if (members.length() == 0) groups
      else groups.append({ expression: expression, members: members })
    })
    {
      exit: if (unavailable > 0 or hasForeign) 2 else 0,
      unknown: unavailable, artifacts: artifacts, groups: groups, foreign: hasForeign,
    }
  }

  action init = all {
    caseId' = 0, scenario' = input(0), decision' = evaluate(input(0)),
  }

  action step = {
    val index = if (caseId < 323) caseId + 1 else caseId
    val request = input(index)
    all { caseId' = index, scenario' = request, decision' = evaluate(request) }
  }

  // The lookup table specifies each input class independently of the ledger fold.
  pure val known = Map(1 -> "MIT", 2 -> "MIT", 3 -> "BSD-3-Clause", 4 -> "LicenseRef-Local")
  pure def expected(outcome: int): Artifact =
    if (outcome.in(known.keys())) { expression: known.get(outcome), issue: "" }
    else {
      val issues = Map(0 -> "missing_result", 5 -> "unknown_license", 6 -> "invalid_result",
                       7 -> "acquisition", 8 -> "hash_mismatch")
      { expression: "", issue: issues.get(outcome) }
    }

  val DescriptionContract = {
    val left = expected(scenario.left)
    val right = expected(scenario.right)
    val unknown = (if (left.expression == "") 1 else 0) + (if (right.expression == "") 1 else 0)
    val grouped = decision.groups.foldl(Set(), (pairs, group) =>
      group.members.foldl(pairs, (items, member) => items.union(Set((group.expression, member)))))
    val expectedPairs = (if (left.expression == "") Set() else Set((left.expression, 0)))
      .union(if (right.expression == "") Set() else Set((right.expression, 1)))
    all {
      decision.artifacts == [left, right],
      decision.unknown == unknown,
      decision.foreign == scenario.foreign,
      decision.exit == (if (unknown > 0 or scenario.foreign) 2 else 0),
      grouped == expectedPairs,
      decision.groups.length() == expectedPairs.map(pair => pair._1).size(),
    }
  }
}

module DescriptionEvidence {
  import DescriptionEvidenceModel(MUTANT = false).*
}

module DescriptionEvidenceMutant {
  import DescriptionEvidenceModel(MUTANT = true).*
}
