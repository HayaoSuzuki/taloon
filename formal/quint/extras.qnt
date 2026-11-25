module ExtrasModel {
  const MUTANT: bool

  pure val hub = 0
  pure val bridge = 1
  pure val activatedHub = 2
  pure val target = 3

  type Scenario = { rootExtra: bool, returnExtra: bool, shortcut: bool }

  var caseId: int
  var scenario: Scenario
  var queue: List[List[int]]
  var seen: Set[int]
  var path: List[int]

  pure def initialQueue(input: Scenario): List[List[int]] = {
    val first = if (input.rootExtra) activatedHub else hub
    if (input.shortcut) [[first], [target]] else [[first]]
  }

  pure def successors(state: int, input: Scenario): List[int] =
    if (state == hub) [bridge]
    else if (state == activatedHub) [bridge, target]
    else if (state == bridge and input.returnExtra) [activatedHub]
    else if (state == bridge) [hub]
    else []

  pure def identity(state: int): int =
    if (MUTANT and state == activatedHub) hub else state

  pure def scenarioFor(caseNumber: int): Scenario = {
    rootExtra: caseNumber.in(Set(2, 3, 6, 7)),
    returnExtra: caseNumber % 2 == 1,
    shortcut: caseNumber >= 4,
  }

  action init = all {
    caseId' = 0,
    scenario' = scenarioFor(0),
    queue' = initialQueue(scenarioFor(0)),
    seen' = Set(),
    path' = [],
  }

  action startNext = {
    val number = caseId + 1
    val input = scenarioFor(number)
    all {
      caseId' = number,
      scenario' = input,
      queue' = initialQueue(input),
      seen' = Set(),
      path' = [],
    }
  }

  action visit = all {
    caseId' = caseId,
    scenario' = scenario,
    if (queue.length() == 0) all {
      queue' = queue,
      seen' = seen,
      path' = path,
    } else {
      val candidate = queue.head()
      val state = candidate[candidate.length() - 1]
      val key = identity(state)
      if (seen.contains(key)) all {
        queue' = queue.tail(),
        seen' = seen,
        path' = path,
      } else if (state == target) all {
        queue' = [],
        seen' = seen.union(Set(key)),
        path' = candidate,
      } else all {
        queue' = successors(state, scenario).foldl(queue.tail(), (waiting, successor) =>
          waiting.append(candidate.append(successor))),
        seen' = seen.union(Set(key)),
        path' = path,
      }
    },
  }

  action step =
    if (queue.length() == 0 and caseId < 7) startNext
    else visit

  // The shape gives exactly three ways to reach the optional target.
  pure def shortest(input: Scenario): List[int] =
    if (input.shortcut) [target]
    else if (input.rootExtra) [activatedHub, target]
    else if (input.returnExtra) [hub, bridge, activatedHub, target]
    else []

  val ExtrasContract = queue.length() > 0 or path == shortest(scenario)
}

module Extras {
  import ExtrasModel(MUTANT = false).*
}

module ExtrasMutant {
  import ExtrasModel(MUTANT = true).*
}
