import TaloonOracle.IndexLookup
import Lean.Data.Json

open TaloonOracle.IndexLookup

def profiles : List (List Entry) := [
  [], [⟨"a", some 1, some 3⟩], [⟨"b", some 2, some 4⟩],
  [⟨"a", some 2, some 4⟩], [⟨"a", none, none⟩],
  [⟨"a", none, some 3⟩], [⟨"a", some 1, none⟩],
  [⟨"a", some 1, some 3⟩, ⟨"a", some 2, some 4⟩],
  [⟨"a", some 1, some 3⟩, ⟨"b", some 2, some 4⟩, ⟨"x", some 1, none⟩] ]

def entryJson (entry : Entry) : Lean.Json := Lean.Json.mkObj [
  ("url", .str entry.url), ("archive", Lean.toJson entry.archive),
  ("metadata", Lean.toJson entry.metadata)]

def outcomeJson : Outcome → Lean.Json
  | .none => Lean.Json.mkObj [("kind", .str "none")]
  | .invalid => Lean.Json.mkObj [("kind", .str "InvalidMetadata")]
  | .hashMismatch => Lean.Json.mkObj [("kind", .str "HashMismatch")]
  | .associated metadata => Lean.Json.mkObj [("kind", .str "associated"), ("metadata", Lean.toJson metadata)]

def sensitivity : Bool :=
  lookup (snapshot ["a"] [⟨"a", some 1, some 3⟩, ⟨"b", some 2, some 4⟩]) "b" 2 != .associated 4 &&
  lookup [⟨"a", some 1, some 3⟩] "a" 2 != .associated 3 &&
  lookup [⟨"a", some 1, some 3⟩, ⟨"a", some 1, some 3⟩] "a" 1 != .associated 3

example : sensitivity = true := by decide

def corpus : String := Id.run do
  let mut rows : List String := []
  let mut number := 0
  for entries in profiles do
    for scope in [[], ["a"], ["b"], ["a", "b"]] do
      for url in ["a", "b", "x"] do
        for locked in [1, 2] do
          rows := rows ++ [(Lean.Json.mkObj [
            ("schema", Lean.toJson (1 : Nat)), ("id", .str s!"index-{number}"),
            ("entries", Lean.toJson (entries.map entryJson)), ("scope", Lean.toJson scope),
            ("url", .str url), ("locked", Lean.toJson locked),
            ("expected", outcomeJson (lookup (snapshot scope entries) url locked))]).compress]
          number := number + 1
  return String.intercalate "\n" rows ++ "\n"

def main (args : List String) : IO UInt32 := do
  unless sensitivity do
    IO.eprintln "index sensitivity failed"
    return 2
  match args with
  | ["--output", path] => IO.FS.writeFile path corpus; return 0
  | ["--check", path] =>
      if (← IO.FS.readFile path) == corpus then return 0
      IO.eprintln "stale index corpus"; return 1
  | [] => IO.print corpus; return 0
  | _ => IO.eprintln "usage: generate_index [--output PATH | --check PATH]"; return 2
