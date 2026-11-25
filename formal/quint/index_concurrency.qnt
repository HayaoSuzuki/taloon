module IndexConcurrencyModel {
  const MUTANT: bool
  pure val workers = Set(1, 2)
  var owner: int
  var snapshot: str
  var results: int -> str
  var attempts: int
  var publications: int

  action init = all {
    owner' = 0,
    snapshot' = "empty",
    results' = Map(1 -> "waiting", 2 -> "waiting"),
    attempts' = 0,
    publications' = 0,
  }

  action claim = {
    nondet worker = workers.oneOf()
    all {
      owner == 0 and snapshot == "empty" and results.get(worker) == "waiting",
      owner' = worker,
      snapshot' = snapshot,
      results' = results,
      attempts' = attempts + 1,
      publications' = publications,
    }
  }

  action finish = {
    nondet outcome = Set("ready", "error", "panic").oneOf()
    all {
      owner != 0,
      owner' = 0,
      snapshot' = if (outcome == "panic" or (MUTANT and outcome == "error"))
                    "empty" else outcome,
      results' = results.set(owner, outcome),
      attempts' = attempts,
      publications' = publications + (if (outcome == "panic") 0 else 1),
    }
  }

  action reuse = {
    nondet worker = workers.oneOf()
    all {
      owner == 0 and snapshot != "empty" and results.get(worker) == "waiting",
      owner' = 0,
      snapshot' = snapshot,
      results' = results.set(worker, snapshot),
      attempts' = attempts,
      publications' = publications,
    }
  }

  action done = all {
    workers.forall(worker => results.get(worker) != "waiting"),
    owner' = owner,
    snapshot' = snapshot,
    results' = results,
    attempts' = attempts,
    publications' = publications,
  }

  action step = any { claim, finish, reuse, done }

  val SnapshotContract = all {
    publications <= 1,
    (publications == 0) == (snapshot == "empty"),
    attempts == workers.filter(worker => results.get(worker) == "panic").size()
      + publications + (if (owner == 0) 0 else 1),
    attempts <= workers.size(),
    workers.forall(worker => {
      val result = results.get(worker)
      result == "waiting" or result == "panic" or result == snapshot
    }),
  }
}

module IndexConcurrency {
  import IndexConcurrencyModel(MUTANT = false).*
}

module IndexConcurrencyMutant {
  import IndexConcurrencyModel(MUTANT = true).*
}
