module ReleaseReservationModel {
  const MUTANT: bool
  type Scenario = { seeded: bool, competitor: int, fault: int }
  type Reservation = {
    phase: str, tags: List[int], head: int, lease: int, attempts: int, result: int
  }

  var caseId: int
  var scenario: Scenario
  var reservation: Reservation

  pure val targetCommit = 2
  pure val exhausted = -2
  pure val stale = -1

  pure def input(number: int): Scenario = {
    seeded: number >= 16,
    competitor: (number / 4) % 4,
    fault: number % 4,
  }

  pure def initial(request: Scenario): Reservation = {
    phase: "read",
    tags: if (request.seeded) [0] else [],
    head: if (request.seeded) 0 else -1,
    lease: -1,
    attempts: 0,
    result: -3,
  }

  pure def tagOffset(tags: List[int]): int =
    tags.indices().fold(-1, (found, offset) =>
      if (tags[offset] == targetCommit) offset else found)

  pure def finish(current: Reservation, result: int): Reservation =
    current.with("phase", "done").with("result", result)

  // A completed push publishes both refs in the same transition.
  pure def publish(current: Reservation, commit: int): Reservation =
    current.with("tags", current.tags.append(commit)).with("head", commit)

  pure def advance(request: Scenario, current: Reservation): Reservation =
    if (current.phase == "read") {
      val existing = tagOffset(current.tags)
      if (existing >= 0) finish(current, existing)
      else if (current.head > targetCommit) finish(current, stale)
      else current.with("phase", "push").with("lease", current.head)
    } else if (current.phase == "recover") {
      val existing = tagOffset(current.tags)
      finish(current, if (existing >= 0 and not(MUTANT)) existing else exhausted)
    } else {
      val attempt = current.attempts + 1
      val racing = current.with("attempts", attempt)
      val remote = if (attempt == 1 and request.competitor != 0)
                     publish(racing, request.competitor) else racing
      val accepted = remote.head == current.lease and request.fault != 3
        and (request.fault != 2 or attempt == 5)
      val after = if (accepted) publish(remote, targetCommit) else remote
      if (accepted and request.fault == 0) finish(after, after.tags.length() - 1)
      else after.with("phase", if (attempt == 5) "recover" else "read")
    }

  action init = all {
    caseId' = 0, scenario' = input(0), reservation' = initial(input(0)),
  }

  action step =
    if (reservation.phase != "done") all {
      caseId' = caseId, scenario' = scenario, reservation' = advance(scenario, reservation),
    } else if (caseId < 31) all {
      caseId' = caseId + 1, scenario' = input(caseId + 1),
      reservation' = initial(input(caseId + 1)),
    } else all {
      caseId' = caseId, scenario' = scenario, reservation' = reservation,
    }

  val ReservationContract = {
    val existing = if (scenario.seeded) [0] else []
    val competing = if (scenario.competitor == 0) existing else existing.append(scenario.competitor)
    val callerPublishes = scenario.competitor.in(Set(0, 1)) and scenario.fault != 3
    val finalTags = if (callerPublishes) competing.append(targetCommit) else competing
    val expectedResult = if (scenario.competitor == 3) stale
      else if (callerPublishes or scenario.competitor == targetCommit) finalTags.length() - 1
      else exhausted
    val pushes = if (scenario.competitor.in(Set(2, 3))) 1
      else if (scenario.fault >= 2) 5
      else if (scenario.competitor == 1) 2 else 1
    all {
      reservation.attempts <= 5,
      reservation.head == (if (reservation.tags.length() == 0) -1
                            else reservation.tags[reservation.tags.length() - 1]),
      reservation.phase != "done" or all {
        reservation.tags == finalTags,
        reservation.result == expectedResult,
        reservation.attempts == pushes,
      },
    }
  }
}

module ReleaseReservation {
  import ReleaseReservationModel(MUTANT = false).*
}

module ReleaseReservationMutant {
  import ReleaseReservationModel(MUTANT = true).*
}
