module LockedForksModel {
  const MUTANT: bool
  type Selector = {
    version: int, source: int, constraint: int, manifest: bool, marked: bool
  }
  type Decision = { accepted: bool, selected: int, marker: str }

  pure val allCandidates = Set(1, 2, 3)
  pure val versions = Map(1 -> 1, 2 -> 2, 3 -> 1)
  pure val sources = Map(1 -> 1, 2 -> 1, 3 -> 2)

  var caseId: int
  var selector: Selector
  var decision: Decision

  pure def input(number: int): Selector = {
    version: number / 48,
    source: (number / 12) % 4,
    constraint: (number / 4) % 3,
    manifest: (number / 2) % 2 == 1,
    marked: number % 2 == 1,
  }

  pure def matches(candidate: int, request: Selector): bool = {
    val version = versions.get(candidate)
    all {
      request.version == 0 or request.version == version,
      MUTANT or request.source == 0 or request.source == sources.get(candidate),
      request.constraint == 0 or (request.constraint == 1 and version >= 2)
        or (request.constraint == 2 and version < 2),
    }
  }

  pure def resolve(request: Selector): Decision = {
    val candidates = allCandidates.filter(candidate => matches(candidate, request))
    val unique = candidates.size() == 1
    {
      accepted: unique or request.manifest,
      selected: if (unique) candidates.fold(0, (_, candidate) => candidate) else 0,
      marker: if (unique and request.marked) "sys_platform == 'linux'" else "",
    }
  }

  action init = all {
    caseId' = 0,
    selector' = input(0),
    decision' = resolve(input(0)),
  }

  action step = {
    val index = if (caseId < 191) caseId + 1 else caseId
    val request = input(index)
    all {
      caseId' = index,
      selector' = request,
      decision' = resolve(request),
    }
  }

  // Independent tables state which locked identities each individual field admits.
  pure val exactVersions = Map(0 -> allCandidates, 1 -> Set(1, 3), 2 -> Set(2), 3 -> Set())
  pure val exactSources = Map(0 -> allCandidates, 1 -> Set(1, 2), 2 -> Set(3), 3 -> Set())
  pure val constraints = Map(0 -> allCandidates, 1 -> Set(2), 2 -> Set(1, 3))

  val UniqueIdentity = {
    val eligible = exactVersions.get(selector.version)
      .intersect(exactSources.get(selector.source))
      .intersect(constraints.get(selector.constraint))
    all {
      decision.accepted == (selector.manifest or eligible.size() == 1),
      Set(decision.selected) == (if (eligible.size() == 1) eligible else Set(0)),
      decision.marker == (if (eligible.size() == 1 and selector.marked)
                           "sys_platform == 'linux'" else ""),
    }
  }
}

module LockedForks {
  import LockedForksModel(MUTANT = false).*
}

module LockedForksMutant {
  import LockedForksModel(MUTANT = true).*
}
