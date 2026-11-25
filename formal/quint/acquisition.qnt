module AcquisitionModel {
  const MUTANT: bool
  pure val caches = Set("miss", "complete", "needsFiles")
  pure val sidecars = Set("unavailable", "complete", "needsFiles")

  var request: { offline: bool, strong: bool, cache: str, sidecar: str }
  var phase: str
  var steps: int

  action init = {
    nondet offline = Bool.oneOf()
    nondet strong = Bool.oneOf()
    nondet cache = caches.oneOf()
    nondet sidecar = sidecars.oneOf()
    all {
      request' = { offline: offline, strong: strong, cache: cache, sidecar: sidecar },
      phase' = "start",
      steps' = 0,
    }
  }

  action step = all {
    request' = request,
    steps' = if (steps < 2) steps + 1 else steps,
    phase' = if (phase == "start") {
      if (request.cache == "complete" or (request.offline and request.cache == "needsFiles")) "cached"
      else if (request.offline and not(MUTANT)) "offlineMiss"
      else if (request.strong or request.cache == "needsFiles") "archive"
      else "sidecar"
    } else if (phase == "sidecar") {
      if (request.sidecar == "complete") "sidecarAccepted" else "archive"
    } else phase,
  }

  val AcquisitionContract = all {
    (phase == "start") == (steps == 0),
    not(steps == 2 and phase == "sidecar"),
    not(request.offline and phase.in(Set("sidecar", "archive", "sidecarAccepted"))),
    (phase != "cached" or request.cache == "complete" or
      (request.offline and request.cache == "needsFiles")),
    (phase != "offlineMiss" or (request.offline and request.cache == "miss")),
    (not(phase.in(Set("sidecar", "sidecarAccepted"))) or
      (not(request.strong) and request.cache == "miss")),
    (phase != "sidecarAccepted" or request.sidecar == "complete"),
    (phase != "archive" or request.cache == "needsFiles" or
      (request.cache == "miss" and
        (request.strong or (steps == 2 and request.sidecar != "complete")))),
  }
}

module Acquisition {
  import AcquisitionModel(MUTANT = false).*
}

module AcquisitionMutant {
  import AcquisitionModel(MUTANT = true).*
}
