module SharedCacheModel {
  const MUTANT: bool
  pure val digests = Set(1, 2)

  var writers: int -> int
  var visible: Set[int]
  var published: Set[int]
  var pinnedReader: int
  var event: { kind: str, key: int, observed: int }

  action init = all {
    writers' = Map(1 -> 0, 2 -> 0),
    visible' = Set(),
    published' = Set(),
    pinnedReader' = 0,
    event' = { kind: "init", key: 0, observed: 0 },
  }

  action write = {
    nondet key = digests.oneOf()
    val phase = writers.get(key)
    all {
      phase != 0 or not(visible.contains(key)),
      writers' = writers.set(key, (phase + 1) % 3),
      visible' = if (phase == 2 or (MUTANT and phase == 0))
                   visible.union(Set(key)) else visible,
      published' = if (phase == 2) published.union(Set(key)) else published,
      pinnedReader' = pinnedReader,
      event' = { kind: if (phase == 0) "begin" else if (phase == 1) "sync" else "publish",
                 key: key, observed: 0 },
    }
  }

  action clear = all {
    writers' = writers,
    visible' = Set(),
    published' = Set(),
    pinnedReader' = pinnedReader,
    event' = { kind: "clear", key: 0, observed: visible.size() },
  }

  action lookup = {
    nondet key = digests.oneOf()
    all {
      writers' = writers,
      visible' = visible,
      published' = published,
      pinnedReader' = pinnedReader,
      event' = { kind: "lookup", key: key, observed: if (visible.contains(key)) key else 0 },
    }
  }

  action open = {
    nondet key = digests.oneOf()
    all {
      pinnedReader == 0 and visible.contains(key),
      writers' = writers,
      visible' = visible,
      published' = published,
      pinnedReader' = key,
      event' = { kind: "open", key: key, observed: 0 },
    }
  }

  action finishRead = all {
    pinnedReader != 0,
    writers' = writers,
    visible' = visible,
    published' = published,
    pinnedReader' = 0,
    event' = { kind: "finishRead", key: pinnedReader, observed: pinnedReader },
  }

  action step = any { write, clear, lookup, open, finishRead }

  val PublicationContract = all {
    visible == published,
    event.kind != "lookup" or
      event.observed == (if (published.contains(event.key)) event.key else 0),
    event.kind != "finishRead" or event.observed == event.key,
  }
}

module SharedCache {
  import SharedCacheModel(MUTANT = false).*
}

module SharedCacheMutant {
  import SharedCacheModel(MUTANT = true).*
}
