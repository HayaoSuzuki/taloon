module LifecycleModel {
  const MUTANT: bool
  type Operation = {
    offline: bool, clear: bool, revision: int, delivery: str,
    permitted: bool, duplicate: bool
  }
  type Report = { parent: int, child: int, exit: int, hits: int, attempts: int }

  // Content keys: parent = 0, child digest A = 1, child digest B = 2.
  var cache: Set[int]
  var before: Set[int]
  var operation: Operation
  var report: Report

  action init = all {
    cache' = Set(),
    before' = Set(),
    operation' = {
      offline: true, clear: false, revision: 1, delivery: "missing",
      permitted: false, duplicate: false
    },
    report' = { parent: 2, child: 2, exit: 2, hits: 0, attempts: 0 },
  }

  pure def count(condition: bool): int = if (condition) 1 else 0

  action step = {
    nondet offline = Bool.oneOf()
    nondet clear = Bool.oneOf()
    nondet revision = Set(1, 2).oneOf()
    nondet delivery = Set("current", "retry", "stale", "missing").oneOf()
    nondet permitted = Bool.oneOf()
    nondet duplicate = Bool.oneOf()
    val existing = if (clear) Set() else cache
    val parentHit = existing.contains(0)
    val childHit = existing.contains(revision) or
      (MUTANT and existing.exists(key => key != 0))
    val fetchParent = not(offline or parentHit)
    val fetchChild = not(offline or childHit)
    val fetchedChild = fetchChild and delivery.in(Set("current", "retry"))
    val parent = if (parentHit or fetchParent) count(not(permitted)) else 2
    val child = if (not(duplicate) and (childHit or fetchedChild)) count(not(permitted)) else 2
    all {
      cache' = existing.union(Set(0).filter(_ => fetchParent))
                       .union(Set(revision).filter(_ => fetchedChild)),
      before' = cache,
      operation' = {
        offline: offline, clear: clear, revision: revision, delivery: delivery,
        permitted: permitted, duplicate: duplicate
      },
      report' = {
        parent: parent, child: child,
        exit: if (parent > child) parent else child,
        hits: count(parentHit) + count(childHit),
        attempts: count(fetchParent) + count(fetchChild) + count(fetchChild and delivery == "retry")
      },
    }
  }

  val LifecycleContract = {
    val required = Set(0, operation.revision)
    val retained = if (operation.clear) Set() else before
    val hits = required.intersect(retained)
    val misses = required.exclude(retained)
    val downloadable = if (operation.offline) Set()
      else if (operation.delivery.in(Set("stale", "missing"))) Set(0)
      else required
    val fresh = misses.intersect(downloadable)
    val account = hits.union(fresh)
    val complete = required.subseteq(account) and not(operation.duplicate)
    val checked = if (operation.duplicate) account.exclude(Set(operation.revision)) else account
    val allowed = if (operation.permitted) checked else Set()
    def expected(key: int): int =
      2 - count(checked.contains(key)) - count(allowed.contains(key))
    all {
      cache == retained.union(fresh),
      report.hits == hits.size(),
      report.attempts == (if (operation.offline) 0 else misses.size() +
        count(misses.contains(operation.revision) and operation.delivery == "retry")),
      report.parent == expected(0),
      report.child == expected(operation.revision),
      report.exit == (if (complete) count(not(operation.permitted)) else 2),
    }
  }
}

module Lifecycle {
  import LifecycleModel(MUTANT = false).*
}

module LifecycleMutant {
  import LifecycleModel(MUTANT = true).*
}
