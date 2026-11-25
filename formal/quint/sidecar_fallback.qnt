module SidecarFallbackModel {
  const MUTANT: bool
  type Scenario = {
    advertisement: int, metadata: int, strong: bool, archive: int, html: bool
  }
  type Decision = {
    indexRequests: int, metadataRequests: int, archiveRequests: int,
    proof: str, expression: str, warning: str, issue: str,
    exit: int, offlineHits: int
  }

  var caseId: int
  var scenario: Scenario
  var decision: Decision

  pure def input(number: int): Scenario = {
    advertisement: number / 60,
    metadata: (number / 12) % 5,
    strong: (number / 6) % 2 == 1,
    archive: (number / 2) % 3,
    html: number % 2 == 1,
  }

  // Advertisement alternatives: exact, wrong URL, wrong archive hash, duplicate.
  // Metadata alternatives: exact, wrong bytes, wrong identity, malformed, missing.
  pure val admission = Map(0 -> "fetch", 1 -> "", 2 -> "hash_mismatch", 3 -> "invalid_metadata")
  pure val metadataResponse = Map(0 -> "accept", 1 -> "hash_mismatch",
    2 -> "identity_mismatch", 3 -> "invalid_metadata", 4 -> "")
  pure val archiveResponse = Map(0 -> "archive", 1 -> "hash_mismatch", 2 -> "acquisition")

  pure def count(condition: bool): int = if (condition) 1 else 0

  pure def evaluate(input: Scenario): Decision = {
    val offer = if (input.strong) "" else admission.get(input.advertisement)
    val sidecar =
      if (offer != "fetch") offer
      else if (MUTANT and input.metadata == 1) "accept"
      else metadataResponse.get(input.metadata)
    val archive = if (sidecar == "accept") "" else archiveResponse.get(input.archive)
    val proof = if (sidecar == "accept") "index" else if (archive == "archive") "archive" else ""
    {
      indexRequests: count(not(input.strong)),
      metadataRequests: count(offer == "fetch"),
      archiveRequests: count(sidecar != "accept"),
      proof: proof,
      expression: if (proof == "index") "MIT" else if (proof == "archive") "Apache-2.0" else "",
      warning: if (sidecar == "accept") "" else sidecar,
      issue: if (proof == "") archive else "",
      exit: if (proof == "") 2 else 0,
      offlineHits: count(proof != ""),
    }
  }

  action init = all {
    caseId' = 0, scenario' = input(0), decision' = evaluate(input(0)),
  }

  action step = {
    val index = if (caseId < 239) caseId + 1 else caseId
    val request = input(index)
    all { caseId' = index, scenario' = request, decision' = evaluate(request) }
  }

  val FallbackContract = {
    val metadataAllowed = not(scenario.strong) and scenario.advertisement == 0
    val indexProof = metadataAllowed and scenario.metadata == 0
    val archiveProof = not(indexProof) and scenario.archive == 0
    all {
      decision.indexRequests == count(not(scenario.strong)),
      decision.metadataRequests == count(metadataAllowed),
      decision.archiveRequests == count(not(indexProof)),
      decision.proof.in(Set("", "index", "archive")),
      (decision.proof == "index") == indexProof,
      (decision.proof == "archive") == archiveProof,
      decision.expression == (if (indexProof) "MIT" else if (archiveProof) "Apache-2.0" else ""),
      decision.offlineHits == count(indexProof or archiveProof),
      decision.exit == (if (indexProof or archiveProof) 0 else 2),
      decision.issue == (if (indexProof or archiveProof) "" else
        if (scenario.archive == 1) "hash_mismatch" else "acquisition"),
      decision.warning.in(Set("", "hash_mismatch", "identity_mismatch", "invalid_metadata")),
      (decision.warning == "hash_mismatch") == (not(scenario.strong) and
        (scenario.advertisement == 2 or (metadataAllowed and scenario.metadata == 1))),
      (decision.warning == "identity_mismatch") == (metadataAllowed and scenario.metadata == 2),
      (decision.warning == "invalid_metadata") == (not(scenario.strong) and
        (scenario.advertisement == 3 or (metadataAllowed and scenario.metadata == 3))),
    }
  }
}

module SidecarFallback {
  import SidecarFallbackModel(MUTANT = false).*
}

module SidecarFallbackMutant {
  import SidecarFallbackModel(MUTANT = true).*
}
