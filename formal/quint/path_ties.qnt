module PathTiesModel {
  const MUTANT: bool
  type Scenario = { mode: int, baseEdge: bool, shortcut: bool, reversed: bool }
  type PathStep = { package: str, marker: str, via: List[str] }

  var caseId: int
  var scenario: Scenario
  var route: int
  var path: List[PathStep]
  var development: List[PathStep]

  pure def input(number: int): Scenario = {
    mode: number / 8,
    baseEdge: (number / 4) % 2 == 1,
    shortcut: (number / 2) % 2 == 1,
    reversed: number % 2 == 1,
  }

  pure val linux = "sys_platform == 'linux'"
  pure val windows = "sys_platform == 'win32'"
  pure def markerA(input: Scenario): str = if (input.mode < 2) linux else windows
  pure def markerB(input: Scenario): str =
    if (input.mode == 1) windows else if (input.mode == 3) "" else linux

  // Routes 1..3 enter hub with extras a,b. Routes 4..5 enter with b.
  // The remaining routes are the root shortcut and the competing other branch.
  pure def candidates(input: Scenario): Set[int] =
    Set(6)
      .union(if (input.shortcut) Set(0) else Set())
      .union(if (input.baseEdge) Set(1, 4) else Set())
      .union(Set(2, 3, 5))

  pure def annotationRank(marker: str): int =
    if (marker == "") 0 else if (marker == linux) 1 else 2

  pure def priority(choice: int, input: Scenario): int = {
    val viaA = choice.in(Set(1, 2, 3))
    val wireRank = if (choice == 6) 2 else if (viaA) 0 else 1
    val rootRank = if (MUTANT) {
      if (input.reversed) 2 - wireRank else wireRank
    } else if (choice == 6) 6
    else if (viaA) annotationRank(markerA(input)) * 2
    else annotationRank(markerB(input)) * 2 + 1
    val childRank = if (choice.in(Set(1, 4, 6))) 0 else if (choice == 2) 1 else 2
    if (choice == 0) 0 else 100 + rootRank * 10 + childRank
  }

  pure def choose(input: Scenario): int =
    candidates(input).fold(6, (best, candidate) =>
      if (priority(candidate, input) < priority(best, input)) candidate else best)

  pure def target(marker: str, via: List[str]): PathStep =
    { package: "target", marker: marker, via: via }

  pure def annotate(choice: int, input: Scenario): List[PathStep] =
    if (choice == 0) [target("", [])]
    else if (choice == 6) [{ package: "other", marker: "", via: [] }, target("", [])]
    else {
      val viaA = choice.in(Set(1, 2, 3))
      val first = {
        package: "hub",
        marker: if (viaA) markerA(input) else markerB(input),
        via: if (viaA) ["extra:a", "extra:b"] else ["extra:b"],
      }
      val last = if (choice.in(Set(1, 4))) target(windows, [])
                 else if (choice == 2) target(windows, ["extra:a"])
                 else target(linux, ["extra:b"])
      [first, last]
    }

  pure val devPath = [
    { package: "other", marker: "", via: ["group:dev"] }, target("", [])
  ]

  action init = all {
    caseId' = 0,
    scenario' = input(0),
    route' = choose(input(0)),
    path' = annotate(choose(input(0)), input(0)),
    development' = devPath,
  }

  action step = {
    val index = if (caseId < 31) caseId + 1 else caseId
    val request = input(index)
    val winner = choose(request)
    all {
      caseId' = index,
      scenario' = request,
      route' = winner,
      path' = annotate(winner, request),
      development' = devPath,
    }
  }

  // Direct reasoning about which branch wins each of the fixture's ties.
  pure def expected(input: Scenario): int =
    if (input.shortcut) 0
    else if (input.mode < 2) {
      if (input.baseEdge) 1 else 2
    } else if (input.baseEdge) 4 else 5

  val CanonicalPath = route == expected(scenario)
}

module PathTies {
  import PathTiesModel(MUTANT = false).*
}

module PathTiesMutant {
  import PathTiesModel(MUTANT = true).*
}
