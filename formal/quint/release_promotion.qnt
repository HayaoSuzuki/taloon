module ReleasePromotionModel {
  const MUTANT: bool
  type Scenario = { testPresent: bool, prodPresent: bool, fault: int, cancelAt: int }
  type Run = { phase: int, jobs: List[int], uploads: List[int], verified: bool }

  var caseId: int
  var scenario: Scenario
  var promotion: Run

  // Phases 1..6 are inspect, publish, verify for each index in order.
  // Faults 7 and 8 represent failed staging and disabled publication.
  pure def input(n: int): Scenario = {
    testPresent: n >= 126, prodPresent: (n / 63) % 2 == 1,
    fault: (n / 7) % 9, cancelAt: n % 7,
  }

  pure def initial(request: Scenario): Run = {
    phase: if (request.fault >= 7) 0 else 1,
    jobs: [], uploads: [], verified: false,
  }

  pure def advance(request: Scenario, current: Run): Run = {
    val phase = current.phase
    if (phase == request.cancelAt) current.with("phase", 0)
    else {
      val upload = (phase == 2 and not(request.testPresent)) or
                   (phase == 5 and not(request.prodPresent))
      val skippedUpload = (phase == 2 or phase == 5) and not(upload)
      val success = phase != request.fault or skippedUpload
      val proceed = success or (MUTANT and phase == 3)
      {
        phase: if (phase == 6 or not(proceed)) 0 else phase + 1,
        jobs: current.jobs.append(phase),
        uploads: if (upload) current.uploads.append(phase) else current.uploads,
        verified: phase == 6 and success,
      }
    }
  }

  action init = all { caseId' = 0, scenario' = input(0), promotion' = initial(input(0)) }

  action step =
    if (promotion.phase != 0) all {
      caseId' = caseId, scenario' = scenario, promotion' = advance(scenario, promotion),
    } else if (caseId < 251) all {
      caseId' = caseId + 1, scenario' = input(caseId + 1), promotion' = initial(input(caseId + 1)),
    } else all {
      caseId' = caseId, scenario' = scenario, promotion' = promotion,
    }

  val PromotionContract = {
    val blocked = scenario.fault >= 7
    val effectiveFault = scenario.fault != 0 and
      not(scenario.fault == 2 and scenario.testPresent) and
      not(scenario.fault == 5 and scenario.prodPresent)
    val jobs = [1, 2, 3, 4, 5, 6].foldl([], (prefix, phase) =>
      if (blocked or (scenario.cancelAt != 0 and phase >= scenario.cancelAt) or
          (effectiveFault and phase > scenario.fault)) prefix else prefix.append(phase))
    val testUpload = if (jobs.length() >= 2 and not(scenario.testPresent)) [2] else []
    val uploads = if (jobs.length() >= 5 and not(scenario.prodPresent))
      testUpload.append(5) else testUpload
    all {
      promotion.jobs.length() < 4 or scenario.fault != 3,
      promotion.phase != 0 or all {
        promotion.jobs == jobs,
        promotion.uploads == uploads,
        promotion.verified == (jobs.length() == 6 and scenario.fault != 6),
      },
    }
  }
}

module ReleasePromotion {
  import ReleasePromotionModel(MUTANT = false).*
}

module ReleasePromotionMutant {
  import ReleasePromotionModel(MUTANT = true).*
}
