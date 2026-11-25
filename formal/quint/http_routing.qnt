module HttpRoutingModel {
  const MUTANT: bool
  type Scenario = { route: int, burst: int, terminal: int }
  type Call = { hop: int, endpoint: int, authenticated: bool, response: int, redirect: int }
  type Session = {
    hop: int, attempt: int, trusted: bool, outcome: str, calls: List[Call], bytes: int
  }

  var caseId: int
  var scenario: Scenario
  var session: Session

  pure def input(number: int): Scenario = {
    route: number / 16,
    burst: (number / 4) % 4,
    terminal: number % 4,
  }

  pure val empty: Session = {
    hop: 0, attempt: 0, trusted: true, outcome: "", calls: [], bytes: 0
  }

  // Routes: direct; same origin; out-and-back by host or port;
  // HTTPS downgrade; and a same-origin redirect loop.
  pure def endpoints(route: int): List[int] =
    if (route == 0) [0]
    else if (route == 1) [0, 1]
    else if (route == 2) [0, 2, 4]
    else if (route == 3) [0, 3, 4]
    else if (route == 4) [0, 5]
    else [0, 0, 0, 0, 0, 0, 0]

  pure val bursts = Map(0 -> 0, 1 -> 1, 2 -> 2, 3 -> 1)
  pure val transient = Map(0 -> 200, 1 -> 429, 2 -> 503, 3 -> -1)
  pure val terminalCodes = Map(0 -> 200, 1 -> 404, 2 -> 503, 3 -> -1)
  pure def retryable(code: int): bool = code == -1 or code == 429 or code >= 500
  pure def originalOrigin(endpoint: int): bool = endpoint.in(Set(0, 1, 4))

  pure def advance(input: Scenario, current: Session): Session = {
    val route = endpoints(input.route)
    val endpoint = route[current.hop]
    if (endpoint == 5) current.with("outcome", "downgrade") else {
      val nextHop = current.hop + 1
      val redirecting = nextHop < route.length()
      val code = if (current.attempt < bursts.get(input.burst)) transient.get(input.burst)
                 else if (redirecting) 302 else terminalCodes.get(input.terminal)
      val destination = if (code == 302) route[nextHop] else -1
      val authenticated = if (MUTANT) originalOrigin(endpoint) else current.trusted
      val call = {
        hop: current.hop, endpoint: endpoint, authenticated: authenticated,
        response: code, redirect: destination
      }
      val updated = current.with("calls", current.calls.append(call))
        .with("bytes", current.bytes + (if (code == -1) 3 else 2))
      if (retryable(code) and current.attempt < 2)
        updated.with("attempt", current.attempt + 1)
      else if (code == 302 and nextHop == 6)
        updated.with("outcome", "redirect_limit")
      else if (code == 302)
        updated.with("hop", nextHop).with("attempt", 0)
          .with("trusted", current.trusted and originalOrigin(destination))
      else updated.with("outcome",
        if (code == 200) "success" else if (code == -1) "transport" else "terminal")
    }
  }

  action init = all {
    caseId' = 0, scenario' = input(0), session' = empty,
  }

  action step = {
    if (session.outcome == "") all {
      caseId' = caseId,
      scenario' = scenario,
      session' = advance(scenario, session),
    } else if (caseId < 95) all {
      caseId' = caseId + 1,
      scenario' = input(caseId + 1),
      session' = empty,
    } else all { caseId' = caseId, scenario' = scenario, session' = session }
  }

  val RoutingContract = {
    val crossesOrigin = scenario.route.in(Set(2, 3))
    val secureHops = [1, 2, 3, 3, 1, 6][scenario.route]
    val prefix = bursts.get(scenario.burst) + 1
    val terminalAttempts = if (scenario.route < 4 and scenario.terminal >= 2) 3 else prefix
    val expectedOutcome = if (scenario.route == 4) "downgrade"
      else if (scenario.route == 5) "redirect_limit"
      else if (scenario.terminal == 0) "success"
      else if (scenario.terminal == 3) "transport" else "terminal"
    all {
      session.calls.foldl(true, (ok, call) =>
        ok and call.endpoint != 5 and call.authenticated == (not(crossesOrigin) or call.hop == 0)),
      session.calls.length() <= secureHops * 3,
      session.outcome == "" or all {
        session.calls.length() == (secureHops - 1) * prefix + terminalAttempts,
        session.outcome == expectedOutcome,
      },
    }
  }
}

module HttpRouting {
  import HttpRoutingModel(MUTANT = false).*
}

module HttpRoutingMutant {
  import HttpRoutingModel(MUTANT = true).*
}
